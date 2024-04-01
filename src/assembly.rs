//! Intel Ivy Bridge introduced the ERMS feature indicating that `rep movs` instructions
//! on larger blocks of data are similarly fast as custom copy loops using simd registers.
//!
//! Intel Icelake added the FSRM (fast short rep movs) feature which indicates
//! that rep movs instructions are fast for short sizes (1-128 bytes).
//!
//! Intel Golden Cove extended this also to zero length inputs. On earlier architectures
//! the caller should check the length before calling this function.
//!
//! Intel Raptor Cove also added the Fast Short REP CMPSB and SCASB feature.

// Note: No need to clear the direction flag, the x86 abi requires that it is cleared on function entry and exit.
// https://stackoverflow.com/questions/41090297/default-state-of-direction-flag-df-during-x86-program-execution

use crate::RegisterType;

/// Copy `len` elements from `src` to `dst`.
///
/// On x86_64 this implementation will use inline `rep movs` instructions.
///
/// On other architectures this will fall back to `copy_nonoverlapping`.
///
/// # Safety:
///
/// The same safety considerations as for [`core::ptr::copy_nonoverlapping`] apply:
///
///  - `src` and `dst` need to be valid for the given `len`
///  - `src` and `dst` memory regions must not overlap
///  - pointers need to be properly aligned
#[inline(always)]
pub unsafe fn rep_movs<T: Copy>(src: *const T, dst: *mut T, len: usize) {
    #[cfg(all(target_arch = "x86_64", not(miri)))]
    {
        use core::arch::asm;

        let size = core::mem::size_of::<T>();
        match size {
            8 => {
                asm!("rep movsq", inout("rcx") len => _, inout("rsi") src => _, inout("rdi") dst => _, options(nostack))
            }
            4 => {
                asm!("rep movsd", inout("rcx") len => _, inout("rsi") src => _, inout("rdi") dst => _, options(nostack))
            }
            2 => {
                asm!("rep movsw", inout("rcx") len => _, inout("rsi") src => _, inout("rdi") dst => _, options(nostack))
            }
            _ => {
                asm!("rep movsb", inout("rcx") len * size => _, inout("rsi") src => _, inout("rdi") dst => _, options(nostack))
            }
        }
    }
    #[cfg(not(all(target_arch = "x86_64", not(miri))))]
    {
        core::ptr::copy_nonoverlapping(src, dst, len)
    }
}

/// Store `len` elements into `dst`.
///
/// On x86_64 this implementation will use inline `rep stos` instructions.
///
/// On other architectures this will fall back to `slice::fill`.
///
/// # Safety:
///
/// The same safety considerations as for [`core::ptr::write`] apply:
///
///  - dst must be valid for writes
///  - dst must be properly aligned
#[inline(always)]
pub unsafe fn rep_stos<T: Copy>(src: T, dst: *mut T, len: usize) {
    #[cfg(all(target_arch = "x86_64", not(miri)))]
    {
        use core::arch::asm;

        let size = core::mem::size_of::<T>();
        match size {
            8 => {
                let src: u64 = core::mem::transmute_copy(&src);
                asm!("rep stosq", inout("rcx") len => _, in("rax") src, inout("rdi") dst => _, options(nostack))
            }
            4 => {
                let src: u32 = core::mem::transmute_copy(&src);
                asm!("rep stosd", inout("rcx") len => _, in("eax") src, inout("rdi") dst => _, options(nostack))
            }
            2 => {
                let src: u16 = core::mem::transmute_copy(&src);
                asm!("rep stosw", inout("rcx") len => _, in("ax") src, inout("rdi") dst => _, options(nostack))
            }
            _ => {
                let src: u8 = core::mem::transmute_copy(&src);
                asm!("rep stosb", inout("rcx") len * size => _, in("al") src, inout("rdi") dst => _, options(nostack))
            }
        }
    }
    #[cfg(not(all(target_arch = "x86_64", not(miri))))]
    {
        core::slice::from_raw_parts_mut(dst, len).fill(src)
    }
}

