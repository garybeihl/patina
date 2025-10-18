//! UEFI Device Path Utilities
//!
//! This library provides various utilities for interacting with UEFI device paths.
//!
//! ## License
//!
//! Copyright (c) Microsoft Corporation.
//!
//! SPDX-License-Identifier: Apache-2.0
//!
#![no_std]
#![feature(coverage_attribute)]

extern crate alloc;

use alloc::vec;
use alloc::{boxed::Box, format, string::String, vec::Vec};
use core::{mem::size_of_val, ptr::slice_from_raw_parts, slice::from_raw_parts};
use r_efi::protocols::device_path::{End, Hardware, Media};

use r_efi::efi;

/// Returns the count of nodes and size (in bytes) of the given device path.
///
/// count and size outputs both include the terminating end node.
///
/// ## SAFETY
///
/// device_path input must be a valid pointer (i.e. not null) that points to
/// a well-formed device path that conforms to UEFI spec 2.11 section 10.
///
/// ## Examples
///
/// ```
/// #![feature(pointer_byte_offsets)]
/// use patina_internal_device_path::device_path_node_count;
/// use r_efi::efi;
/// let device_path_bytes = [
///   efi::protocols::device_path::TYPE_HARDWARE,
///   efi::protocols::device_path::Hardware::SUBTYPE_PCI,
///   0x6,  //length[0]
///   0x0,  //length[1]
///   0x0,  //func
///   0x1C, //device
///   efi::protocols::device_path::TYPE_HARDWARE,
///   efi::protocols::device_path::Hardware::SUBTYPE_PCI,
///   0x6, //length[0]
///   0x0, //length[1]
///   0x0, //func
///   0x0, //device
///   efi::protocols::device_path::TYPE_HARDWARE,
///   efi::protocols::device_path::Hardware::SUBTYPE_PCI,
///   0x6, //length[0]
///   0x0, //length[1]
///   0x2, //func
///   0x0, //device
///   efi::protocols::device_path::TYPE_END,
///   efi::protocols::device_path::End::SUBTYPE_ENTIRE,
///   0x4,  //length[0]
///   0x00, //length[1]
/// ];
/// let device_path_ptr = device_path_bytes.as_ptr() as *const efi::protocols::device_path::Protocol;
/// let (nodes, length) = device_path_node_count(device_path_ptr).unwrap();
/// assert_eq!(nodes, 4);
/// assert_eq!(length, device_path_bytes.len());
/// ```
///
pub fn device_path_node_count(
    device_path: *const efi::protocols::device_path::Protocol,
) -> Result<(usize, usize), efi::Status> {
    let mut node_count = 0;
    let mut dev_path_size: usize = 0;
    let mut current_node_ptr = device_path;
    if current_node_ptr.is_null() {
        debug_assert!(!current_node_ptr.is_null());
        return Err(efi::Status::INVALID_PARAMETER);
    }
    loop {
        // SAFETY: caller must guarantee that device_path is a valid pointer to
        // a well-formed device path as described in the function documentation above.
        let current_node = unsafe { current_node_ptr.read_unaligned() };
        let current_length: usize = u16::from_le_bytes(current_node.length).into();
        node_count += 1;
        dev_path_size += current_length;

        if current_node.r#type == efi::protocols::device_path::TYPE_END {
            break;
        }

        let offset = current_length.try_into().map_err(|_| efi::Status::INVALID_PARAMETER)?;
        // SAFETY: caller must guarantee that device_path is well formed
        current_node_ptr = unsafe { current_node_ptr.byte_offset(offset) };
    }
    Ok((node_count, dev_path_size))
}

/// Copies the device path from the given pointer into a Boxed [u8] slice.
pub fn copy_device_path_to_boxed_slice(
    device_path: *const efi::protocols::device_path::Protocol,
) -> Result<Box<[u8]>, efi::Status> {
    let dp_slice = device_path_as_slice(device_path)?;
    Ok(dp_slice.to_vec().into_boxed_slice())
}

