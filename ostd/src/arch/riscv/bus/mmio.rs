// use alloc::vec::Vec;

// use fdt::{node::FdtNode, Fdt};
// use log::info;

// use crate::{
//     arch::{boot::DEVICE_TREE, trap::plic::PLIC},
//     bus::mmio::{common_device::MmioCommonDevice, MMIO_BUS},
//     trap::IrqLine,
// };

// pub(crate) fn get_mmio_devices() -> Vec<MmioCommonDevice> {
//     let fdt = DEVICE_TREE.get().unwrap();
//     let mut devices = Vec::new();
//     for node in fdt.all_nodes() {
//         if let Some(compatible) = node.compatible() {
//             if compatible.all().any(|s| s == "virtio,mmio") {
//                 if let Some(device) = probe_virtio_device(node) {
//                     devices.push(device);
//                 }
//             }
//         }
//     }

//     devices
// }

// /// Probe a virtio device from the device tree node.
// fn probe_virtio_device(node: FdtNode) -> Option<MmioCommonDevice> {
//     if let Some(reg) = node.reg().unwrap().next() {
//         let paddr = reg.starting_address as usize;
//         let size = reg.size.unwrap();
//         let interrupt = node.interrupts().unwrap().next().unwrap() as u8;
//         info!(
//             "Found virtio device at addr={:#x}, size={:#x}, interrupt={}",
//             paddr, size, interrupt
//         );
//         let handle = IrqLine::alloc_specific(interrupt).unwrap();
//         PLIC.get().unwrap().set_priority(interrupt, 6);
//         PLIC.get().unwrap().enable(0, true, interrupt);
//         return Some(MmioCommonDevice::new(paddr, handle));
//     }
//     None
// }
