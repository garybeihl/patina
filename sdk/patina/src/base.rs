//! UEFI Base Definitions
//!
//! Basic definitions for UEFI development.
//!
//! ## License
//!
//! Copyright (C) Microsoft Corporation.
//!
//! SPDX-License-Identifier: Apache-2.0

use num_traits;
use r_efi::efi;

use crate::error::EfiError;

pub mod guid;
pub mod memory_map;

/// EFI memory allocation functions work in units of EFI_PAGEs that are 4KB.
/// This should in no way be confused with the page size of the processor.
/// An EFI_PAGE is just the quanta of memory in EFI.
pub const UEFI_PAGE_SIZE: usize = 0x1000;

/// The mask to apply to an address to get the page offset in UEFI.
pub const UEFI_PAGE_MASK: usize = UEFI_PAGE_SIZE - 1;

/// The shift to apply to an address to get the page frame number in UEFI.
pub const UEFI_PAGE_SHIFT: usize = 12;

/// 1KB, 1024 bytes, 0x400, 2^10
pub const SIZE_1KB: usize = 0x400;

/// 2KB, 2048 bytes, 0x800, 2^11
pub const SIZE_2KB: usize = 0x800;

/// 4KB, 4096 bytes, 0x1000, 2^12
pub const SIZE_4KB: usize = 0x1000;

/// 8KB, 8192 bytes, 0x2000, 2^13
pub const SIZE_8KB: usize = 0x2000;

/// 16KB, 16384 bytes, 0x4000, 2^14
pub const SIZE_16KB: usize = 0x4000;

/// 32KB, 32768 bytes, 0x8000, 2^15
pub const SIZE_32KB: usize = 0x8000;

/// 64KB, 65536 bytes, 0x10000, 2^16
pub const SIZE_64KB: usize = 0x10000;

/// 128KB, 0x20000, 2^17
pub const SIZE_128KB: usize = 0x20000;

/// 256KB, 0x40000, 2^18
pub const SIZE_256KB: usize = 0x40000;

/// 512KB, 0x80000, 2^19
pub const SIZE_512KB: usize = 0x80000;

/// 1MB, 0x100000, 2^20
pub const SIZE_1MB: usize = 0x100000;

/// 2MB, 0x200000, 2^21
pub const SIZE_2MB: usize = 0x200000;

/// 4MB, 0x400000, 2^22
pub const SIZE_4MB: usize = 0x400000;

/// 8MB, 0x800000, 2^23
pub const SIZE_8MB: usize = 0x800000;

/// 16MB, 0x1000000, 2^24
pub const SIZE_16MB: usize = 0x1000000;

/// 32MB, 0x2000000, 2^25
pub const SIZE_32MB: usize = 0x2000000;

/// 64MB, 0x4000000, 2^26
pub const SIZE_64MB: usize = 0x4000000;

/// 128MB, 0x8000000, 2^27
pub const SIZE_128MB: usize = 0x8000000;

/// 256MB, 0x10000000, 2^28
pub const SIZE_256MB: usize = 0x10000000;

/// 512MB, 0x20000000, 2^29
pub const SIZE_512MB: usize = 0x20000000;

/// 1GB, 0x40000000, 2^30
pub const SIZE_1GB: usize = 0x40000000;

/// 2GB, 0x80000000, 2^31
pub const SIZE_2GB: usize = 0x80000000;

/// 4GB, 0x100000000, 2^32
pub const SIZE_4GB: usize = 0x100000000;

/// 8GB, 0x200000000, 2^33
pub const SIZE_8GB: usize = 0x200000000;

/// 16GB, 0x400000000, 2^34
pub const SIZE_16GB: usize = 0x400000000;

/// 32GB, 0x800000000, 2^35
pub const SIZE_32GB: usize = 0x800000000;

/// 64GB, 0x1000000000, 2^36
pub const SIZE_64GB: usize = 0x1000000000;

/// 128GB, 0x2000000000, 2^37
pub const SIZE_128GB: usize = 0x2000000000;

/// 256GB, 0x4000000000, 2^38
pub const SIZE_256GB: usize = 0x4000000000;

/// 512GB, 0x8000000000, 2^39
pub const SIZE_512GB: usize = 0x8000000000;

/// 1TB, 0x10000000000, 2^40
pub const SIZE_1TB: usize = 0x10000000000;

/// 2TB, 0x20000000000, 2^41
pub const SIZE_2TB: usize = 0x20000000000;

/// 4TB, 0x40000000000, 2^42
pub const SIZE_4TB: usize = 0x40000000000;

/// 8TB, 0x80000000000, 2^43
pub const SIZE_8TB: usize = 0x80000000000;

