# no_denormals
[![build](https://github.com/Sin-tel/no_denormals/actions/workflows/rust.yml/badge.svg)](https://github.com/Sin-tel/no_denormals/actions/workflows/rust.yml)
[![Crates.io](https://img.shields.io/crates/v/no_denormals.svg)](https://crates.io/crates/no_denormals) 
[![MIT licensed](https://img.shields.io/badge/license-MIT-blue.svg)](./LICENSE)

Temporarily turn off floating point denormals.

Internally, this uses a RAII-style guard to manage the state of certain processor flags.
On `x86` and `x86_64`, this sets the flush-to-zero and denormals-are-zero flags in the MXCSR register.
On `aarch64` this sets the flush-to-zero flag in the FPCR register.
In all cases, the register will be reset to its initial state when the guard is dropped.

Note that according to the Rust docs "modifying the masking flags, rounding mode, or denormals-are-zero mode flags leads to immediate Undefined Behavior: Rust assumes that these are always in their default state and will optimize accordingly."
So use this at your own risk.

## Usage

```rust
use no_denormals::no_denormals;

no_denormals(|| {
	// your DSP code here.
});

```