/// Return the index of the first mismatching element between `a` and `b`.
///
/// On x86_64 this implementation will use inline `rep cmps` instructions.
///
/// On other architectures this will fall back to `slice::iter::position`.
///
/// # Safety:
///
/// The same safety considerations as for [`core::ptr::write`] apply:
///
///  - `a` and `b` need to be valid for the given `len`
///  - pointers need to be properly aligned
#[inline(always)]
pub unsafe fn rep_cmps<T: RegisterType>(a: *const T, b: *const T, len: usize) -> Option<usize> {
    #[cfg(all(target_arch = "x86_64", not(miri)))]
    {
        use core::arch::asm;

        let size = core::mem::size_of::<T>();
        let mut eq: u8;
        let mut p: *const T;
        match size {
            8 => {
                asm!(
                "test rcx, rcx",
                "repe cmpsq",
                "sete {eq}",
                inout("rcx") len => _, inout("rdi") a => p, inout("rsi") b => _, eq = lateout(reg_byte) eq,
                options(nostack, readonly)
                );
            }
            4 => {
                asm! {
                "test rcx, rcx",
                "repe cmpsd",
                "sete {eq}",
                inout("rcx") len => _, inout("rdi") a => p, inout("rsi") b => _, eq = lateout(reg_byte) eq,
                options(nostack, readonly)
                };
            }
            2 => {
                asm!(
                "test rcx, rcx",
                "repe cmpsw",
                "sete {eq}",
                inout("rcx") len => _, inout("rdi") a => p, inout("rsi") b => _, eq = lateout(reg_byte) eq,
                options(nostack, readonly)
                );
            }
            _ => {
                asm!(
                "test rcx, rcx",
                "repe cmpsb",
                "sete {eq}",
                inout("rcx") len => _, inout("rdi") a => p, inout("rsi") b => _, eq = lateout(reg_byte) eq,
                options(nostack, readonly)
                );
            }
        }
        if (eq & 0b1) == 0 {
            Some(p.offset_from(a) as usize - 1)
        } else {
            None
        }
    }
    #[cfg(not(all(target_arch = "x86_64", not(miri))))]
    {
        core::slice::from_raw_parts(a, len)
            .iter()
            .zip(core::slice::from_raw_parts(b, len))
            .position(|(a, b)| !a.bitwise_eq(b))
    }
}