/// Returns the device_path as a byte slice.
pub fn device_path_as_slice(
    device_path: *const efi::protocols::device_path::Protocol,
) -> Result<&'static [u8], efi::Status> {
    let (_, byte_count) = device_path_node_count(device_path)?;
    // SAFETY: Caller must ensure that device_path is valid, that device_path
    // will remain valid for lifetime of slice and that byte_count is valid
    unsafe { Ok(from_raw_parts(device_path as *const u8, byte_count)) }
}

/// Computes the remaining device path and the number of nodes in common for two device paths.
///
/// if device path `a` is a prefix of or identical to device path `b`, result is Some(pointer to the portion of
/// device path `b` that remains after removing device path `a`, nodes_in_common).
/// if device path `a` is not a prefix of device path `b` (i.e. the first node in `a` that is different from
/// `b` is not an end node), then the result is None.
///
/// note: nodes_in_common does not count the terminating end node.
///
/// ## Safety
///
/// a and b inputs must be a valid pointers to well-formed device paths.
/// b memory must remain valid memory for the lifetime of the returned device path.
///
///
/// ## Examples
///
/// ```
/// #![feature(pointer_byte_offsets)]
/// use patina_internal_device_path::{device_path_node_count, remaining_device_path};
/// use core::mem::size_of;
/// use r_efi::efi;
/// let device_path_a_bytes = [
///   efi::protocols::device_path::TYPE_HARDWARE,
///   efi::protocols::device_path::Hardware::SUBTYPE_PCI,
///   0x6,  //length[0]
///   0x0,  //length[1]
///   0x0,  //func
///   0x1C, //device
///   efi::protocols::device_path::TYPE_HARDWARE,
///   efi::protocols::device_path::Hardware::SUBTYPE_PCI,
///   0x6, //length[0]
///   0x0, //length[1]
///   0x0, //func
///   0x0, //device
///   efi::protocols::device_path::TYPE_END,
///   efi::protocols::device_path::End::SUBTYPE_ENTIRE,
///   0x4,  //length[0]
///   0x00, //length[1]
/// ];
/// let device_path_a = device_path_a_bytes.as_ptr() as *const efi::protocols::device_path::Protocol;
/// let device_path_b_bytes = [
///   efi::protocols::device_path::TYPE_HARDWARE,
///   efi::protocols::device_path::Hardware::SUBTYPE_PCI,
///   0x6,  //length[0]
///   0x0,  //length[1]
///   0x0,  //func
///   0x1C, //device
///   efi::protocols::device_path::TYPE_HARDWARE,
///   efi::protocols::device_path::Hardware::SUBTYPE_PCI,
///   0x6, //length[0]
///   0x0, //length[1]
///   0x0, //func
///   0x0, //device
///   efi::protocols::device_path::TYPE_HARDWARE,
///   efi::protocols::device_path::Hardware::SUBTYPE_PCI,
///   0x6, //length[0]
///   0x0, //length[1]
///   0x2, //func
///   0x0, //device
///   efi::protocols::device_path::TYPE_END,
///   efi::protocols::device_path::End::SUBTYPE_ENTIRE,
///   0x4,  //length[0]
///   0x00, //length[1]
/// ];
/// let device_path_b = device_path_b_bytes.as_ptr() as *const efi::protocols::device_path::Protocol;
/// let device_path_c_bytes = [
///   efi::protocols::device_path::TYPE_HARDWARE,
///   efi::protocols::device_path::Hardware::SUBTYPE_PCI,
///   0x6,  //length[0]
///   0x0,  //length[1]
///   0x0,  //func
///   0x0A, //device
///   efi::protocols::device_path::TYPE_END,
///   efi::protocols::device_path::End::SUBTYPE_ENTIRE,
///   0x4,  //length[0]
///   0x00, //length[1]
/// ];
/// let device_path_c = device_path_c_bytes.as_ptr() as *const efi::protocols::device_path::Protocol;
/// // a is a prefix of b.
/// let result = unsafe {remaining_device_path(device_path_a, device_path_b)};
/// assert!(result.is_some());
/// let result = result.unwrap();
/// // the remaining device path of b after going past the prefix in a should start at the size of a in bytes minus the size of the end node.
/// let a_path_length = device_path_node_count(device_path_a).unwrap();
/// let offset = a_path_length.1 - size_of::<efi::protocols::device_path::End>();
/// let offset = offset.try_into().unwrap();
/// let expected_ptr =
///   unsafe { device_path_b_bytes.as_ptr().byte_offset(offset) } as *const efi::protocols::device_path::Protocol;
/// assert_eq!(result, (expected_ptr, a_path_length.0 - 1));
///
/// //b is equal to b.
/// let result = unsafe {remaining_device_path(device_path_b, device_path_b)};
/// assert!(result.is_some());
/// let result = result.unwrap();
/// let b_path_length = device_path_node_count(device_path_b).unwrap();
/// let offset = b_path_length.1 - size_of::<efi::protocols::device_path::End>();
/// let offset = offset.try_into().unwrap();
/// let expected_ptr =
///   unsafe { device_path_b_bytes.as_ptr().byte_offset(offset) } as *const efi::protocols::device_path::Protocol;
/// assert_eq!(result, (expected_ptr, b_path_length.0 - 1));
///
/// //a is not a prefix of c.
/// let result = unsafe {remaining_device_path(device_path_a, device_path_c)};
/// assert!(result.is_none());
///
/// //b is not a prefix of a.
/// let result = unsafe {remaining_device_path(device_path_b, device_path_a)};
/// assert!(result.is_none());
/// ```
pub unsafe fn remaining_device_path(
    a: *const efi::protocols::device_path::Protocol,
    b: *const efi::protocols::device_path::Protocol,
) -> Option<(*const efi::protocols::device_path::Protocol, usize)> {
    let mut a_ptr = a;
    let mut b_ptr = b;
    let mut node_count = 0;
    loop {
        // SAFETY: Caller must ensure pointers are valid device_paths
        let (a_node, b_node) = unsafe { (*a_ptr, *b_ptr) };

        if unsafe { is_device_path_end(&a_node) } {
            return Some((b_ptr, node_count));
        }

        node_count += 1;

        let a_length: usize = u16::from_le_bytes(a_node.length).into();
        let b_length: usize = u16::from_le_bytes(b_node.length).into();
        // SAFETY: caller must assure that device path is valid
        let a_slice = unsafe { slice_from_raw_parts(a_ptr as *const u8, a_length).as_ref() };

        // SAFETY: caller must assure that device path is valid and that memory will remain
        // available for the lifetime of the slice
        let b_slice = unsafe { slice_from_raw_parts(b_ptr as *const u8, b_length).as_ref() };

        if a_slice != b_slice {
            return None;
        }

        let a_offset: isize = a_length.try_into().ok()?;
        let b_offset: isize = b_length.try_into().ok()?;
        // SAFETY: Caller must ensure that the device path is well formed and valid
        a_ptr = unsafe { a_ptr.byte_offset(a_offset) };
        // SAFETY: Caller must ensure that the device path is well formed and valid
        b_ptr = unsafe { b_ptr.byte_offset(b_offset) };
    }
}