/// 16TB, 0x100000000000, 2^44
pub const SIZE_16TB: usize = 0x100000000000;

/// 32TB, 0x200000000000, 2^45
pub const SIZE_32TB: usize = 0x200000000000;

/// 64TB, 0x400000000000, 2^46
pub const SIZE_64TB: usize = 0x400000000000;

/// 128TB, 0x800000000000, 2^47
pub const SIZE_128TB: usize = 0x800000000000;

/// 256TB, 0x1000000000000, 2^48
pub const SIZE_256TB: usize = 0x1000000000000;

/// Patina uses write back as the default cache attribute for memory allocations.
pub const DEFAULT_CACHE_ATTR: u64 = efi::MEMORY_WB;

/// A macro to generate a bit mask with the nth bit set.
///
/// This macro should generally be used to simplify bit references in
/// in masking operations where bit position is significant.
#[macro_export]
macro_rules! bit {
    ($n:expr) => {
        1 << $n
    };
}

/// Checks if the given value is a power of two.
/// This function checks if the value `x` is greater than zero and if it is a power of two.
/// # Parameters
/// - `x`: The value to check.
/// # Returns
/// - `true`: If `x` is a power of two.
/// - `false`: If `x` is not a power of two.
#[inline]
pub fn is_power_of_two<T>(x: T) -> bool
where
    T: num_traits::PrimInt,
{
    x > T::zero() && (x & (x - T::one())) == T::zero()
}

/// Aligns the given address down to the nearest boundary specified by align.
///
/// # Parameters
///
/// - `addr`: The address to be aligned.
/// - `align`: The alignment boundary, which must be a power of two.
///
/// # Returns
///
/// A `Result<T, EfiError>` which is:
/// - `Ok(T)`: The aligned address if `align` is a power of two.
/// - `Err(EfiError)`: An error indicating that `align` must be a power of two.
///
/// # Example
///
/// ```rust
/// use patina::base::align_down;
///
/// let addr: u64 = 1023;
/// let align: u64 = 512;
/// match align_down(addr, align) {
///     Ok(aligned_addr) => {
///         println!("Aligned address: {}", aligned_addr);
///         assert_eq!(aligned_addr, 512);
///     },
///     Err(e) => println!("Error: {:?}", e),
/// }
/// ```
///
/// In this example, the address `1023` is aligned down to `512`.
///
/// # Errors
///
/// The function returns an error if:
/// - `align` is not a power of two.
#[inline]
pub fn align_down<T>(addr: T, align: T) -> Result<T, EfiError>
where
    T: num_traits::PrimInt,
{
    if !is_power_of_two(align) {
        return Err(EfiError::InvalidParameter);
    }
    Ok(addr & !(align - T::one()))
}

/// Aligns the given address up to the nearest boundary specified by align.
///
/// # Parameters
///
/// - `addr`: The address to be aligned.
/// - `align`: The alignment boundary, which must be a power of two.
///
/// # Returns
///
/// A `Result<T, EfiError>` which is:
/// - `Ok(T)`: The aligned address if `align` is a power of two and no overflow occurs.
/// - `Err(EfiError)`: An error indicating the reason for failure (either invalid `align` or overflow).
///
/// # Example
///
/// ```rust
/// use patina::base::align_up;
/// use patina::error::EfiError;
///
/// let addr: u64 = 1025;
/// let align: u64 = 512;
/// match align_up(addr, align) {
///     Ok(aligned_addr) => {
///         println!("Aligned address: {}", aligned_addr);
///         assert_eq!(aligned_addr, 1536);
///     },
///     Err(EfiError::InvalidParameter) => println!("Invalid alignment parameter"),
///     Err(_) => println!("Other alignment error"),
/// }
/// ```
///
/// In this example, the address `1025` is aligned up to `1536`.
///
/// # Errors
///
/// The function returns an error if:
/// - `align` is not a power of two.
/// - An overflow occurs during the alignment process.
#[inline]
pub fn align_up<T>(addr: T, align: T) -> Result<T, EfiError>
where
    T: num_traits::PrimInt,
{
    if !is_power_of_two(align) {
        return Err(EfiError::InvalidParameter);
    }
    let align_mask = align - T::one();
    if addr & align_mask == T::zero() {
        Ok(addr) // already aligned
    } else {
        (addr | align_mask).checked_add(&T::one()).ok_or(EfiError::InvalidParameter)
    }
}

