//! A few utilities related to serial output

use nb;
use embedded_hal::prelude::_embedded_hal_blocking_serial_Write;
use embedded_hal::prelude::_embedded_hal_serial_Read;
use msp430fr2x5x_hal::serial;
use msp430fr2x5x_hal::serial::{Rx, Tx};
use crate::pac;
use crate::pac::E_USCI_A1;

static mut RX_GLOBAL: Option<Rx<E_USCI_A1>> = None;
static mut TX_GLOBAL: Option<Tx<E_USCI_A1>> = None;

pub fn init_serial(rx: Rx<E_USCI_A1>, tx: Tx<E_USCI_A1>){
    unsafe{RX_GLOBAL = Some(rx);}
    unsafe{TX_GLOBAL = Some(tx);}
}

pub fn print_bytes(bytes:&[u8]){
    unsafe {
        match TX_GLOBAL.as_mut() {
            Some(mut tx) =>{
                tx.bwrite_all(bytes).ok();
            }
            None=>{}
        }
    }
}

pub fn get_bytes(bytes:&mut [u8]) -> Result<(), ()>{
    match unsafe{RX_GLOBAL.as_mut()} {
        Some(rx) => {
            for i in 0..bytes.len(){
                match nb::block!(rx.read()) {
                    Ok(data) => {
                        bytes[i] = data;
                        // print_bytes(&[0xAAu8]);
                    }
                    Err(serial::RecvError::Overrun(data)) => {
                        // bytes[i] = data;
                        // return Err(());
                    }
                    Err(_) => {
                        return Err(());
                    }
                };
            }
            Ok(())
        }
        None=>{
            Err(())
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

pub fn u16_to_dec(val:u16) -> [u8;5]{
    let mut out_buf: [u8;5] = [0;5];
    let mut over_ten = val;
    for i in 0..=4 {
        let next = over_ten / 10;
        out_buf[4-i] = ((over_ten - (next * 10) ) as u8) + b'0';
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









