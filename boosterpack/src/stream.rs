//! Functions for streaming data over UART.

use core::cell::RefCell;
use core::cmp::min;
use msp430::critical_section::{with};
use embedded_graphics::pixelcolor::raw::RawU16;
use embedded_graphics::pixelcolor::Rgb565;
use embedded_graphics::prelude::RgbColor;
use embedded_hal::blocking::spi;
use embedded_hal::digital::v2::OutputPin;
use msp430::asm;
use st7735_lcd::ST7735;
use crate::queuebuf::QueueBuf;
use crate::serial_utils;

enum Command{
    GetNumImg = 0x1,
    GetImg = 0x2,
    ESCAPED = 0xFF,
}

impl From<Command> for u8 {
    fn from(value: Command) -> Self {
        value as u8
    }
}

pub const SQUARE_WIDTH : usize = 128;
pub const SQUARE_HEIGHT : usize = 128;
const BUF_SIZE : usize = 1024;
const PACKET_SIZE : usize = BUF_SIZE / 2;
const BUF_PACKETS : usize = (SQUARE_WIDTH * SQUARE_HEIGHT * 2) / BUF_SIZE;
const SCREEN_DY : u16 =  ((BUF_SIZE / SQUARE_WIDTH) / 2) as u16;

pub struct  ImageContainer {
    pub x: u16,
    pub y: u16,
}

fn to_u8(num: u16) -> [u8;2]{
    [(num & 0x00FF) as u8, ((num&0xFF00)>>8) as u8]
}

#[inline]
fn to_u16(arr: &[u8]) -> u16{
    ((arr[1] as u16) << 8) | (arr[0] as u16)
}

#[inline]
fn to_u16_msb(arr: &[u8]) -> u16{
    ((arr[0] as u16) << 8) | (arr[1] as u16)
}

pub fn request_img<SPI: spi::Write<u8>, DC: OutputPin, RST: OutputPin>
    (num: u16, screen : &mut ST7735<SPI, DC, RST>) {
    let split = to_u8(num);
    serial_utils::print_bytes(&[0xFFu8, Command::GetImg.into(), split[0], split[1]]);
    let mut byte_buf = [0u8];
    serial_utils::print_bytes(&[0xAAu8]);
    serial_utils::get_bytes(&mut byte_buf).ok();
    let start_x = byte_buf[0] as u16;
    serial_utils::print_bytes(&[0xAAu8]);
    serial_utils::get_bytes(&mut byte_buf).ok();
    let start_y = byte_buf[0] as u16;
    serial_utils::print_bytes(&[0xAAu8]);
    serial_utils::get_bytes(&mut byte_buf).ok();
    let end_x = byte_buf[0] as u16;
    serial_utils::print_bytes(&[0xAAu8]);
    serial_utils::get_bytes(&mut byte_buf).ok();
    let end_y = byte_buf[0] as u16;
    let bytes_required = ((1+end_y - start_y) * (1+end_x-start_x)) << 1;
    let square_width = 1+end_x-start_x;
    let square_height = 1+end_y-start_y;
    let screen_dy = ((BUF_SIZE as u16) / square_width) >> 1;
    let buf_packets = ((square_width*square_height) << 1) / (BUF_SIZE as u16);
    let mut current_y = start_y;
    let mut rd_buf = [0u8; BUF_SIZE];
    for _ in 0..BUF_PACKETS {
        serial_utils::print_bytes(&[0xAAu8]);
        serial_utils::get_bytes(&mut rd_buf).ok();

        let color_data: [u16; PACKET_SIZE] = unsafe { core::mem::transmute(rd_buf) };

        let next_y: u16 = current_y + screen_dy;
        screen.set_address_window(
            start_x, current_y,
            (start_x + square_width as u16 - 1u16) as u16,
            (next_y - 1u16) as u16
        ).ok();
        current_y = next_y;
        screen.write_pixels(color_data).ok();
    }
}

pub fn get_num_images() -> u16{
    let mut rd_buf = [0u8;2];
    serial_utils::print_bytes(&[0xFFu8, Command::GetNumImg.into()]);
    serial_utils::get_bytes(&mut rd_buf).ok();
    to_u16(&rd_buf)
}


#[interrupt]
fn EUSCI_A1(cs : critical_section){

}

// static SPI_RX_BUF_B0: Mutex<RefCell<QueueBuf>> = Mutex::new(RefCell::new(QueueBuf::new()));
//
// #[inline]
// fn spi_interrupt_shared(){
//
//     with(|cs| {
//         if usci.receive_flag() {
//             let rx_buf = &mut *USCI::get_rx_buf().borrow_ref_mut(cs);
//             rx_buf.put(usci.rxbuf_rd());
//         }
//         if usci.transmit_flag() {
//             let tx_buf = &mut *USCI::get_tx_buf().borrow_ref_mut(cs);
//             if tx_buf.has_data() {
//                 usci.txbuf_wr(tx_buf.get());
//                 if tx_buf.is_empty(){
//                     usci.transmit_interrupt_set(false);
//                     return;
//                 }
//                 // usci.rxbuf_rd(); //dummy read
//             }else{
//                 usci.transmit_interrupt_set(false);
//             }
//         }
//     });
// }