/// Aligns the given address down to the nearest boundary specified by align.
/// Also calculates the aligned length based on the base and length provided.
///
/// # Parameters
/// - `base`: The base address to be aligned.
/// - `length`: The length to be aligned.
/// - `align`: The alignment boundary, which must be a power of two.
///
/// # Returns
/// A `Result<(T, T), EfiError>` which is:
/// - `Ok((T, T))`: A tuple containing the aligned base address and the aligned length.
/// - `Err(EfiError)`: An error indicating that `align` must be a power of two.
///
/// # Example
/// ```rust
/// use patina::base::align_range;
/// let base: u64 = 1023;
/// let length: u64 = 2048;
/// let align: u64 = 512;
/// match align_range(base, length, align) {
///     Ok((aligned_base, aligned_length)) => {
///         println!("Aligned base: {}, Aligned length: {}", aligned_base, aligned_length);
///         assert_eq!(aligned_base, 512);
///         assert_eq!(aligned_length, 2560);
///     },
///     Err(e) => println!("Error: {:?}", e),
/// }
/// ```
///
/// In this example, the base address `1023` is aligned down to `512`, and the length is adjusted accordingly.
/// # Errors
/// The function returns an error if:
/// - `align` is not a power of two.
#[inline]
pub fn align_range<T>(base: T, length: T, align: T) -> Result<(T, T), EfiError>
where
    T: num_traits::PrimInt,
{
    if !is_power_of_two(align) {
        return Err(EfiError::InvalidParameter);
    }

    let aligned_end = align_up(base + length, align)?;
    let aligned_base = align_down(base, align)?;
    let aligned_length = aligned_end - aligned_base;
    Ok((aligned_base, aligned_length))
}

/// Generates a UEFI-style signature from between 1 to 8 bytes, packing them into a u16, u32
/// or u64 as appropriate for the parameters passed.
///
/// # Examples
///
/// ```rust
/// use patina::signature;
/// const SIG: u32 = signature!('A', 'B', 'C', 'D');
/// assert_eq!(SIG, 0x44434241);
/// ```
///
/// # Note
/// This macro is typically used to create signatures for UEFI structures
/// and is the equivalent of the SIGNATURE_16, SIGNATURE_32 and SIGNATURE_64
/// macros from EDK2.
#[allow(unused_macros)]
#[macro_export]
macro_rules! signature {
    ($a:literal) => {
        ($a as u16)
    };
    ($a:literal, $b:literal) => {
        ($a as u16) | (($b as u16) << 8)
    };
    ($a:literal, $b:literal, $c:literal) => {
        ($a as u32) | (($b as u32) << 8) | (($c as u32) << 16)
    };
    ($a:literal, $b:literal, $c:literal, $d:literal) => {
        ($a as u32) | (($b as u32) << 8) | (($c as u32) << 16) | (($d as u32) << 24)
    };
    ($a:literal, $b:literal, $c:literal, $d:literal, $e:literal) => {
        ($a as u64) | (($b as u64) << 8) | (($c as u64) << 16) | (($d as u64) << 24) | (($e as u64) << 32)
    };
    ($a:literal, $b:literal, $c:literal, $d:literal, $e:literal, $f:literal) => {
        ($a as u64)
            | (($b as u64) << 8)
            | (($c as u64) << 16)
            | (($d as u64) << 24)
            | (($e as u64) << 32)
            | (($f as u64) << 40)
    };
    ($a:literal, $b:literal, $c:literal, $d:literal, $e:literal, $f:literal, $g:literal) => {
        ($a as u64)
            | (($b as u64) << 8)
            | (($c as u64) << 16)
            | (($d as u64) << 24)
            | (($e as u64) << 32)
            | (($f as u64) << 40)
            | (($g as u64) << 48)
    };
    ($a:literal, $b:literal, $c:literal, $d:literal, $e:literal, $f:literal, $g:literal, $h:literal) => {
        ($a as u64)
            | (($b as u64) << 8)
            | (($c as u64) << 16)
            | (($d as u64) << 24)
            | (($e as u64) << 32)
            | (($f as u64) << 40)
            | (($g as u64) << 48)
            | (($h as u64) << 56)
    };
}
#[cfg(test)]
#[coverage(off)]
mod tests {
    use super::*;

    #[test]
    fn test_is_power_of_two() {
        assert!(is_power_of_two(1u64));
        assert!(is_power_of_two(2u64));
        assert!(is_power_of_two(4u64));
        assert!(is_power_of_two(1024u64));
        assert!(!is_power_of_two(0u64));
        assert!(!is_power_of_two(3u64));
        assert!(!is_power_of_two(1023u64));
    }

    #[test]
    fn test_align_down() {
        assert_eq!(align_down(1023u64, 512u64).unwrap(), 512u64);
        assert_eq!(align_down(1024u64, 512u64).unwrap(), 1024u64);
        assert_eq!(align_down(0u64, 512u64).unwrap(), 0u64);
        assert_eq!(align_down(513u64, 512u64).unwrap(), 512u64);
        assert_eq!(align_down(0xFFFFu64, 0x1000u64).unwrap(), 0xF000u64);
        assert_eq!(align_down(0x1000u64, 0x1000u64).unwrap(), 0x1000u64);
        assert!(align_down(100u64, 3u64).is_err()); // not power of two
    }

