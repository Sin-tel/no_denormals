![build](https://github.com/Sin-tel/no_denormals/actions/workflows/rust.yml/badge.svg)
# no_denormals
Temporarily turn off floating point denormals.

Internally, this uses a RAII-style guard to manage the state of certain processor flags.
On `x86` and `x86_64`, this sets the flush-to-zero and denormals-are-zero flags in the MXCSR register.
On `aarch64` this sets the flush-to-zero flag in the FPCR register.
In all cases, the register will be reset to its initial state when the guard is dropped.

## Usage

```rust
use no_denormals::no_denormals;

no_denormals(|| {
	// your DSP code here.
});

```
