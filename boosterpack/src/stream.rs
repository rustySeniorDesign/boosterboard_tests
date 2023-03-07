//! Functions for streaming data over UART.


use core::{
    borrow::BorrowMut,
    cell::UnsafeCell,
    panic::PanicInfo,
    sync::atomic::Ordering::{Relaxed, Acquire},
};
use core::mem::MaybeUninit;
use core::sync::atomic::Ordering::Release;
use msp430::critical_section::with;
use embedded_graphics::pixelcolor::raw::RawU16;
use embedded_graphics::prelude::RgbColor;
use embedded_hal::blocking::spi;
use embedded_hal::digital::v2::OutputPin;
use embedded_hal::prelude::_embedded_hal_blocking_spi_Write;
use embedded_hal::spi::FullDuplex;
use msp430fr2355::{interrupt};
use msp430::{
    asm,
    interrupt::{CriticalSection, Mutex}
};
use msp430fr2355::E_USCI_B1;
use msp430fr2x5x_hal::{
    spi::SPIPins,
};
use msp430fr2x5x_hal::gpio::{Output, Pin, P3, Pin2};
use portable_atomic::{AtomicBool, AtomicU16};
use st7735_lcd::instruction::Instruction;
use st7735_lcd::ST7735;
use crate::{
    serial_utils,
    serial_utils::RX_GLOBAL,
    queuebuf::QueueBuf
};
use crate::serial_utils::u16_to_dec;

enum Command{
    GetNumImg = 0x1,
    GetImg = 0x2,
    GetStream = 0x3,
    ESCAPED = 0xFF,
}

impl From<Command> for u8 {
    fn from(value: Command) -> Self {
        value as u8
    }
}

pub const BUF_SIZE : usize = 512;
pub const SQUARE_WIDTH : usize = 128;
pub const SQUARE_HEIGHT : usize = 128;
const PACKET_SIZE : usize = BUF_SIZE / 2;
const BUF_PACKETS : usize = (SQUARE_WIDTH * SQUARE_HEIGHT * 2) / BUF_SIZE;
const SCREEN_DY : u16 =  ((BUF_SIZE / SQUARE_WIDTH) / 2) as u16;
const BYTES_REQ : u16 = 32768u16;

fn to_u8(num: u16) -> [u8;2]{
    [(num & 0x00FF) as u8, ((num&0xFF00)>>8) as u8]
}

#[inline]
fn to_u16(arr: &[u8]) -> u16{
    ((arr[1] as u16) << 8) | (arr[0] as u16)
}

/// The global itself technically need to be initialized, since this type holds no data.
/// Make sure, however, that the SPI peripheral has been initialized before using this.
pub static mut SCREEN_SPI_GLOBAL : MaybeUninit<SPIPins<E_USCI_B1>> = MaybeUninit::uninit();
pub static mut DC_PIN : MaybeUninit<Pin<P3, Pin2, Output>> = MaybeUninit::uninit();

// pub fn init_stream(spi_periph: SPIPins<E_USCI_B1>){
//     unsafe {SCREEN_SPI_GLOBAL.write(spi_periph)};
// }

pub fn request_img<SPI: spi::Write<u8>, DC: OutputPin, RST: OutputPin>
(num: u16, screen : &mut ST7735<SPI, DC, RST>) {
    let split = to_u8(num);
    serial_utils::print_bytes(&[0xFFu8, Command::GetImg.into(), split[0], split[1]]);
    download(screen)
}

pub fn request_stream<SPI: spi::Write<u8>, DC: OutputPin, RST: OutputPin>
(screen : &mut ST7735<SPI, DC, RST>) {
    serial_utils::print_bytes(&[0xFFu8, Command::GetStream.into()]);
    download(screen)
}

fn download<SPI: spi::Write<u8>, DC: OutputPin, RST: OutputPin>
    (screen : &mut ST7735<SPI, DC, RST>) {
    let mut spi = unsafe{SCREEN_SPI_GLOBAL.assume_init_mut()};
    let mut rx = unsafe{RX_GLOBAL.assume_init_mut()};
    let mut dc = unsafe{DC_PIN.assume_init_mut()};


    let mut byte_buf = [0u8;6];
    serial_utils::get_bytes(&mut byte_buf).ok();
    serial_utils::print_bytes(&[0xAAu8]);

    let start_x = byte_buf[0] as u16;
    let start_y = byte_buf[1] as u16;
    let end_x = byte_buf[2] as u16;
    let end_y = byte_buf[3] as u16;
    let bytes_required = to_u16(&byte_buf[4..6]);


    BYTES_LEFT.store(bytes_required, Release);
    screen.set_address_window(
        start_x, start_y,end_x,end_y
    ).ok();

    dc.set_low().ok();
    spi.write(&[Instruction::RAMWR as u8]).ok();
    dc.set_high().ok();

    rx.enable_rx_interrupts();
    serial_utils::print_bytes(&[0xAAu8]);

    while BYTES_LEFT.load(Relaxed) != 0 {
        asm::nop();
    }
    rx.disable_rx_interrupts();
}

pub fn get_num_images() -> u16{
    let mut rd_buf = [0u8;2];
    serial_utils::print_bytes(&[0xFFu8, Command::GetNumImg.into()]);
    serial_utils::get_bytes(&mut rd_buf).ok();
    to_u16(&rd_buf)
}

static SPI_TX_BUF: Mutex<UnsafeCell<QueueBuf<BUF_SIZE>>> =
    Mutex::new(UnsafeCell::new(QueueBuf::new([0u8;BUF_SIZE])));
static BYTES_LEFT : AtomicU16 = AtomicU16::new(0u16);

/// UART Rx interrupt from USB, forwards data to SPI Tx handler.
#[interrupt]
fn EUSCI_A1(cs : CriticalSection){
    let mut rx = unsafe{RX_GLOBAL.assume_init_mut()};
    let mut tx_buf : &mut QueueBuf<BUF_SIZE> = unsafe{&mut *SPI_TX_BUF.borrow(cs).get()};
    let spi = unsafe{SCREEN_SPI_GLOBAL.assume_init_mut()};

    spi.tx_interrupt_set(true);
    tx_buf.put(rx.read_no_check());
}

/// SPI Tx interrupt for screen.
/// Should be able to transmit much faster than the UART Rx can receive.
#[interrupt]
fn EUSCI_B1(cs : CriticalSection){
    let mut spi = unsafe{SCREEN_SPI_GLOBAL.assume_init_mut()};
    let tx_buf : &mut QueueBuf<BUF_SIZE> = unsafe{&mut *SPI_TX_BUF.borrow(cs).get()};

    spi.write_no_check(tx_buf.get());
    BYTES_LEFT.sub(1, Relaxed);
    spi.tx_interrupt_set(tx_buf.has_data());
}




