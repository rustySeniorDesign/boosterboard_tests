#![no_main]
#![no_std]

use core::panic::PanicInfo;
use embedded_hal::digital::v2::*;
use embedded_hal::prelude::*;
use msp430_rt::entry;
use msp430fr2x5x_hal::{
    clock::{ClockConfig, DcoclkFreqSel, MclkDiv, SmclkDiv},
    fram::Fram,
    gpio::Batch,
    pmm::Pmm,
    serial::*,
    watchdog::Wdt,
};
use nb::block;

// #[cfg(debug_assertions)]
// use panic_msp430 as _;

#[cfg(not(debug_assertions))]
use panic_never as _;

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    // Disable interrupts to prevent further damage.
    msp430::interrupt::disable();
    // if let Some(location) = _info.location() {
    //     print_bytes(b"Panic occurred in file ");
    //     print_bytes(location.file().as_bytes());
    //     print_bytes(b" at line ");
    //     print_bytes(&u32_to_dec(location.line()));
    //     print_bytes(b"\n");
    // } else {
    //     print_bytes(b"Panic handler was called, something bad happened.\n");
    // }
    loop {
        // Prevent optimizations that can remove this loop.
        msp430::asm::barrier();
    }
}

// Prints "HELLO" when started then echos on UART1
// Serial settings are listed in the code
#[entry]
fn main() -> ! {
    if let Some(periph) = msp430fr2355::Peripherals::take() {
        let mut fram = Fram::new(periph.FRCTL);
        let _wdt = Wdt::constrain(periph.WDT_A);

        let (_smclk, aclk, _) = ClockConfig::new(periph.CS)
            .mclk_dcoclk(DcoclkFreqSel::_1MHz, MclkDiv::_1)
            .smclk_on(SmclkDiv::_2)
            .aclk_refoclk()
            .freeze(&mut fram);

        let pmm = Pmm::new(periph.PMM);
        let mut led = Batch::new(periph.P1).split(&pmm).pin0.to_output();
        let p4 = Batch::new(periph.P4).split(&pmm);
        led.set_low().ok();
        let (mut tx, mut rx) = SerialConfig::new(
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

        tx.bwrite_all(b"HELLO\n").ok();
        led.set_high().ok();

        loop {
            tx.bwrite_all(b"HELLO\n").ok();
            // let ch = match block!(rx.read()) {
            //     Ok(c) => c,
            //     Err(err) => {
            //         (match err {
            //             RecvError::Parity => '!',
            //             RecvError::Overrun(_) => '}',
            //             RecvError::Framing => '?',
            //         }) as u8
            //     }
            // };
            // block!(tx.write(ch)).ok();
            led.toggle().ok();
        }
    } else {
        loop {}
    }
}

// The compiler will emit calls to the abort() compiler intrinsic if debug assertions are
// enabled (default for dev profile). MSP430 does not actually have meaningful abort() support
// so for now, we create our own in each application where debug assertions are present.
#[no_mangle]
extern "C" fn abort() -> ! {
    panic!();
}