    #[test]
    fn test_align_up() {
        assert_eq!(align_up(1025u64, 512u64).unwrap(), 1536u64);
        assert_eq!(align_up(1024u64, 512u64).unwrap(), 1024u64);
        assert_eq!(align_up(0u64, 512u64).unwrap(), 0u64);
        assert_eq!(align_up(513u64, 512u64).unwrap(), 1024u64);
        assert_eq!(align_up(0xFFFFu64, 0x1000u64).unwrap(), 0x10000u64);
        assert_eq!(align_up(0x1000u64, 0x1000u64).unwrap(), 0x1000u64);
        assert!(align_up(100u64, 3u64).is_err()); // not power of two
        // Check for overflow
        assert!(align_up(u64::MAX, 2u64).is_err());
    }

    #[test]
    fn test_align_range() {
        let (base, len) = align_range(1023u64, 2048u64, 512u64).unwrap();
        assert_eq!(base, 512u64);
        assert_eq!(len, 2560u64);

        let (base, len) = align_range(0u64, 100u64, 64u64).unwrap();
        assert_eq!(base, 0u64);
        assert_eq!(len, 128u64);

        let (base, len) = align_range(100u64, 100u64, 64u64).unwrap();
        assert_eq!(base, 64u64);
        assert_eq!(len, 192u64);

        assert!(align_range(100u64, 100u64, 3u64).is_err()); // not power of two
    }
    #[test]
    fn test_signature() {
        const TEST0: u16 = signature!('A');
        assert_eq!(TEST0, 0x0041);

        const TEST1: u16 = signature!('A', '\0');
        assert_eq!(TEST1, 0x0041);

        const TEST2: u16 = signature!('A', 'B');
        assert_eq!(TEST2, 0x4241);

        const TEST3: u32 = signature!('A', 'B', 'C');
        assert_eq!(TEST3, 0x00434241);

        const TEST4: u32 = signature!('A', 'B', 'C', 'D');
        assert_eq!(TEST4, 0x44434241);

        const TEST5: u32 = signature!('\0', '\0', 'C', 'D');
        assert_eq!(TEST5, 0x44430000);

        const TEST6: u64 = signature!('A', 'B', 'C', 'D', 'E');
        assert_eq!(TEST6, 0x0000004544434241);

        const TEST7: u64 = signature!('A', 'B', 'C', 'D', 'E', 'F', 'G', 'H');
        assert_eq!(TEST7, 0x4847464544434241);
    }

    #[test]
    fn test_bit_macro_simple() {
        assert_eq!(bit!(0), 0b1);
        assert_eq!(bit!(1), 0b10);
        assert_eq!(bit!(2), 0b100);
        assert_eq!(bit!(3), 0b1000);
        assert_eq!(bit!(4), 0b1_0000);
        assert_eq!(bit!(5), 0b10_0000);
        assert_eq!(bit!(6), 0b100_0000);
        assert_eq!(bit!(7), 0b1000_0000);
        assert_eq!(bit!(8), 0b1_0000_0000);
        assert_eq!(bit!(9), 0b10_0000_0000);
        assert_eq!(bit!(10), 0b100_0000_0000);
        assert_eq!(bit!(20), 0b1_0000_0000_0000_0000_0000);
        assert_eq!(bit!(30), 0b100_0000_0000_0000_0000_0000_0000_0000u64);
        assert_eq!(bit!(63), 0x8000_0000_0000_0000u64);
    }

    #[test]
    fn test_bit_macro_or() {
        let combined = bit!(1) | bit!(3) | bit!(5);
        assert_eq!(combined, 0b101010);

        let combined = bit!(0) | bit!(2) | bit!(4) | bit!(6) | bit!(8);
        assert_eq!(combined, 0b101010101);
    }

    #[test]
    fn test_bit_with_types_specified_works() {
        let b1: u8 = bit!(3);
        assert_eq!(b1, 0b0000_1000u8);

        let b2: u16 = bit!(10);
        assert_eq!(b2, 0b0000_0100_0000_0000u16);

        let b3: u32 = bit!(20);
        assert_eq!(b3, 0x0010_0000u32);

        let b4: u64 = bit!(40);
        assert_eq!(b4, 0x0100_0000_0000u64);

        let b5: u128 = bit!(50);
        assert_eq!(b5, 0x0004_0000_0000_0000u128);
    }
}
