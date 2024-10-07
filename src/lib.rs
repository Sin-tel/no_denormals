//! Temporarily turn off floating point denormals.
//!
//! Internally, this uses a RAII-style guard to manage the state of certain processor flags.
//! On x86 and x86_64, this sets the flush-to-zero and denormals-are-zero flags in the MXCSR register.
//! On aarch64 this sets the flush-to-zero flag in the FPCR register.
//! In all cases, the register will be reset to its initial state when the guard is dropped.
//!
//! ## Usage
//!
//! ```rust
//! use no_denormals::no_denormals;
//!
//! no_denormals(|| {
//!     // your DSP code here.
//! });
//!
//! ```

#![warn(missing_docs, missing_debug_implementations, rust_2018_idioms)]

use core::marker::PhantomData;
use std::arch::asm;

#[cfg(not(any(target_arch = "x86", target_arch = "x86_64", target_arch = "aarch64")))]
compile_error!("This crate only supports x86, x86_64 and aarch64.");

// FTZ and DAZ
#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
const X86_MASK: u32 = 0x8040;

// FTZ
#[cfg(target_arch = "aarch64")]
const AARCH64_MASK: u64 = 1 << 24;

struct DenormalGuard {
	#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
	mxcsr: u32,
	#[cfg(target_arch = "aarch64")]
	fpcr: u64,

	// These processor flags are local to each thread.
	// We implement !Send and !Sync with this workaround,
	// because negative trait bounds are not yet supported.
	// https://users.rust-lang.org/t/negative-trait-bounds-are-not-yet-fully-implemented-use-marker-types-for-now/64495/2
	_not_send_sync: PhantomData<*const ()>,
}

fn set_csr(control: u32) {
	unsafe {
		asm!("ldmxcsr [{}]", in(reg) &control);
	}
}

fn get_csr() -> u32 {
	let control: u32;
	unsafe {
		asm!("stmxcsr [{tmp}]",
            "mov {x:e}, [{tmp}]",
            x = out(reg) control,
            tmp = out(reg) _)
	}
	control
}

impl DenormalGuard {
	fn new() -> Self {
		#[cfg(all(
			any(target_arch = "x86_64", target_arch = "x86"),
			target_feature = "sse"
		))]
		{
			let mxcsr = get_csr();
			set_csr(mxcsr | X86_MASK);

			DenormalGuard {
				mxcsr,
				_not_send_sync: PhantomData,
			}
		}
		#[cfg(target_arch = "aarch64")]
		{
			let mut fpcr: u64;
			unsafe { asm!("mrs {}, fpcr", out(reg) fpcr) };
			unsafe { asm!("msr fpcr, {}", in(reg) fpcr | AARCH64_MASK) };

			DenormalGuard {
				fpcr,
				_not_send_sync: PhantomData,
			}
		}
	}
}

impl Drop for DenormalGuard {
	fn drop(&mut self) {
		#[cfg(all(
			any(target_arch = "x86_64", target_arch = "x86"),
			target_feature = "sse"
		))]
		{
			set_csr(self.mxcsr);
		}

		#[cfg(target_arch = "aarch64")]
		{
			unsafe { asm!("msr fpcr, {}", in(reg) self.fpcr) }
		};
	}
}

/// Calls the `func` closure.
#[inline]
pub fn no_denormals<T, F: FnOnce() -> T>(func: F) -> T {
	let guard = DenormalGuard::new();
	let ret = func();
	std::mem::drop(guard);

	ret
}

#[cfg(test)]
mod tests {
	use crate::no_denormals;
	use std::num::FpCategory;

	fn half(x: f32) -> f32 {
		std::hint::black_box(x) * 0.5
	}

	// #[test]
	// fn arch() {
	// 	println!("Architecture: {:?}", std::env::consts::ARCH);
	// }

	#[test]
	fn test_positive() {
		let small: f32 = f32::MIN_POSITIVE;
		{
			let smaller = half(small);
			assert_eq!(smaller.classify(), FpCategory::Subnormal);
		}
		no_denormals(|| {
			let smaller = half(small);
			assert_eq!(smaller.classify(), FpCategory::Zero);
		});
		{
			let smaller = half(small);
			assert_eq!(smaller.classify(), FpCategory::Subnormal);
		};
	}

	#[test]
	fn test_negative() {
		let small: f32 = -f32::MIN_POSITIVE;
		{
			let smaller = half(small);
			assert_eq!(smaller.classify(), FpCategory::Subnormal);
		}
		no_denormals(|| {
			let smaller = half(small);
			assert_eq!(smaller.classify(), FpCategory::Zero);
		});
		{
			let smaller = half(small);
			assert_eq!(smaller.classify(), FpCategory::Subnormal);
		};
	}
}
