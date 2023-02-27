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
        print_bytes(b"Panic handler was called, something bad happened.\n");
    }
    loop {
        // Prevent optimizations that can remove this loop.
        msp430::asm::barrier();
    }
}

// #[cfg(debug_assertions)]
// // use panic_msp430 as _;
// #[cfg(not(debug_assertions))]
// use panic_never as _;

fn byte_to_dec(byte:u8, out_buf:&mut [u8]){
    let digit1 = byte/100;

    out_buf[0] = (digit1) + b'0';
    out_buf[1] = digit1/10 + b'0';
    out_buf[2] = digit1%10 + b'0';
}

fn byte_to_hex(byte:u8, out_buf:&mut [u8]){
    out_buf[0] = ((byte&0xF0) >> 4) + b'0';
    out_buf[1] = ((byte&0x0F) >> 4) + b'0';
    if out_buf[0] > b'9' {
        out_buf[0] += 7;
    }
    if out_buf[1] > b'9' {
        out_buf[1] += 7;
    }
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
        let mut config: I2CBusConfig<E_USCI_B0> = I2CBusConfig::new(periph.E_USCI_B0, 12);
        config.use_smclk(&_smclk);
        let mut periph_i2c : SDL<E_USCI_B0> = config.sdl(p1.pin3.to_alternate1(), p1.pin2.to_alternate1());
        //
        print_bytes(b"I2C peripheral configured\n\nConfiguring opt3001 sensor...\n");

        // address:  1000100 (0x44)
        // config reg (0x1):
        // 15:12 : 1100
        // 11 : 0
        // 10:9 : 10
        // 8:5 : dc
        // 4 : 1
        // 3 : 0
        // 2 : 0
        // 1:0 : 00
        let address:u8 = 0x44;
        let config_cmd: [u8; 3] = [0x1, 0xC4, 0x10];
        let is_ok;
        let mut res = periph_i2c.write(address, &config_cmd)
            .and_then(|_| {periph_i2c.write(address, &[0x00u8])});
            // .and_then(|_| {periph_i2c.write(address, &[0xFFu8])});
        match res{
            Ok(()) =>  {
                print_bytes(b"Configuration successful\n\n");
                is_ok = true;
            },
            Err(I2CErr::GotNACK) => {
                print_bytes(b"Configuration failed: got NACK response\n\n");
                is_ok = false;
            },
            _ => {
                print_bytes(b"Configuration failed\n\n");
                is_ok = false;
            }
        };

        if is_ok {
            print_bytes(b"Polling from device...\n");
            let mut buf : [u8; 2] = [0;2];
            loop {
                buf[0] = 0;
                buf[1] = 0;
                match periph_i2c.read(address, &mut buf) {
                    Ok(()) =>  {
                        let mut byte_chars : [u8;2] = [0;2];
                        byte_to_hex(buf[1], &mut byte_chars);
                        print_bytes(b"0x");
                        print_bytes(&byte_chars);
                        byte_to_hex(buf[0], &mut byte_chars);
                        print_bytes(&byte_chars);
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
        }

    }
    loop {}
}

#[no_mangle]
extern "C" fn abort() -> ! {
    panic!();
}