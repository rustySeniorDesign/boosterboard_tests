use embedded_hal::prelude::_embedded_hal_blocking_serial_Write;
use msp430fr2x5x_hal::serial::{SerialUsci, Tx};
use crate::pac;
use crate::pac::E_USCI_A1;
use msp430fr2x5x_hal::{
    clock::{ClockConfig, DcoclkFreqSel, MclkDiv, SmclkDiv},
    fram::Fram,
    gpio::Batch,
    pmm::Pmm,
    serial::*,
    watchdog::Wdt,
};
use msp430fr2x5x_hal::clock::Aclk;

static mut TX_GLOBAL: Option<Tx<E_USCI_A1>> = None;

pub fn init_serial(tx: Tx<E_USCI_A1>){
    unsafe{TX_GLOBAL = Some(tx);}
}

pub fn print_bytes(bytes:&[u8]){
    unsafe {
        if TX_GLOBAL.is_some() {
            let tx = TX_GLOBAL.as_mut().expect("no tx!");
            tx.bwrite_all(bytes).ok();
        }

    }
}

pub fn byte_to_dec(val:u8) -> [u8;3]{
    let mut out_buf: [u8;3] = [0;3];
    let mut over_ten = val;
    for i in 0..=2 {
        let next = over_ten / 10;
        out_buf[2-i] = ((over_ten - (next * 10) ) as u8) + b'0';
        over_ten = next;
    }
    out_buf
}

pub fn u32_to_dec(val:u32) -> [u8;9]{
    let mut out_buf: [u8;9] = [0;9];
    let mut over_ten = val;
    for i in 0..=8 {
        let next = over_ten / 10;
        out_buf[8-i] = ((over_ten - (next * 10) ) as u8) + b'0';
        over_ten = next;
    }
    out_buf
}

static HEX_LOOKUP: [u8;16] = [b'0', b'1', b'2', b'3', b'4', b'5', b'6', b'7',
    b'8', b'9', b'A', b'B', b'C', b'D', b'E', b'F'];

pub fn byte_to_hex(val:u8) -> [u8;2] {
    [
        HEX_LOOKUP[((val&0xF0) >> 4) as usize],
        HEX_LOOKUP[(val&0x0F) as usize],
    ]
}

pub fn u16_to_hex(val:u16) -> [u8;4]{
    [
        HEX_LOOKUP[((val&0xF000) >> 12) as usize],
        HEX_LOOKUP[((val&0x0F00) >> 8) as usize],
        HEX_LOOKUP[((val&0x00F0) >> 4) as usize],
        HEX_LOOKUP[(val&0x000F) as usize]
    ]
}

pub fn u32_to_hex(val:u32) -> [u8;8]{
    [
        HEX_LOOKUP[((val&0xF0000000) >> 28) as usize],
        HEX_LOOKUP[((val&0x0F000000) >> 24) as usize],
        HEX_LOOKUP[((val&0x00F00000) >> 20) as usize],
        HEX_LOOKUP[((val&0x000F0000) >> 16) as usize],
        HEX_LOOKUP[((val&0x0000F000) >> 12) as usize],
        HEX_LOOKUP[((val&0x00000F00) >> 8) as usize],
        HEX_LOOKUP[((val&0x000000F0) >> 4) as usize],
        HEX_LOOKUP[ (val&0x0000000F) as usize]
    ]
}
