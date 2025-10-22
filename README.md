# ThalIR

An intermediate representation for smart contract security auditing. Extends Cranelift IR with EVM-specific semantics while preserving Cranelift's SSA form and design principles.

Tameshi ([tameshi.dev](https://tameshi.dev)) is the reference auditing platform using ThalIR.

> **Note on Syntax**: This document uses `ext.sol.*` notation to describe ThalIR's conceptual extension namespace for EVM operations. The actual grammar implementation uses shorter opcodes like `storage_load`, `mapping_load`, `get_context`, etc. Code examples show the implemented syntax.

---

## Why ThalIR Exists

Source-level Solidity presents challenges for systematic security analysis:

- **Syntax obscures patterns**: The same vulnerability can appear in multiple syntactic forms
- **Implicit behavior**: Overflow checks, storage layouts, and call semantics require inference
- **Mixed concerns**: Business logic and security-critical operations are interleaved

ThalIR provides a canonical, explicit representation optimized for vulnerability detection rather than code generation.

---

## Design Goals

**Pattern Recognition**

Represent security-relevant operation sequences explicitly (storage writes after external calls, unchecked arithmetic, access control checks).

**Explicit Semantics**

Storage operations, external calls, and overflow behavior are first-class IR concepts rather than implicit.

**Canonical Form**

SSA with block parameters and multi-value returns provides uniform representation regardless of source-level syntax variations.

**Obfuscation Support**

Name obfuscation and metadata stripping for confidential auditing while preserving security-relevant behavior.

---

## What ThalIR Adds to Cranelift

ThalIR is a superset of Cranelift IR with the following additions:

### Smart Contract Operations

**Storage operations:**
- `storage_load %key` - read from persistent storage
- `storage_store %key, %val` - write to persistent storage
- `mapping_load %mapping, %key` - load from mapping
- `mapping_store %mapping, %key, %val` - store to mapping

**External calls:**
- `call` - external call with explicit reentrancy points
- `staticcall` - call that can't modify state
- `delegatecall` - call using caller's storage context

**Events:**
- `log` operations for event emission

**ABI encoding:**
- `abi.encode` - encode arguments
- `abi.decode` - decode return data

**Environment access:**
- `get_context msg.sender` - transaction sender
- `get_context msg.value` - ether value
- `get_context block.timestamp` - block timestamp
- `get_context block.number` - block number

### Overflow Semantics

Explicit overflow behavior:

**Checked arithmetic** (Solidity ≥0.8):
- `add.trap`, `sub.trap`, `mul.trap` - trap on overflow

**Unchecked arithmetic** (Solidity unchecked blocks):
- `add.wrap`, `sub.wrap`, `mul.wrap` - wrap silently

**No-overflow assertions:**
- `add.nsw`, `add.nuw` - poison on signed/unsigned overflow

### Memory Regions

Address spaces for EVM memory regions:

- **ptr[as=0]** - transient memory (mutable, per-call)
- **ptr[as=1]** - calldata (read-only, immutable)
- **ptr[as=2]** - code (read-only, contract bytecode)

Address spaces prevent accidental aliasing and enable precise memory analysis.

### Obfuscation

- Deterministic name hashing with optional salt
- Bidirectional mapping files
- Configurable retention levels

---

## Example

Solidity:

```solidity
function transfer(address to, uint256 amount) public {
    balances[msg.sender] -= amount;
    balances[to] += amount;
}
```

ThalIR:

```
function %transfer(i160, i256) public {
block0(v0: i160, v1: i256):
    v2 = get_context msg.sender
    v3 = mapping_load balances, v2
    v4 = isub.i256 v3, v1
    mapping_store balances, v2, v4
    v5 = mapping_load balances, v0
    v6 = iadd.i256 v5, v1
    mapping_store balances, v0, v6
    return
}
```

The representation makes control flow and data dependencies explicit through SSA values and block parameters.

---

## Common Vulnerability Patterns

ThalIR's explicit representation enables detection of the following vulnerability patterns:

### Reentrancy

Look for external calls followed by state writes:

```
%ok = call_ext %target, %value
; risky: state update after external call
mapping_store balances, %key, %new_balance
```

Classic reentrancy: the called contract can reenter before the balance update, seeing stale state.

### Unchecked Arithmetic

Scan for `.wrap` variants in security-critical paths:

```
%new_bal = sub.wrap %balance, %amount : i256  ; DANGER: can underflow!
storage_store %bal_key, %new_bal
```

Solidity ≥0.8 defaults to checked arithmetic, but `unchecked {}` blocks use `.wrap` and can silently underflow.

### Access Control Bypass

Missing guard checks before privileged operations:

```
function %withdraw(i256) public {
block0(v0: i256):
    ; missing: icmp sender, owner
    ; missing: brif check, block_allowed, block_revert
    v1 = storage_load slot0
    v2 = call_ext v1, v0
    return
}
```

No sender check means anyone can call `withdraw`. In ThalIR, the absence of `icmp` + `brif` before the call is a red flag.

### Storage Collision

Incorrect storage key derivation can alias different variables:

```
; Two mappings using same slot → collision!
%key1 = mapping_key 0, %addr1
%key2 = mapping_key 0, %addr2
```

If both mappings use slot 0, they'll overwrite each other. ThalIR makes slots explicit so checkers can detect this.

---

## Types

ThalIR uses Cranelift's type system with Solidity-specific conventions:

### Integers

- `i1, i8, i16, i32, i64, i128, i256` - two's complement integers
- EVM word = `i256`
- `address` = `i160` (zero-extended to `i256` when needed)
- `bytesN` = integer of width `8*N` bits

### Booleans

- `bool` - logical true/false (distinct from `i1` for clarity)

### Pointers

- `ptr` or `ptr[as=0]` - default pointer (transient memory)
- `ptr[as=1]` - calldata pointer
- `ptr[as=2]` - code pointer

Address space `N` distinguishes memory regions for alias analysis.

---

## Instruction Categories

### Integer Arithmetic

```
%z = add.wrap %x, %y : i256    ; wrapping add (unchecked)
%z = add.trap %x, %y : i256    ; trapping add (checked)
%z = sub.trap %x, %y : i256    ; checked subtraction
%z = mul.wrap %x, %y : i256    ; unchecked multiplication
```

### Comparisons

```
%b = icmp.eq %x, %y : i256 -> bool
%b = icmp.slt %x, %y : i256 -> bool   ; signed less-than
%b = icmp.ult %x, %y : i256 -> bool   ; unsigned less-than
```

Variants: `eq, ne, ult, ule, ugt, uge, slt, sle, sgt, sge`.

### Control Flow

```
br block1(%arg1, %arg2)
br_if %cond, then:block1(%arg1), else:block2(%arg2)
ret %v0, %v1, ...
call @function(%arg1, %arg2)
trap [reason="assertion failed"]
```

### Storage Operations

```
%v = storage_load %key
storage_store %key, %val
%v = mapping_load %mapping, %key
mapping_store %mapping, %key, %val
```

### External Calls

```
%ret = call_ext %addr, %value
%ret = staticcall %addr
%ret = delegatecall %addr
```

---

## Validation Rules

ThalIR programs must satisfy:

**Type correctness**: Operand and result types match instruction signatures. Block parameters match branch arguments.

**SSA dominance**: Every value is defined exactly once before use. Uses must be dominated by definitions.

**Control flow**: Every block ends with a terminator (`br`, `br_if`, `ret`, `trap`). No fall-through.

**Memory**: Load/store types must be sized. Alignment must be a power of two. Address space consistency enforced.

---

## Examples

### Loop with block parameters

```
function %sum(i32) -> i32 {
block0(v0: i32):
    v1 = iconst.i32 0
    v2 = iconst.i32 1
    jump block1(v1, v2)

block1(v3: i32, v4: i32):
    v5 = icmp slt v4, v0
    brif v5, block2(v3, v4), block3(v3)

block2(v6: i32, v7: i32):
    v8 = iadd v6, v7
    v9 = iadd v7, 1
    jump block1(v8, v9)

block3(v10: i32):
    return v10
}
```

### External call with reentrancy risk

```
function %withdraw(i256) public {
block0(v0: i256):
    v1 = get_context msg.sender
    v2 = mapping_load balances, v1
    v3 = isub.i256 v2, v0

    ; DANGEROUS: external call before state update!
    v4 = call_ext v1, v0

    ; Reentrancy window: sender can call back before this line
    mapping_store balances, v1, v3
    return
}
```

Auditor's note: Move the `mapping_store` before the `call_ext` to prevent reentrancy.

---

## Components

- **thalir-core** - IR data structures, types, instruction builders
- **thalir-emit** - IR formatters
- **thalir-parser** - Text format parser
- **thalir-transform** - Solidity → ThalIR transformation
- **thalir** - Unified crate

---

## Usage

```rust
use thalir::{transform_solidity_to_ir, ThalIREmitter};

let solidity = "contract Token { ... }";
let contracts = transform_solidity_to_ir(solidity)?;

let emitter = ThalIREmitter::new(contracts);
let ir_text = emitter.emit_to_string(false);
println!("{}", ir_text);
```

### Obfuscation

```rust
use thalir::obfuscation::{ObfuscationPass, ObfuscationConfig};

let config = ObfuscationConfig::standard();
let mapping = ObfuscationPass::new(config)
    .obfuscate_contract(&mut contract)?;

mapping.save("mapping.json")?;
```

This transforms:
```
contract MyToken {
  function transfer(address to, uint256 amount) { ... }
}
```

Into:
```
contract c_a1b2c3 {
  function f_d4e5f6(address p0, uint256 p1) { ... }
}
```

The mapping file lets you convert findings back to original names:
```json
{
  "c_a1b2c3": "MyToken",
  "f_d4e5f6": "transfer"
}
```

Auditors can analyze the obfuscated IR without seeing proprietary logic, then report vulnerabilities using deobfuscated names.

---

## Comparison with Cranelift

| Feature | Cranelift IR | ThalIR |
|---------|--------------|--------|
| SSA with block params | ✓ | ✓ |
| Multi-value returns | ✓ | ✓ |
| Explicit traps | ✓ | ✓ (enhanced) |
| Memory model | C11 atomics | C11 atomics |
| Storage operations | ✗ (use memory) | ✓ |
| External calls | ✗ (use indirect call) | ✓ |
| Overflow semantics | wrap/trap | wrap/trap/nsw/nuw |
| Address spaces | ✓ (basic) | ✓ (EVM-specific) |
| Obfuscation | ✗ | ✓ (built-in) |
| Auditing focus | ✗ (compiler focus) | ✓ |

---

## License

Mozilla Public License 2.0 (MPL-2.0)

Copyright (c) 2025 Gianluca Brigandi