/// Determines whether the given device path points to an end-of-device-path node.
/// # Safety
///
/// Caller must ensure that the device_path is valid and aligned
pub unsafe fn is_device_path_end(device_path: *const efi::protocols::device_path::Protocol) -> bool {
    let node_ptr = device_path;
    // SAFETY: Caller must ensure that device_path is valid and aligned
    if let Some(device_path_node) = unsafe { node_ptr.as_ref() } {
        device_path_node.r#type == efi::protocols::device_path::TYPE_END
            && device_path_node.sub_type == efi::protocols::device_path::End::SUBTYPE_ENTIRE
    } else {
        true
    }
}

/// Produces a new byte vector that is the concatenation of `a` and `b`
pub fn concat_device_path_to_boxed_slice(
    a: *const efi::protocols::device_path::Protocol,
    b: *const efi::protocols::device_path::Protocol,
) -> Result<Box<[u8]>, efi::Status> {
    let a_slice = device_path_as_slice(a)?;
    let b_slice = device_path_as_slice(b)?;
    let end_path_size = core::mem::size_of::<efi::protocols::device_path::End>();
    let mut out_bytes = vec![0u8; a_slice.len() + b_slice.len() - end_path_size];
    out_bytes[..a_slice.len()].copy_from_slice(a_slice);
    out_bytes[a_slice.len() - end_path_size..].copy_from_slice(b_slice);
    Ok(out_bytes.into_boxed_slice())
}

