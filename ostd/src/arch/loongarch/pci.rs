// SPDX-License-Identifier: MPL-2.0

//! PCI bus access

use spin::Once;

use crate::{
    bus::pci::PciDeviceLocation, io::IoMem, mm::VmIoOnce, prelude::*, trap::IrqLine, Error,
};

static PCI_IO_MEM: Once<IoMem> = Once::new();

pub(crate) fn has_pci_bus() -> bool {
    PCI_IO_MEM.is_completed()
}

pub(crate) fn write32(location: &PciDeviceLocation, offset: u32, value: u32) -> Result<()> {
    PCI_IO_MEM.get().ok_or(Error::IoError)?.write_once(
        (encode_as_address_offset(location) | (offset & 0xfc)) as usize,
        &value,
    )
}

pub(crate) fn read32(location: &PciDeviceLocation, offset: u32) -> Result<u32> {
    PCI_IO_MEM
        .get()
        .ok_or(Error::IoError)?
        .read_once((encode_as_address_offset(location) | (offset & 0xfc)) as usize)
}

/// Encodes the bus, device, and function into an address offset in the PCI MMIO region.
fn encode_as_address_offset(location: &PciDeviceLocation) -> u32 {
    ((location.bus as u32) << 16)
        | ((location.device as u32) << 11)
        | ((location.function as u32) << 8)
}

pub(crate) fn init() -> Result<()> {
    const PCI_BASE_ADDR: usize = 0x2000_0000;
    const PCI_BASE_SIZE: usize = 0x800_0000;

    PCI_IO_MEM.call_once(|| unsafe {
        IoMem::new(
            PCI_BASE_ADDR..PCI_BASE_ADDR + PCI_BASE_SIZE,
            crate::mm::page_prop::PageFlags::RW,
            crate::mm::page_prop::CachePolicy::Uncacheable,
        )
    });

    Ok(())
}

pub(crate) const MSIX_DEFAULT_MSG_ADDR: u32 = 0x2ff0_0000;

pub(crate) fn construct_remappable_msix_address(remapping_index: u32) -> u32 {
    unimplemented!()
}
