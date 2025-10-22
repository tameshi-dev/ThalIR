use std::any::{Any, TypeId};
use std::collections::{HashMap, VecDeque};
use std::sync::{Arc, RwLock};
use std::time::{Duration, Instant};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct CacheKey {
    pub analysis_type: TypeId,
    pub target: String,
    pub version: u64,
}

impl CacheKey {
    pub fn new<T: 'static>(target: impl Into<String>, version: u64) -> Self {
        Self {
            analysis_type: TypeId::of::<T>(),
            target: target.into(),
            version,
        }
    }
}

struct CacheEntry {
    value: Box<dyn Any + Send + Sync>,
    last_accessed: Instant,
    size_bytes: usize,
    access_count: u64,
}

pub struct AnalysisCache {
    entries: HashMap<CacheKey, CacheEntry>,
    lru_order: VecDeque<CacheKey>,
    max_size_bytes: usize,
    current_size_bytes: usize,
    max_entries: usize,
    generation: u64,
    stats: CacheStatistics,
}

#[derive(Debug, Clone, Default)]
pub struct CacheStatistics {
    pub hits: u64,
    pub misses: u64,
    pub evictions: u64,
    pub invalidations: u64,
    pub total_compute_time: Duration,
}

impl AnalysisCache {
    pub fn new(max_size_bytes: usize, max_entries: usize) -> Self {
        Self {
            entries: HashMap::new(),
            lru_order: VecDeque::new(),
            max_size_bytes,
            current_size_bytes: 0,
            max_entries,
            generation: 0,
            stats: CacheStatistics::default(),
        }
    }

    pub fn get_or_compute<T, F>(&mut self, key: CacheKey, compute: F) -> Arc<T>
    where
        T: Send + Sync + 'static,
        F: FnOnce() -> T,
    {
        let cached_value = if let Some(entry) = self.entries.get_mut(&key) {
            if let Some(value) = entry.value.downcast_ref::<Arc<T>>() {
                self.stats.hits += 1;
                entry.last_accessed = Instant::now();
                entry.access_count += 1;
                Some(value.clone())
            } else {
                None
            }
        } else {
            None
        };

        if let Some(value) = cached_value {
            self.promote_lru(&key);
            return value;
        }

        self.stats.misses += 1;
        let start = Instant::now();
        let value = Arc::new(compute());
        self.stats.total_compute_time += start.elapsed();

        let size_bytes = std::mem::size_of::<T>() + 64;

        self.evict_if_needed(size_bytes);

        let entry = CacheEntry {
            value: Box::new(value.clone()),
            last_accessed: Instant::now(),
            size_bytes,
            access_count: 1,
        };

        self.entries.insert(key.clone(), entry);
        self.lru_order.push_back(key);
        self.current_size_bytes += size_bytes;

        value
    }

    pub fn get<T>(&mut self, key: &CacheKey) -> Option<Arc<T>>
    where
        T: Send + Sync + 'static,
    {
        let result = if let Some(entry) = self.entries.get_mut(key) {
            if let Some(value) = entry.value.downcast_ref::<Arc<T>>() {
                self.stats.hits += 1;
                entry.last_accessed = Instant::now();
                entry.access_count += 1;
                Some(value.clone())
            } else {
                None
            }
        } else {
            None
        };

        if result.is_some() {
            self.promote_lru(key);
        } else {
            self.stats.misses += 1;
        }

        result
    }

    pub fn insert<T>(&mut self, key: CacheKey, value: T)
    where
        T: Send + Sync + 'static,
    {
        let arc_value = Arc::new(value);
        let size_bytes = std::mem::size_of::<T>() + 64;

        self.evict_if_needed(size_bytes);

        let entry = CacheEntry {
            value: Box::new(arc_value),
            last_accessed: Instant::now(),
            size_bytes,
            access_count: 0,
        };

        self.entries.insert(key.clone(), entry);
        self.lru_order.push_back(key);
        self.current_size_bytes += size_bytes;
    }

    pub fn invalidate<F>(&mut self, predicate: F)
    where
        F: Fn(&CacheKey) -> bool,
    {
        let keys_to_remove: Vec<CacheKey> = self
            .entries
            .keys()
            .filter(|k| predicate(k))
            .cloned()
            .collect();

        for key in keys_to_remove {
            if let Some(entry) = self.entries.remove(&key) {
                self.current_size_bytes -= entry.size_bytes;
                self.stats.invalidations += 1;
            }
            self.lru_order.retain(|k| k != &key);
        }
    }

