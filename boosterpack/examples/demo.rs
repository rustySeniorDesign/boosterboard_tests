//! Usage example for the OPT3001 driver

#![no_main]
#![no_std]

use core::panic::PanicInfo;
use embedded_graphics::{
    pixelcolor::{raw::RawU16, Rgb565, RgbColor},
    prelude::*,
};
use embedded_hal::digital::v2::{InputPin, OutputPin};
use embedded_hal::prelude::_embedded_hal_adc_OneShot;
use embedded_hal::spi::MODE_0;
use msp430::interrupt;
use msp430_rt::entry;
use msp430fr2355::{E_USCI_B1};
use msp430fr2355_boosterpack::serial_utils::*;
use msp430fr2x5x_hal::{
    adc::*,
    clock::{ClockConfig, DcoclkFreqSel, MclkDiv, SmclkDiv},
    fram::Fram,
    gpio::Batch,
    pmm::Pmm,
    serial::*,
    spi::{SPIBusConfig, SPIPins},
    watchdog::Wdt,
};
use st7735_lcd::ST7735;

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
        let (smclk, aclk, mut delay) = ClockConfig::new(periph.CS)
            .mclk_dcoclk(DcoclkFreqSel::_24MHz, MclkDiv::_1)
            .smclk_on(SmclkDiv::_2)
            .aclk_refoclk()
            .freeze(&mut fram);

        let pmm = Pmm::new(periph.PMM);

        let p1 = Batch::new(periph.P1).split(&pmm);
        let p2 = Batch::new(periph.P2).split(&pmm);
        let p3 = Batch::new(periph.P3).split(&pmm);
        let p4 = Batch::new(periph.P4).split(&pmm);
        let p5 = Batch::new(periph.P5).split(&pmm);
        let p6 = Batch::new(periph.P6).split(&pmm);

        let mut led_g_pin = p6.pin6.to_output();
        let btn_1_pin = p2.pin4.pulldown();
        // let btn_2_pin = p3.pin3.pulldown();  // hardware issue on launchpad - always low for some reason, even when off?

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
        // ADC Channel 8 (P5.0) = Joystick Y Axis
        // see page 96 of datasheet -> (P1Selx = 0b11)
        let mut joystick_x_pin = p1.pin5.to_alternate3();
        let mut joystick_y_pin = p5.pin0.to_alternate3();

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

        print_bytes(b"Configuring SPI...\n");

        // Launchpad lcd pins
        // P4.7: MISO
        // P4.6: MOSI
        // P4.5: SCLK
        // P4.4: CS
        // P3.2: rs
        let mut spi_config: SPIBusConfig<E_USCI_B1> =
            SPIBusConfig::new(periph.E_USCI_B1, MODE_0, true);
        spi_config.use_smclk(&smclk, 1);
        let periph_spi: SPIPins<E_USCI_B1> = spi_config.spi_pins(
            p4.pin7.to_alternate1(),
            p4.pin6.to_alternate1(),
            p4.pin5.to_alternate1(),
            p4.pin4.to_alternate1(),
        );
        unsafe {
            interrupt::enable();
        }

        print_bytes(b"Initializing LCD...\n");
        let lcd_rst = p4.pin0.to_output();
        let lcd_rs = p3.pin2.to_output();
        let mut screen = ST7735::new(periph_spi, lcd_rs, lcd_rst, false, false, 128, 128);
        screen.init(&mut delay).unwrap();
        screen.set_offset(2, 3);
        screen
            .set_orientation(&st7735_lcd::Orientation::PortraitSwapped)
            .ok();
        screen.clear(Rgb565::BLACK).unwrap();

        let mut btn_1_prev = false;

        loop {
            let btn_1 = btn_1_pin.is_low().unwrap();
            let btn_1_pressed = btn_1 && !btn_1_prev;
            btn_1_prev = btn_1;

            let joy_x: u16 = adc.read(&mut joystick_x_pin).unwrap();
            let mut joy_y: u16 = adc.read(&mut joystick_y_pin).unwrap();

            // invert y axis
            joy_y = 1023 - joy_y;

            let coord_x = joy_x / 10 + (128 - 102) / 2;
            let coord_y = joy_y / 10 + 20;

            screen
                .set_pixel(coord_x, coord_y, RawU16::from(Rgb565::WHITE).into_inner())
                .unwrap();

            if btn_1_pressed {
                screen.clear(Rgb565::BLACK).unwrap();
            }

            // turn on green LED if joystick pushed
            if joy_x > 1024 - 100 || joy_x < 100 || joy_y > 1024 - 100 || joy_y < 100 {
                led_g_pin.set_high().unwrap();
            } else {
                led_g_pin.set_low().unwrap();
            }
        }
    }
    loop {}
}

#[no_mangle]
extern "C" fn abort() -> ! {
    panic!();
}
