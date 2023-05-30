// RAII-style guard guard for turning off floating point denormals
//
// previous code also had the following:
//        // All exceptions are masked
//        mxcsr |= ((1 << 6) - 1) << 7;
// but I don't know if this is actually necessary
//
// TODO: What should we do in case neither architectures are supported?
// TODO: should we add the !send !sync hack?
// https://stackoverflow.com/questions/62713667/how-to-implement-send-or-sync-for-a-type

// FTZ and DAZ
#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
const X86_MASK: u32 = 0x8040;

// FTZ
#[cfg(target_arch = "aarch64")]
const AARCH64_MASK: u64 = 1 << 24;

pub struct DenormalGuard {
	#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
	mxcsr: u32,
	#[cfg(target_arch = "aarch64")]
	fpcr: u64,
}

impl DenormalGuard {
	fn new() -> Self {
		#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
		{
			#[cfg(all(target_arch = "x86_64", target_feature = "sse"))]
			use std::arch::x86_64::{_mm_getcsr, _mm_setcsr};

			#[cfg(all(target_arch = "x86", target_feature = "sse"))]
			use std::arch::x86::{_mm_getcsr, _mm_setcsr};

			let mxcsr = unsafe { _mm_getcsr() };
			unsafe { _mm_setcsr(mxcsr | X86_MASK) };

			DenormalGuard { mxcsr }
		}
		#[cfg(target_arch = "aarch64")]
		{
			let mut fpcr: u64;
			unsafe { std::arch::asm!("mrs {}, fpcr", out(reg) fpcr) };
			unsafe { std::arch::asm!("msr fpcr, {}", in(reg) fpcr | AARCH64_MASK) };

			DenormalGuard { fpcr }
		}
	}
}

impl Drop for DenormalGuard {
	fn drop(&mut self) {
		#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
		{
			#[cfg(all(target_arch = "x86_64", target_feature = "sse"))]
			use std::arch::x86_64::_mm_setcsr;

			#[cfg(all(target_arch = "x86", target_feature = "sse"))]
			use std::arch::x86::_mm_setcsr;

			unsafe { _mm_setcsr(self.mxcsr) };
		}
		#[cfg(target_arch = "aarch64")]
		{
			unsafe { std::arch::asm!("msr fpcr, {}", in(reg) self.fpcr) }
		};
	}
}

pub fn no_denormals<T, F: FnOnce() -> T>(func: F) -> T {
	let guard = DenormalGuard::new();
	let ret = func();
	std::mem::drop(guard);

	return ret;
}

#[cfg(test)]
mod tests {
	use crate::no_denormals;

	#[test]
	fn test_positive() {
		let small: f32 = f32::MIN_POSITIVE;
		{
			let smaller = small * 0.5;
			assert!(smaller.is_subnormal());
		}
		no_denormals(|| {
			let smaller = small * 0.5;
			assert!(!smaller.is_subnormal());
		});
		{
			let smaller = small * 0.5;
			assert!(smaller.is_subnormal());
		};
	}

	#[test]
	fn test_negative() {
		let small: f32 = -f32::MIN_POSITIVE;
		{
			let smaller = small * 0.5;
			assert!(smaller.is_subnormal());
		}
		no_denormals(|| {
			let smaller = small * 0.5;
			assert!(!smaller.is_subnormal());
		});
		{
			let smaller = small * 0.5;
			assert!(smaller.is_subnormal());
		};
	}
}