    pub fn invalidate_target(&mut self, target: &str) {
        self.invalidate(|k| k.target == target);
    }

    pub fn increment_generation(&mut self) {
        let current_gen = self.generation;
        self.generation += 1;
        self.invalidate(|k| k.version < current_gen);
    }

    pub fn clear(&mut self) {
        self.entries.clear();
        self.lru_order.clear();
        self.current_size_bytes = 0;
        self.stats.invalidations += self.entries.len() as u64;
    }

    pub fn statistics(&self) -> &CacheStatistics {
        &self.stats
    }

    pub fn hit_rate(&self) -> f64 {
        let total = self.stats.hits + self.stats.misses;
        if total == 0 {
            0.0
        } else {
            self.stats.hits as f64 / total as f64
        }
    }

    fn promote_lru(&mut self, key: &CacheKey) {
        self.lru_order.retain(|k| k != key);
        self.lru_order.push_back(key.clone());
    }

    fn evict_if_needed(&mut self, needed_bytes: usize) {
        while self.entries.len() >= self.max_entries {
            if let Some(key) = self.lru_order.pop_front() {
                if let Some(entry) = self.entries.remove(&key) {
                    self.current_size_bytes -= entry.size_bytes;
                    self.stats.evictions += 1;
                }
            } else {
                break;
            }
        }

        while self.current_size_bytes + needed_bytes > self.max_size_bytes {
            if let Some(key) = self.lru_order.pop_front() {
                if let Some(entry) = self.entries.remove(&key) {
                    self.current_size_bytes -= entry.size_bytes;
                    self.stats.evictions += 1;
                }
            } else {
                break;
            }
        }
    }
}

impl Default for AnalysisCache {
    fn default() -> Self {
        Self::new(100 * 1024 * 1024, 1000)
    }
}

pub struct SharedAnalysisCache {
    inner: Arc<RwLock<AnalysisCache>>,
}

impl SharedAnalysisCache {
    pub fn new(max_size_bytes: usize, max_entries: usize) -> Self {
        Self {
            inner: Arc::new(RwLock::new(AnalysisCache::new(max_size_bytes, max_entries))),
        }
    }

    pub fn get_or_compute<T, F>(&self, key: CacheKey, compute: F) -> Arc<T>
    where
        T: Send + Sync + 'static,
        F: FnOnce() -> T,
    {
        {
            let mut cache = self.inner.write().unwrap();
            if let Some(value) = cache.get::<T>(&key) {
                return value;
            }
        }

        let mut cache = self.inner.write().unwrap();
        cache.get_or_compute(key, compute)
    }
}

impl Default for SharedAnalysisCache {
    fn default() -> Self {
        Self::new(100 * 1024 * 1024, 1000)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cache_basic_operations() {
        let mut cache = AnalysisCache::new(1024, 10);

        let key = CacheKey::new::<String>("test", 0);
        let value = cache.get_or_compute(key.clone(), || "hello".to_string());

        assert_eq!(*value, "hello");
        assert_eq!(cache.statistics().hits, 0);
        assert_eq!(cache.statistics().misses, 1);

        let value2 = cache.get::<String>(&key).unwrap();
        assert_eq!(*value2, "hello");
        assert_eq!(cache.statistics().hits, 1);
    }

    #[test]
    fn test_cache_eviction() {
        let mut cache = AnalysisCache::new(1024, 2);

        let key1 = CacheKey::new::<i32>("test1", 0);
        let key2 = CacheKey::new::<i32>("test2", 0);
        let key3 = CacheKey::new::<i32>("test3", 0);

        cache.insert(key1.clone(), 1);
        cache.insert(key2.clone(), 2);
        cache.insert(key3.clone(), 3);

        assert!(cache.get::<i32>(&key1).is_none());
        assert_eq!(cache.statistics().evictions, 1);
    }

    #[test]
    fn test_cache_invalidation() {
        let mut cache = AnalysisCache::new(1024, 10);

        let key1 = CacheKey::new::<String>("target1", 0);
        let key2 = CacheKey::new::<String>("target2", 0);

        cache.insert(key1.clone(), "value1".to_string());
        cache.insert(key2.clone(), "value2".to_string());

        cache.invalidate_target("target1");

        assert!(cache.get::<String>(&key1).is_none());
        assert!(cache.get::<String>(&key2).is_some());
        assert_eq!(cache.statistics().invalidations, 1);
    }
}
