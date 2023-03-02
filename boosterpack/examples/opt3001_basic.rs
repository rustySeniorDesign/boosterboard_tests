//! Usage example for the OPT3001 driver

#![no_main]
#![no_std]

use embedded_hal::prelude::*;
use msp430_rt::entry;
use msp430fr2355::{E_USCI_A1, E_USCI_B0};
use msp430fr2x5x_hal::{
    clock::{ClockConfig, DcoclkFreqSel, MclkDiv, SmclkDiv},
    fram::Fram,
    gpio::Batch,
    pmm::Pmm,
    serial::*,
    watchdog::Wdt,
    i2c::*,
};
use core::panic::PanicInfo;
use msp430fr2x5x_hal::clock::Aclk;
use msp430fr2355_boosterpack::{
    opt3001::DeviceOpt3001,
    serial_utils::{print_bytes, u32_to_dec, byte_to_dec}
};
use msp430fr2355_boosterpack::serial_utils::init_serial;


#[cfg(debug_assertions)]
#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    // Disable interrupts to prevent further damage.
    msp430::interrupt::disable();
    if let Some(location) = _info.location() {
        print_bytes(b"Panic occurred in file ");
        print_bytes(location.file().as_bytes());
        print_bytes(b" at line ");
        print_bytes(&u32_to_dec(location.line()));
        print_bytes(b"\n");
    } else {
        print_bytes(b"Panic handler was called, something bad happened.\n");
    }
    loop {
        // Prevent optimizations that can remove this loop.
        msp430::asm::barrier();
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

        init_serial(tx);


        print_bytes(b"Serial started\n\nConfiguring USCI B0 for I2C...\n");

        // P1.3 SCL, P1.2 SDA
        let p1 = Batch::new(periph.P1).split(&pmm);
        let mut config: I2CBusConfig<E_USCI_B0> = I2CBusConfig::new(periph.E_USCI_B0);
        config.use_smclk(&_smclk, 5);// ~100 MHz
        let periph_i2c : SDL<E_USCI_B0> = config.sdl(p1.pin3.to_alternate1(), p1.pin2.to_alternate1());

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