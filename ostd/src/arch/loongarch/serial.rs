// SPDX-License-Identifier: MPL-2.0

//! The console I/O.

use spin::mutex::SpinMutex;

use super::device::serial::Serial;

const UART_ADDR: usize = 0x800000001FE001E0;

static CONSOLE_COM1: SpinMutex<Serial> = SpinMutex::new(Serial::new(UART_ADDR));

/// Initializes the serial port.
pub(crate) fn init() {}

/// Sends a byte on the serial port.
pub fn send(data: u8) {
    let mut uart = CONSOLE_COM1.lock();
    match data {
        b'\n' => {
            uart.send(b'\r');
            uart.send(b'\n');
        }
        c => uart.send(c),
    }
}
