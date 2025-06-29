use core::marker::PhantomData;

pub struct WriteOnlyAccess;
pub struct ReadWriteAccess;

pub trait IoPortWriteAccess {}
pub trait IoPortReadAccess {}

impl IoPortWriteAccess for WriteOnlyAccess {}
impl IoPortWriteAccess for ReadWriteAccess {}
impl IoPortReadAccess for ReadWriteAccess {}

pub trait PortRead: Sized {
    unsafe fn read_from_port(_port: u16) -> Self {
        unimplemented!()
    }
}

pub trait PortWrite: Sized {
    unsafe fn write_to_port(_port: u16, _value: Self) {
        unimplemented!()
    }
}

impl PortRead for u8 {
    unsafe fn read_from_port(port: u16) -> Self {
        loongArch64::iocsr::iocsr_read_b(port as _)
    }
}

impl PortWrite for u8 {
    unsafe fn write_to_port(port: u16, value: Self) {
        loongArch64::iocsr::iocsr_write_b(port as _, value);
    }
}
impl PortRead for u16 {
    unsafe fn read_from_port(port: u16) -> Self {
        loongArch64::iocsr::iocsr_read_h(port as _)
    }
}
impl PortWrite for u16 {
    unsafe fn write_to_port(port: u16, value: Self) {
        loongArch64::iocsr::iocsr_write_h(port as _, value);
    }
}
impl PortRead for u32 {
    unsafe fn read_from_port(port: u16) -> Self {
        loongArch64::iocsr::iocsr_read_w(port as _)
    }
}
impl PortWrite for u32 {
    unsafe fn write_to_port(port: u16, value: Self) {
        loongArch64::iocsr::iocsr_write_w(port as _, value);
    }
}
