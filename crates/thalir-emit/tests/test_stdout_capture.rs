use std::io::{self, Write};
use std::sync::{Arc, Mutex};

struct CaptureWriter {
    buffer: Arc<Mutex<Vec<u8>>>,
}

impl CaptureWriter {
    fn new() -> (Self, Arc<Mutex<Vec<u8>>>) {
        let buffer = Arc::new(Mutex::new(Vec::new()));
        (
            Self {
                buffer: buffer.clone(),
            },
            buffer,
        )
    }
}

impl Write for CaptureWriter {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        let mut buffer = self.buffer.lock().unwrap();
        buffer.extend_from_slice(buf);
        Ok(buf.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

#[test]
fn test_direct_stdout_write() {
    println!("This should be captured by test framework");
    print!("This too ");
    println!("should be captured");

    std::io::stdout()
        .write_all(b"Direct stdout write\n")
        .unwrap();
    std::io::stdout().flush().unwrap();
}

#[test]
fn test_custom_writer_capture() {
    let (mut writer, buffer) = CaptureWriter::new();

    writeln!(writer, "Test output line 1").unwrap();
    writeln!(writer, "Test output line 2").unwrap();

    let captured = buffer.lock().unwrap();
    let output = String::from_utf8_lossy(&captured);

    println!("Captured output: '{}'", output);
    assert!(output.contains("Test output line 1"));
    assert!(output.contains("Test output line 2"));
}

#[test]
fn test_vec_buffer_capture() {
    let mut buffer = Vec::new();

    writeln!(&mut buffer, "Line 1").unwrap();
    writeln!(&mut buffer, "Line 2").unwrap();

    let output = String::from_utf8(buffer).unwrap();
    println!("Buffer output: '{}'", output);

    assert!(output.contains("Line 1"));
    assert!(output.contains("Line 2"));
}

#[test]
fn test_println_vs_writeln_stdout() {
    println!("println! output");

    let mut stdout = std::io::stdout();
    writeln!(stdout, "writeln! to stdout").unwrap();

    stdout.write_all(b"write_all to stdout\n").unwrap();
    stdout.flush().unwrap();

    assert!(true);
}
