use loongArch64::register::ecfg::LineBasedInterrupt;
use spin::Once;

use crate::arch::kernel::{eiointc::Eiointc, platic::Platic};
pub struct IrqCtrl {
    platic: Platic,
}

impl IrqCtrl {
    fn create() -> Option<Self> {
        Eiointc::init(1);
        const PLATIC_BASE: usize = 0x8000_0000_1000_0000;
        let platic = Platic::new(PLATIC_BASE);
        platic.write_w(Platic::INT_POLARITY, 0x0);
        platic.write_w(Platic::INT_POLARITY + 4, 0x0);
        platic.write_w(Platic::INTEDGE, 0x0);
        platic.write_w(Platic::INTEDGE + 4, 0x0);
        Some(Self { platic })
    }

    fn enable_irq(&self, no: usize) {
        Eiointc::enable_irq(no);
        if no < 32 {
            let mut mask = self.platic.read_w(Platic::INT_MASK);
            mask &= !(1 << no);
            self.platic.write_w(Platic::INT_MASK, mask);
            self.platic.write_b(Platic::HTMSI_VECTOR0 + no, no as u8);
        } else if no < 64 {
            let mut mask = self.platic.read_w(Platic::INT_MASK);
            mask &= !(1 << (no - 32));
            self.platic.write_w(Platic::INT_MASK, mask);
            self.platic.write_b(Platic::HTMSI_VECTOR32 + no, no as u8);
        } else {
            log::warn!("[IrqCtrl] irq_no > 64");
        }
    }

    fn disable_irq(&self, no: usize) {
        Eiointc::disable_irq(no);
        if no < 32 {
            let mut mask = self.platic.read_w(Platic::INT_MASK);
            mask |= 1 << no;
            self.platic.write_w(Platic::INT_MASK, mask);
        } else if no < 64 {
            let mut mask = self.platic.read_w(Platic::INT_MASK);
            mask |= 1 << (no - 32);
            self.platic.write_w(Platic::INT_MASK, mask);
        } else {
            log::warn!("[IrqCtrl] irq_no > 64");
        }
    }

    fn claim_irq(&self) -> Option<usize> {
        Eiointc::claim_irq()
    }

    fn complete_irq(&self, no: usize) {
        Eiointc::disable_irq(no);
    }
}

pub static IRQ_CTRL: Once<IrqCtrl> = Once::new();

pub fn init() {
    IRQ_CTRL.call_once(|| IrqCtrl::create().expect("Failed to create IrqCtrl"));
    for i in 0..64 {
        IRQ_CTRL.get().unwrap().enable_irq(i);
    }
    loongArch64::register::ecfg::set_lie(
        LineBasedInterrupt::HWI0
            | LineBasedInterrupt::HWI1
            | LineBasedInterrupt::HWI2
            | LineBasedInterrupt::HWI3
            | LineBasedInterrupt::HWI4
            | LineBasedInterrupt::HWI5
            | LineBasedInterrupt::HWI6
            | LineBasedInterrupt::HWI7,
    );
}

pub fn claim_interrupt() -> Option<usize> {
    IRQ_CTRL.get().and_then(|irq_ctrl| irq_ctrl.claim_irq())
}

pub fn complete_interrupt(irq_number: usize) {
    if let Some(irq_ctrl) = IRQ_CTRL.get() {
        irq_ctrl.complete_irq(irq_number);
    }
}
