//! Usage example for the OPT3001 driver

#![no_main]
#![no_std]

use core::panic::PanicInfo;
use embedded_hal::digital::v2::OutputPin;
use embedded_hal::prelude::_embedded_hal_adc_OneShot;
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

#[entry]
fn main() -> ! {
    if let Some(periph) = msp430fr2355::Peripherals::take() {
        let mut fram = Fram::new(periph.FRCTL);
        let _wdt = Wdt::constrain(periph.WDT_A);
        let (_smclk, aclk, _delay) = ClockConfig::new(periph.CS)
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
            Parity::NoParity,
            Loopback::NoLoop,
            9600,
        )
        .use_aclk(&aclk)
        .split(p4.pin3.to_alternate1(), p4.pin2.to_alternate1());

        init_serial(rx, tx);

        print_bytes(b"Serial started\n");

        print_bytes(b"Configuring ADC...\n");

        // ADC Channel 5 (P1.5) = Joystick X Axis
        // see page 96 of datasheet -> (P1Selx = 0b11)
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
            let result: u16 = adc.read(&mut adc_pin).unwrap();

            print_bytes(&u16_to_dec(result));

            // turn on green LED if joystick pushed to far left or right
            if result > 900 || result < 100 {
                led_g.set_high().unwrap();
            } else {
                led_g.set_low().unwrap();
            }
        }
    }
    loop {}
}

#[no_mangle]
extern "C" fn abort() -> ! {
    panic!();
}
