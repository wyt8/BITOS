use crate::{
    bus::pci::PciDeviceLocation, io::IoMem, mm::VmIoOnce, prelude::*, trap::IrqLine, Error,
};

pub(crate) fn has_pci_bus() -> bool {
    false
}

pub(crate) const MSIX_DEFAULT_MSG_ADDR: u32 = 0x2400_0000;

pub(crate) fn construct_remappable_msix_address(irq: &IrqLine) -> u32 {
    unimplemented!()
}

pub(crate) fn write32(location: &PciDeviceLocation, offset: u32, value: u32) -> Result<()> {
    // PCI_BASE_ADDR.get().ok_or(Error::IoError)?.write_once(
    //     (encode_as_address_offset(location) | (offset & 0xfc)) as usize,
    //     &value,
    // )
    Ok(())
}

pub(crate) fn read32(location: &PciDeviceLocation, offset: u32) -> Result<u32> {
    // PCI_BASE_ADDR
    //     .get()
    //     .ok_or(Error::IoError)?
    //     .read_once((encode_as_address_offset(location) | (offset & 0xfc)) as usize)

    Ok(0)
}
