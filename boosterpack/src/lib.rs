#![no_std]

#![feature(abi_msp430_interrupt)]
#![feature(core_panic)]

pub mod opt3001;
pub mod serial_utils;
pub mod stream;
pub mod queuebuf;

pub use msp430fr2355 as pac;
pub use embedded_hal as hal;
pub use msp430fr2x5x_hal as msp_hal;