/// Device Path Node
#[derive(Debug)]
pub struct DevicePathNode {
    header: efi::protocols::device_path::Protocol,
    data: Vec<u8>,
}

impl PartialEq for DevicePathNode {
    fn eq(&self, other: &Self) -> bool {
        self.header.r#type == other.header.r#type
            && self.header.sub_type == other.header.sub_type
            && self.data == other.data
    }
}
impl Eq for DevicePathNode {}

impl DevicePathNode {
    /// Create a DevicePathNode from raw pointer.
    ///
    /// # Safety
    ///
    /// Caller must ensure that the raw pointer points to a valid device path node structure.
    pub unsafe fn new(node: *const efi::protocols::device_path::Protocol) -> Option<Self> {
        // SAFETY: Caller must ensure node is a valid and well formatted device path
        let header = unsafe { core::ptr::read_unaligned(node) };
        let node_len = u16::from_le_bytes(header.length);
        let data_len = node_len.checked_sub(size_of_val(&header).try_into().ok()?)?;
        // SAFETY: Caller must ensure node is a valid and well formatted device path
        let data_ptr = unsafe { node.byte_offset(size_of_val(&header).try_into().ok()?) } as *const u8;
        // SAFETY: Caller must ensure node is a valid and well formatted device path
        let data = unsafe { from_raw_parts(data_ptr, data_len.into()).to_vec() };
        Some(Self { header, data })
    }

    #[inline]
    /// Returns the header information of the device path node.
    pub fn header(&self) -> &efi::protocols::device_path::Protocol {
        &self.header
    }

    #[inline]
    /// Returns the raw data of the device path node.
    pub fn data(&self) -> &[u8] {
        &self.data
    }

    fn len(&self) -> u16 {
        u16::from_le_bytes(self.header.length)
    }
}

/// Iterator that returns DevicePathNodes for a given raw device path pointer.
///
/// This iterator copies the device path data into DevicePathNode structs to abstract
/// the unsafe raw pointer operations necessary for direct interaction with a device path.
///
pub struct DevicePathWalker {
    next_node: Option<*const efi::protocols::device_path::Protocol>,
}

impl From<DevicePathWalker> for String {
    fn from(device_path_walker: DevicePathWalker) -> Self {
        let mut result = String::new();
        for node in device_path_walker {
            if unsafe { is_device_path_end(&node.header) } {
                break;
            }
            result.push_str(protocol_to_subtype_str(node.header));
            if !node.data.is_empty() {
                result.push_str(": ");
                for (i, byte) in node.data.iter().enumerate() {
                    if i > 0 {
                        result.push(',');
                    }
                    result.push_str(&format!("0x{byte:02x}"));
                }
                result.push('/');
            }
        }
        result
    }
}

impl DevicePathWalker {
    /// Creates a DevicePathWalker iterator for the given raw device path pointer.
    ///
    /// ## Safety
    /// Caller must ensure that the raw pointer points to a valid device path structure,
    /// including a proper device path end node.
    pub unsafe fn new(device_path: *const efi::protocols::device_path::Protocol) -> Self {
        Self { next_node: Some(device_path) }
    }
}