/// Return the index of the first occurrence of `valule` in `src`.
///
/// On x86_64 this implementation will use inline `rep scas` instructions.
///
/// On other architectures this will fall back to `slice::iter::position`.
///
/// # Safety:
///
/// The same safety considerations as for [`core::ptr::write`] apply:
///
///  - `src` needs to be valid for the given `len`
///  - pointers need to be properly aligned
#[inline(always)]
pub unsafe fn rep_scas<T: RegisterType>(src: *const T, value: T, len: usize) -> Option<usize> {
    #[cfg(all(target_arch = "x86_64", not(miri)))]
    {
        use core::arch::asm;

        let size = core::mem::size_of::<T>();
        let mut eq: u8;
        let mut p: *const T;
        match size {
            8 => {
                let value: u64 = core::mem::transmute_copy(&value);
                asm!(
                "test rdi, rdi # clear ZF",
                "repne scasq",
                "sete {eq}",
                in("rax") value, inout("rcx") len => _, inout("rdi") src => p, eq = lateout(reg_byte) eq,
                options(nostack, readonly)
                );
            }
            4 => {
                let value: u32 = core::mem::transmute_copy(&value);
                asm! {
                "test rdi, rdi # clear ZF",
                "repne scasd",
                "sete {eq}",
                in("eax") value, inout("rcx") len => _, inout("rdi") src => p, eq = lateout(reg_byte) eq,
                options(nostack, readonly)
                };
            }
            2 => {
                let value: u16 = core::mem::transmute_copy(&value);
                asm!(
                "test rdi, rdi # clear ZF",
                "repne scasw",
                "sete {eq}",
                in("ax") value, inout("rcx") len => _, inout("rdi") src => p, eq = lateout(reg_byte) eq,
                options(nostack, readonly)
                );
            }
            _ => {
                let value: u8 = core::mem::transmute_copy(&value);
                asm!(
                "test rdi, rdi # clear ZF",
                "repne scasb",
                "sete {eq}",
                in("al") value, inout("rcx") len => _, inout("rdi") src => p, eq = lateout(reg_byte) eq,
                options(nostack, readonly)
                );
            }
        }
        if (eq & 0b1) != 0 {
            Some(p.offset_from(src) as usize - 1)
        } else {
            None
        }
    }
    #[cfg(not(all(target_arch = "x86_64", not(miri))))]
    {
        core::slice::from_raw_parts(src, len)
            .iter()
            .position(|a| a.bitwise_eq(&value))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rep_movsb() {
        let input = [1_u8, 2, 3, 4, 5];
        let mut output = [0_u8; 5];
        unsafe {
            rep_movs(input.as_ptr(), output.as_mut_ptr(), output.len());
        }
        assert_eq!(&output, &input)
    }

    #[test]
    fn test_rep_movsw() {
        let input = [1_i16, 2, 3, 4, 5, 6, 7];
        let mut output = [0_i16; 7];
        unsafe {
            rep_movs(input.as_ptr(), output.as_mut_ptr(), output.len());
        }
        assert_eq!(&output, &input)
    }

    #[test]
    fn test_rep_movsd() {
        let input = [1_i32, 2, 3];
        let mut output = [0_i32; 3];
        unsafe {
            rep_movs(input.as_ptr(), output.as_mut_ptr(), output.len());
        }
        assert_eq!(&output, &input)
    }

    #[test]
    fn test_rep_movsq() {
        let input = [1_f64, 2_f64, 3_f64];
        let mut output = [0_f64; 3];
        unsafe {
            rep_movs(input.as_ptr(), output.as_mut_ptr(), output.len());
        }
        assert_eq!(&output, &input)
    }

    #[test]
    fn test_rep_stosb() {
        let mut output = [0; 5];
        unsafe {
            rep_stos(42_u8, output.as_mut_ptr(), output.len());
        }
        assert_eq!(&output, &[42; 5])
    }

    #[test]
    fn test_rep_stosw() {
        let mut output = [0; 7];
        unsafe {
            rep_stos(42_i16, output.as_mut_ptr(), output.len());
        }
        assert_eq!(&output, &[42; 7])
    }

    #[test]
    fn test_rep_stosd() {
        let mut output = [0; 6];
        unsafe {
            rep_stos(42_i32, output.as_mut_ptr(), output.len());
        }
        assert_eq!(&output, &[42; 6])
    }

    #[test]
    fn test_rep_stosq() {
        let mut output = [0; 5];
        unsafe {
            rep_stos(42_i64, output.as_mut_ptr(), output.len());
        }
        assert_eq!(&output, &[42; 5])
    }

    #[test]
    fn test_rep_cmpsb() {
        unsafe {
            assert_eq!(rep_cmps::<u8>([].as_ptr(), [].as_ptr(), 0), None);
            assert_eq!(rep_cmps::<u8>([1].as_ptr(), [2].as_ptr(), 1), Some(0));
            assert_eq!(
                rep_cmps::<u8>([1_u8, 2].as_ptr(), [1_u8, 3].as_ptr(), 2),
                Some(1)
            );
            assert_eq!(
                rep_cmps::<u8>([1_u8, 2, 3, 4].as_ptr(), [1_u8, 2, 3, 4].as_ptr(), 4),
                None
            );
            assert_eq!(
                rep_cmps::<u8>([1_u8, 2, 3, 4, 5].as_ptr(), [1_u8, 2, 3, 5, 5].as_ptr(), 5),
                Some(3)
            );
        }
    }

    #[test]
    fn test_rep_cmpsw() {
        unsafe {
            assert_eq!(rep_cmps::<u16>([].as_ptr(), [].as_ptr(), 0), None);
            assert_eq!(rep_cmps::<u16>([1].as_ptr(), [2].as_ptr(), 1), Some(0));
            assert_eq!(
                rep_cmps::<u16>([1, 2, 3, 4].as_ptr(), [1, 2, 3, 4].as_ptr(), 4),
                None
            );
            assert_eq!(
                rep_cmps::<u16>([1, 2, 3, 4, 5].as_ptr(), [1, 2, 3, 5, 5].as_ptr(), 5),
                Some(3)
            );
        }
    }

    #[test]
    fn test_rep_cmpsd() {
        unsafe {
            assert_eq!(rep_cmps::<i32>([].as_ptr(), [].as_ptr(), 0), None);
            assert_eq!(rep_cmps::<i32>([1].as_ptr(), [2].as_ptr(), 1), Some(0));
            assert_eq!(
                rep_cmps::<i32>([1, 2, 3, 4].as_ptr(), [1, 2, 3, 4].as_ptr(), 4),
                None
            );
            assert_eq!(
                rep_cmps::<i32>([1, 2, 3, 4, 5].as_ptr(), [1, 2, 3, 5, 5].as_ptr(), 5),
                Some(3)
            );
        }
    }

    #[test]
    fn test_rep_cmpsq() {
        unsafe {
            assert_eq!(rep_cmps::<i64>([].as_ptr(), [].as_ptr(), 0), None);
            assert_eq!(rep_cmps::<i64>([1].as_ptr(), [2].as_ptr(), 1), Some(0));
            assert_eq!(
                rep_cmps::<i64>([1, 2, 3, 4].as_ptr(), [1, 2, 3, 4].as_ptr(), 4),
                None
            );
            assert_eq!(
                rep_cmps::<i64>([1, 2, 3, 4, 5].as_ptr(), [1, 2, 3, 5, 5].as_ptr(), 5),
                Some(3)
            );
        }
    }

    #[test]
    fn test_rep_scasb() {
        unsafe {
            assert_eq!(rep_scas([].as_ptr(), 1_u8, 0), None);
            assert_eq!(rep_scas([1].as_ptr(), 2_u8, 1), None);
            assert_eq!(rep_scas([1].as_ptr(), 1_u8, 1), Some(0));
            assert_eq!(rep_scas([1, 2].as_ptr(), 2_u8, 2), Some(1));
            assert_eq!(rep_scas([1, 2, 2].as_ptr(), 2_u8, 3), Some(1));
            assert_eq!(rep_scas([1, 2, 3].as_ptr(), 2_u8, 3), Some(1));
        }
    }

    #[test]
    fn test_rep_scasw() {
        unsafe {
            assert_eq!(rep_scas([].as_ptr(), 1_u16, 0), None);
            assert_eq!(rep_scas([1].as_ptr(), 2_u16, 1), None);
            assert_eq!(rep_scas([1].as_ptr(), 1_u16, 1), Some(0));
            assert_eq!(rep_scas([1, 2].as_ptr(), 2_u16, 2), Some(1));
            assert_eq!(rep_scas([1, 2, 2].as_ptr(), 2_u16, 3), Some(1));
            assert_eq!(rep_scas([1, 2, 3].as_ptr(), 2_u16, 3), Some(1));
        }
    }

    #[test]
    fn test_rep_scasd() {
        unsafe {
            assert_eq!(rep_scas([].as_ptr(), 1_u32, 0), None);
            assert_eq!(rep_scas([1].as_ptr(), 2_u32, 1), None);
            assert_eq!(rep_scas([1].as_ptr(), 1_u32, 1), Some(0));
            assert_eq!(rep_scas([1, 2].as_ptr(), 2_u32, 2), Some(1));
            assert_eq!(rep_scas([1, 2, 2].as_ptr(), 2_u32, 3), Some(1));
            assert_eq!(rep_scas([1, 2, 3].as_ptr(), 2_u32, 3), Some(1));
        }
    }

    #[test]
    fn test_rep_scasq() {
        unsafe {
            assert_eq!(rep_scas([].as_ptr(), 1_f64, 0), None);
            assert_eq!(rep_scas([1_f64].as_ptr(), 2_f64, 1), None);
            assert_eq!(rep_scas([1_f64].as_ptr(), 1_f64, 1), Some(0));
            assert_eq!(rep_scas([1_f64, 2_f64].as_ptr(), 2_f64, 2), Some(1));
            assert_eq!(rep_scas([1_f64, 2_f64, 2_f64].as_ptr(), 2_f64, 3), Some(1));
            assert_eq!(rep_scas([1_f64, 2_f64, 3_f64].as_ptr(), 2_f64, 3), Some(1));
        }
    }
}
