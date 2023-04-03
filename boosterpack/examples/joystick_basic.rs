//! Usage example for the OPT3001 driver

#![no_main]
#![no_std]

use core::panic::PanicInfo;
use embedded_hal::{digital::v2::*, prelude::_embedded_hal_adc_OneShot};
use msp430_rt::entry;
use msp430fr2355_boosterpack::serial_utils::*;
use msp430fr2x5x_hal::{
    adc::*,
    clock::{ClockConfig, DcoclkFreqSel, MclkDiv, SmclkDiv},
    fram::Fram,
    gpio::Batch,
    pmm::Pmm,
    serial::*,
    watchdog::Wdt,
};

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

fn print_regs_gpio() {
    unsafe {
        let p1_dir: u8 = *(0x204 as *mut u8);
        let p1_sel0: u8 = *(0x20A as *mut u8);
        let p1_sel1: u8 = *(0x20C as *mut u8);
        let pm5_ctl0: u16 = *((0x120 + 0x10) as *mut u16);

        print_bytes(b"p1_dir: ");
        print_bytes(&u16_to_hex(p1_dir as u16));
        print_bytes(b"\n");
        print_bytes(b"p1_sel0: ");
        print_bytes(&u16_to_hex(p1_sel0 as u16));
        print_bytes(b"\n");
        print_bytes(b"p1_sel1: ");
        print_bytes(&u16_to_hex(p1_sel1 as u16));
        print_bytes(b"\n");
        print_bytes(b"pm5_ctl0: ");
        print_bytes(&u16_to_hex(pm5_ctl0));
        print_bytes(b"\n");
    }
}

fn print_regs() {
    unsafe {
        let ctl0: u16 = *(0x700 as *mut u16);
        let ctl1: u16 = *(0x702 as *mut u16);
        let ctl2: u16 = *(0x704 as *mut u16);
        let lo: u16 = *(0x706 as *mut u16);
        let hi: u16 = *(0x708 as *mut u16);
        let mctl0: u16 = *(0x70A as *mut u16);
        let mem0: u16 = *(0x712 as *mut u16);
        let ie: u16 = *(0x71A as *mut u16);
        let ifg: u16 = *(0x71C as *mut u16);
        let iv: u16 = *(0x71E as *mut u16);

        print_bytes(b"ctl0: ");
        print_bytes(&u16_to_hex(ctl0));
        print_bytes(b"\n");
        print_bytes(b"ctl1: ");
        print_bytes(&u16_to_hex(ctl1));
        print_bytes(b"\n");
        print_bytes(b"ctl2: ");
        print_bytes(&u16_to_hex(ctl2));
        print_bytes(b"\n");
        print_bytes(b"lo: ");
        print_bytes(&u16_to_hex(lo));
        print_bytes(b"\n");
        print_bytes(b"hi: ");
        print_bytes(&u16_to_hex(hi));
        print_bytes(b"\n");
        print_bytes(b"mctl0: ");
        print_bytes(&u16_to_hex(mctl0));
        print_bytes(b"\n");
        print_bytes(b"mem0: ");
        print_bytes(&u16_to_hex(mem0));
        print_bytes(b"\n");
        print_bytes(b"ie: ");
        print_bytes(&u16_to_hex(ie));
        print_bytes(b"\n");
        print_bytes(b"ifg: ");
        print_bytes(&u16_to_hex(ifg));
        print_bytes(b"\n");
        print_bytes(b"iv: ");
        print_bytes(&u16_to_hex(iv));
        print_bytes(b"\n");
        print_bytes(b"==================\n");
    }
}

#[entry]
fn main() -> ! {
    if let Some(periph) = msp430fr2355::Peripherals::take() {
        let mut fram = Fram::new(periph.FRCTL);
        let _wdt = Wdt::constrain(periph.WDT_A);
        let (_smclk, aclk, mut delay) = ClockConfig::new(periph.CS)
            .mclk_dcoclk(DcoclkFreqSel::_1MHz, MclkDiv::_1)
            .smclk_on(SmclkDiv::_2)
            .aclk_refoclk()
            .freeze(&mut fram);

        let pmm = Pmm::new(periph.PMM);

        let p1 = Batch::new(periph.P1).split(&pmm);
        let p4 = Batch::new(periph.P4).split(&pmm);
        let p6 = Batch::new(periph.P6).split(&pmm);

        let mut led_g = p6.pin6.to_output();

        let (tx, rx) = SerialConfig::new(
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

        init_serial(rx, tx);

        print_bytes(b"Serial started\n\nConfiguring ADC...\n");

        // set port 1 pin 5 to analog input (P1Selx = 0b11)
        // page 96
        let mut adc_pin = p1.pin5.to_alternate3();

        let mut adc = AdcConfig::new(
            periph.ADC,
            ClockSource::MODCLK,
            ClockDivider::_1,
            Predivider::_1,
            Resolution::_10BIT,
            SamplingRate::_200KSPS,
            SampleTime::_4,
        )
        .config_hw();

        loop {
            let result: Result<u16, nb::Error<()>> = adc.read(&mut adc_pin);

            print_bytes(&u16_to_dec(result.unwrap()));
        }
    }
    loop {}
}

#[no_mangle]
extern "C" fn abort() -> ! {
    panic!();
}