impl Iterator for DevicePathWalker {
    type Item = DevicePathNode;
    fn next(&mut self) -> Option<Self::Item> {
        match self.next_node {
            Some(node) => {
                // SAFETY: Caller must assure that node is a valid, well formatted device path
                let current = unsafe { DevicePathNode::new(node)? };
                if unsafe { is_device_path_end(node) } {
                    self.next_node = None;
                } else {
                    // SAFETY: Caller must ensure that node is a valid, well formatted device path
                    self.next_node = Some(unsafe { node.byte_offset(current.len().try_into().ok()?) });
                }
                Some(current)
            }
            None => None,
        }
    }
}

fn protocol_to_subtype_str(protocol: efi::protocols::device_path::Protocol) -> &'static str {
    match protocol.r#type {
        r_efi::protocols::device_path::TYPE_HARDWARE => match protocol.sub_type {
            Hardware::SUBTYPE_PCI => "Pci",
            Hardware::SUBTYPE_PCCARD => "PcCard",
            Hardware::SUBTYPE_MMAP => "MemMap",
            Hardware::SUBTYPE_VENDOR => "Vendor",
            Hardware::SUBTYPE_CONTROLLER => "Controller",
            Hardware::SUBTYPE_BMC => "Bmc",
            _ => "UnknownHardware",
        },
        r_efi::protocols::device_path::TYPE_ACPI => "Acpi",
        r_efi::protocols::device_path::TYPE_MESSAGING => "Msg",
        r_efi::protocols::device_path::TYPE_BIOS => "Bios",
        r_efi::protocols::device_path::TYPE_MEDIA => match protocol.sub_type {
            Media::SUBTYPE_HARDDRIVE => "HardDrive",
            Media::SUBTYPE_CDROM => "CdRom",
            Media::SUBTYPE_VENDOR => "Vendor",
            Media::SUBTYPE_FILE_PATH => "FilePath",
            Media::SUBTYPE_MEDIA_PROTOCOL => "MediaProtocol",
            Media::SUBTYPE_PIWG_FIRMWARE_FILE => "FirmwareFile",
            Media::SUBTYPE_PIWG_FIRMWARE_VOLUME => "FirmwareVolume",
            Media::SUBTYPE_RELATIVE_OFFSET_RANGE => "RelativeOffsetRange",
            Media::SUBTYPE_RAM_DISK => "RamDisk",
            _ => "UnknownMedia",
        },
        r_efi::protocols::device_path::TYPE_END => match protocol.sub_type {
            End::SUBTYPE_INSTANCE => "EndInstance",
            End::SUBTYPE_ENTIRE => "EndEntire",
            _ => "UnknownEnd",
        },
        _ => "UnknownType",
    }
}

#[cfg(test)]
#[coverage(off)]
mod tests {
    use core::mem::size_of;

    use efi::protocols::device_path::{End, Hardware, TYPE_END, TYPE_HARDWARE};
    use r_efi::protocols::device_path::{TYPE_ACPI, TYPE_MEDIA};

    use super::*;

    #[test]
    fn device_path_node_count_should_return_the_right_number_of_nodes_and_length() {
        //build a device path as a byte array for the test.
        let device_path_bytes = [
            TYPE_HARDWARE,
            Hardware::SUBTYPE_PCI,
            0x6,  //length[0]
            0x0,  //length[1]
            0x0,  //func
            0x1C, //device
            TYPE_HARDWARE,
            Hardware::SUBTYPE_PCI,
            0x6, //length[0]
            0x0, //length[1]
            0x0, //func
            0x0, //device
            TYPE_HARDWARE,
            Hardware::SUBTYPE_PCI,
            0x6, //length[0]
            0x0, //length[1]
            0x2, //func
            0x0, //device
            TYPE_END,
            End::SUBTYPE_ENTIRE,
            0x4,  //length[0]
            0x00, //length[1]
        ];
        let device_path_ptr = device_path_bytes.as_ptr() as *const efi::protocols::device_path::Protocol;
        let (nodes, length) = device_path_node_count(device_path_ptr).unwrap();
        assert_eq!(nodes, 4);
        assert_eq!(length, device_path_bytes.len());
    }

