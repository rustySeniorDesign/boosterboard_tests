//! Usage example for the OPT3001 driver
//!
//! Also contains a few utilities related to serial output

#![no_main]
#![no_std]

use embedded_hal::digital::v2::OutputPin;
use embedded_hal::prelude::*;
use msp430_rt::entry;
use msp430fr2355::{E_USCI_A1, E_USCI_B0, Peripherals};
use msp430fr2x5x_hal::{
    clock::{ClockConfig, DcoclkFreqSel, MclkDiv, SmclkDiv, Aclk},
    fram::Fram,
    gpio::Batch,
    pmm::Pmm,
    serial::*,
    watchdog::Wdt,
    i2c::*,
};
use msp430fr2355_boosterpack::opt3001;
use core::panic::PanicInfo;
use core::str::Bytes;
use msp430fr2355_boosterpack::opt3001::DeviceOpt3001;

static mut TX_GLOBAL: Option<Tx<E_USCI_A1>> = None;

fn print_bytes(bytes:&[u8]){
    unsafe {
        if TX_GLOBAL.is_some() {
            let tx = TX_GLOBAL.as_mut().expect("no tx!");
            tx.bwrite_all(bytes).ok();
        }

    }
}

#[cfg(debug_assertions)]
#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    // Disable interrupts to prevent further damage.
    msp430::interrupt::disable();
    unsafe {
        if let Some(location) = _info.location() {
            print_bytes(b"Panic occurred in file ");
            print_bytes(location.file().as_bytes());
            print_bytes(b" at line ");
            print_bytes(&u32_to_dec(location.line()));
            print_bytes(b"\n");
        } else {
            print_bytes(b"Panic handler was called, something bad happened.\n");
        }

    }
    loop {
        // Prevent optimizations that can remove this loop.
        msp430::asm::barrier();
    }
}


fn byte_to_dec(val:u8) -> [u8;3]{
    let mut out_buf: [u8;3] = [0;3];
    let mut over_ten = val;
    for i in 0..=2 {
        let next = over_ten / 10;
        out_buf[2-i] = ((over_ten - (next * 10) ) as u8) + b'0';
        over_ten = next;
    }
    out_buf
}

fn u32_to_dec(val:u32) -> [u8;9]{
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

fn byte_to_hex(val:u8) -> [u8;2] {
    [
        HEX_LOOKUP[((val&0xF0) >> 4) as usize],
        HEX_LOOKUP[(val&0x0F) as usize],
    ]
}

fn u16_to_hex(val:u16) -> [u8;4]{
    [
        HEX_LOOKUP[((val&0xF000) >> 12) as usize],
        HEX_LOOKUP[((val&0x0F00) >> 8) as usize],
        HEX_LOOKUP[((val&0x00F0) >> 4) as usize],
        HEX_LOOKUP[(val&0x000F) as usize]
    ]
}

fn u32_to_hex(val:u32) -> [u8;8]{
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

#[entry]
fn main() -> ! {

    if let Some(periph) = msp430fr2355::Peripherals::take() {
        let mut fram = Fram::new(periph.FRCTL);
        let _wdt = Wdt::constrain(periph.WDT_A);
        let (_smclk, aclk) = ClockConfig::new(periph.CS)
            .mclk_dcoclk(DcoclkFreqSel::_1MHz, MclkDiv::_1)
            .smclk_on(SmclkDiv::_2)
            .aclk_refoclk()
            .freeze(&mut fram);
        let pmm = Pmm::new(periph.PMM);
        let p4 = Batch::new(periph.P4).split(&pmm);
        let (tx, mut _rx) = SerialConfig::new(
            periph.E_USCI_A1,
            BitOrder::LsbFirst,
            BitCount::EightBits,
            StopBits::OneStopBit,
            // Launchpad UART-to-USB converter doesn't handle parity, so we don't use it
            Parity::NoParity,
            Loopback::NoLoop,
            9600,
        )
            .use_aclk(&aclk)
            .split(p4.pin3.to_alternate1(), p4.pin2.to_alternate1());


        unsafe{TX_GLOBAL = Some(tx);}


        print_bytes(b"Serial started\n\nConfiguring USCI B0 for I2C...\n");

        // P1.3 SCL, P1.2 SDA
        let p1 = Batch::new(periph.P1).split(&pmm);
        let mut config: I2CBusConfig<E_USCI_B0> = I2CBusConfig::new(periph.E_USCI_B0);
        config.use_smclk(&_smclk, 5);// ~100 MHz
        let mut periph_i2c : SDL<E_USCI_B0> = config.sdl(p1.pin3.to_alternate1(), p1.pin2.to_alternate1());

        print_bytes(b"I2C peripheral configured\n\nConfiguring opt3001 sensor...\n");


        let mut device : DeviceOpt3001<E_USCI_B0>;
        match DeviceOpt3001::new(periph_i2c){
            Ok(dev) =>  {
                device = dev;
                print_bytes(b"Configuration successful\n\n");
                print_bytes(b"Polling from device...\n");
                loop {
                    match device.read_light() {
                        Ok(res) =>  {
                            print_bytes(b"lux: ");
                            print_bytes(&u32_to_dec(res.whole)[3..=8]);
                            print_bytes(b".");
                            print_bytes(&byte_to_dec(res.frac)[1..=2]);
                            print_bytes(b"\n");
                        },
                        _ => {
                            print_bytes(b"Read failed\n");
                            break;
                        }
                    }
                    for _ in 0..50000 {
                        msp430::asm::nop();
                    }
                }
            },
            Err(I2CErr::GotNACK) => {
                print_bytes(b"Configuration failed: got NACK response\n\n");
            },
            _ => {
                print_bytes(b"Configuration failed\n\n");
            }
        };
    }
    loop {}
}

#[no_mangle]
extern "C" fn abort() -> ! {
    panic!();
}