    #[test]
    fn remaining_device_path_should_return_remaining_device_path() {
        //build device paths as byte arrays for the tests.
        let device_path_a_bytes = [
            TYPE_HARDWARE,
            Hardware::SUBTYPE_PCI,
            0x6,  //length[0]
            0x0,  //length[1]
            0x0,  //func
            0x1C, //device
            TYPE_HARDWARE,
            Hardware::SUBTYPE_PCI,
            0x6, //length[0]
            0x0, //length[1]
            0x0, //func
            0x0, //device
            TYPE_END,
            End::SUBTYPE_ENTIRE,
            0x4,  //length[0]
            0x00, //length[1]
        ];
        let device_path_a = device_path_a_bytes.as_ptr() as *const efi::protocols::device_path::Protocol;
        let device_path_b_bytes = [
            TYPE_HARDWARE,
            Hardware::SUBTYPE_PCI,
            0x6,  //length[0]
            0x0,  //length[1]
            0x0,  //func
            0x1C, //device
            TYPE_HARDWARE,
            Hardware::SUBTYPE_PCI,
            0x6, //length[0]
            0x0, //length[1]
            0x0, //func
            0x0, //device
            TYPE_HARDWARE,
            Hardware::SUBTYPE_PCI,
            0x6, //length[0]
            0x0, //length[1]
            0x2, //func
            0x0, //device
            TYPE_END,
            End::SUBTYPE_ENTIRE,
            0x4,  //length[0]
            0x00, //length[1]
        ];
        let device_path_b = device_path_b_bytes.as_ptr() as *const efi::protocols::device_path::Protocol;
        let device_path_c_bytes = [
            TYPE_HARDWARE,
            Hardware::SUBTYPE_PCI,
            0x6,  //length[0]
            0x0,  //length[1]
            0x0,  //func
            0x0A, //device
            TYPE_END,
            End::SUBTYPE_ENTIRE,
            0x4,  //length[0]
            0x00, //length[1]
        ];
        let device_path_c = device_path_c_bytes.as_ptr() as *const efi::protocols::device_path::Protocol;

        // a is a prefix of b.
        let result = unsafe { remaining_device_path(device_path_a, device_path_b) };
        assert!(result.is_some());
        let result = result.unwrap();
        // the remaining device path of b after going past the prefix in a should start at the size of a in bytes minus the size of the end node.
        let a_path_length = device_path_node_count(device_path_a).unwrap();
        let offset = a_path_length.1 - size_of::<efi::protocols::device_path::End>();
        let offset = offset.try_into().unwrap();
        let expected_ptr =
            unsafe { device_path_b_bytes.as_ptr().byte_offset(offset) } as *const efi::protocols::device_path::Protocol;
        assert_eq!(result, (expected_ptr, a_path_length.0 - 1));

        //b is equal to b.
        let result = unsafe { remaining_device_path(device_path_b, device_path_b) };
        assert!(result.is_some());
        let result = result.unwrap();
        let b_path_length = device_path_node_count(device_path_b).unwrap();
        let offset = b_path_length.1 - size_of::<efi::protocols::device_path::End>();
        let offset = offset.try_into().unwrap();
        let expected_ptr =
            unsafe { device_path_b_bytes.as_ptr().byte_offset(offset) } as *const efi::protocols::device_path::Protocol;
        assert_eq!(result, (expected_ptr, b_path_length.0 - 1));

        //a is not a prefix of c.
        let result = unsafe { remaining_device_path(device_path_a, device_path_c) };
        assert!(result.is_none());

        //b is not a prefix of a.
        let result = unsafe { remaining_device_path(device_path_b, device_path_a) };
        assert!(result.is_none());
    }

    #[test]
    fn device_path_walker_should_return_correct_device_path_nodes() {
        //build a device path as a byte array for the test.
        let device_path_bytes = [
            TYPE_HARDWARE,
            Hardware::SUBTYPE_PCI,
            0x6,  //length[0]
            0x0,  //length[1]
            0x0,  //func
            0x1C, //device
            TYPE_HARDWARE,
            Hardware::SUBTYPE_PCI,
            0x6, //length[0]
            0x0, //length[1]
            0x0, //func
            0x0, //device
            TYPE_HARDWARE,
            Hardware::SUBTYPE_PCI,
            0x6, //length[0]
            0x0, //length[1]
            0x2, //func
            0x0, //device
            TYPE_END,
            End::SUBTYPE_ENTIRE,
            0x4,  //length[0]
            0x00, //length[1]
        ];
        let device_path_ptr = device_path_bytes.as_ptr() as *const efi::protocols::device_path::Protocol;

        let mut device_path_walker = unsafe { DevicePathWalker::new(device_path_ptr) };

        let node = device_path_walker.next().unwrap();
        assert_eq!(node.header.r#type, TYPE_HARDWARE);
        assert_eq!(node.header.sub_type, Hardware::SUBTYPE_PCI);
        assert_eq!(node.data, vec![0x0u8, 0x1C]);

        let node = device_path_walker.next().unwrap();
        assert_eq!(node.header.r#type, TYPE_HARDWARE);
        assert_eq!(node.header.sub_type, Hardware::SUBTYPE_PCI);
        assert_eq!(node.data, vec![0x0u8, 0x0]);

        let node = device_path_walker.next().unwrap();
        assert_eq!(node.header.r#type, TYPE_HARDWARE);
        assert_eq!(node.header.sub_type, Hardware::SUBTYPE_PCI);
        assert_eq!(node.data, vec![0x02u8, 0x0]);

        let node = device_path_walker.next().unwrap();
        assert_eq!(node.header.r#type, TYPE_END);
        assert_eq!(node.header.sub_type, End::SUBTYPE_ENTIRE);
        assert_eq!(node.data, vec![]);

        assert_eq!(device_path_walker.next(), None);
    }

    #[test]
    fn device_path_nodes_can_be_compared_for_equality() {
        //build a device path as a byte array for the test.
        let device_path_bytes = [
            TYPE_HARDWARE,
            Hardware::SUBTYPE_PCI,
            0x6, //length[0]
            0x0, //length[1]
            0x0, //func
            0x0, //device
            TYPE_HARDWARE,
            Hardware::SUBTYPE_PCI,
            0x6, //length[0]
            0x0, //length[1]
            0x0, //func
            0x0, //device
            TYPE_HARDWARE,
            Hardware::SUBTYPE_PCI,
            0x6, //length[0]
            0x0, //length[1]
            0x2, //func
            0x0, //device
            TYPE_END,
            End::SUBTYPE_ENTIRE,
            0x4,  //length[0]
            0x00, //length[1]
        ];
        let device_path_ptr = device_path_bytes.as_ptr() as *const efi::protocols::device_path::Protocol;
        let device_path_walker = unsafe { DevicePathWalker::new(device_path_ptr) };

        let nodes: Vec<DevicePathNode> = device_path_walker.collect();

        assert_eq!(nodes[0], nodes[0]);
        assert_eq!(nodes[0], nodes[1]);
        assert_ne!(nodes[0], nodes[2]);
        assert_ne!(nodes[0], nodes[3]);
        assert_ne!(nodes[1], nodes[2]);
        assert_ne!(nodes[1], nodes[3]);
        assert_ne!(nodes[2], nodes[3]);
    }

    #[test]
    fn device_path_node_can_be_converted_to_boxed_slice() {
        //build a device path as a byte array for the test.
        let device_path_bytes = [
            TYPE_HARDWARE,
            Hardware::SUBTYPE_PCI,
            0x6, //length[0]
            0x0, //length[1]
            0x0, //func
            0x0, //device
            TYPE_HARDWARE,
            Hardware::SUBTYPE_PCI,
            0x6, //length[0]
            0x0, //length[1]
            0x0, //func
            0x0, //device
            TYPE_HARDWARE,
            Hardware::SUBTYPE_PCI,
            0x6, //length[0]
            0x0, //length[1]
            0x2, //func
            0x0, //device
            TYPE_END,
            End::SUBTYPE_ENTIRE,
            0x4,  //length[0]
            0x00, //length[1]
        ];
        let device_path_ptr = device_path_bytes.as_ptr() as *const efi::protocols::device_path::Protocol;
        let boxed_device_path = copy_device_path_to_boxed_slice(device_path_ptr);

        assert_eq!(boxed_device_path.unwrap().to_vec(), device_path_bytes.to_vec());
    }

    #[test]
    fn device_path_walker_can_be_converted_to_string() {
        let device_path_bytes = [
            TYPE_HARDWARE,
            Hardware::SUBTYPE_PCI,
            0x6,  //length[0]
            0x0,  //length[1]
            0x0,  //func
            0x1C, //device
            TYPE_ACPI,
            0x0, // subtype doesn't matter for ACPI
            0xC, //length[0]
            0x0, //length[1]
            0x0,
            0x1,
            0x2,
            0x3,
            0x4,
            0x5,
            0x6,
            0x7,
            TYPE_END,
            End::SUBTYPE_ENTIRE,
            0x4,  //length[0]
            0x00, //length[1]
        ];
        let device_path_ptr = device_path_bytes.as_ptr() as *const efi::protocols::device_path::Protocol;
        let device_path_walker = unsafe { DevicePathWalker::new(device_path_ptr) };
        let string: String = device_path_walker.into();

        assert_eq!(string, "Pci: 0x00,0x1c/Acpi: 0x00,0x01,0x02,0x03,0x04,0x05,0x06,0x07/");
    }

    #[test]
    fn test_protocol_to_subtype_str() {
        let mut protocol = efi::protocols::device_path::Protocol {
            r#type: TYPE_HARDWARE,
            sub_type: Hardware::SUBTYPE_PCI,
            length: [0, 0],
        };
        assert_eq!(protocol_to_subtype_str(protocol), "Pci");

        protocol.sub_type = Hardware::SUBTYPE_PCCARD;
        assert_eq!(protocol_to_subtype_str(protocol), "PcCard");

        protocol.sub_type = Hardware::SUBTYPE_MMAP;
        assert_eq!(protocol_to_subtype_str(protocol), "MemMap");

        protocol.sub_type = Hardware::SUBTYPE_VENDOR;
        assert_eq!(protocol_to_subtype_str(protocol), "Vendor");

        protocol.sub_type = Hardware::SUBTYPE_CONTROLLER;
        assert_eq!(protocol_to_subtype_str(protocol), "Controller");

        protocol.sub_type = Hardware::SUBTYPE_BMC;
        assert_eq!(protocol_to_subtype_str(protocol), "Bmc");

        protocol.sub_type = 99; // Unknown hardware subtype
        assert_eq!(protocol_to_subtype_str(protocol), "UnknownHardware");

        protocol.r#type = TYPE_MEDIA;
        protocol.sub_type = Media::SUBTYPE_HARDDRIVE;
        assert_eq!(protocol_to_subtype_str(protocol), "HardDrive");

        protocol.r#type = TYPE_END;
        protocol.sub_type = End::SUBTYPE_INSTANCE;
        assert_eq!(protocol_to_subtype_str(protocol), "EndInstance");

        protocol.r#type = 99; // Unknown type
        assert_eq!(protocol_to_subtype_str(protocol), "UnknownType");
    }
}
