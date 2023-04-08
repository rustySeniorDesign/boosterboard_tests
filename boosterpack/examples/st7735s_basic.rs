//! Usage example for st7735s driver

#![no_main]
#![no_std]

use core::panic::PanicInfo;
use embedded_hal::prelude::{
    _embedded_hal_blocking_delay_DelayMs
};
use embedded_hal::spi::{MODE_0};
use embedded_graphics::{
    prelude::*,
    pixelcolor::{Rgb565, RgbColor}
};
use msp430::{interrupt};
use msp430_rt::entry;
use msp430fr2355::{E_USCI_B1};
use msp430fr2355_boosterpack::{
    serial_utils::{print_bytes, init_serial},
    serial_utils,
    stream,
};
use msp430fr2x5x_hal::{
    clock::{ClockConfig, DcoclkFreqSel, MclkDiv, SmclkDiv},
    fram::Fram,
    gpio::Batch,
    pmm::Pmm,
    serial::*,
    watchdog::Wdt,
    spi::{SPIPins, SPIBusConfig}
};
use st7735_lcd::ST7735;
use msp430fr2355_boosterpack::stream::request_stream;


#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    msp430::interrupt::disable();
    print_bytes(b"Panic handler was called.\n");
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
        let (smclk, _aclk, mut delay) = ClockConfig::new(periph.CS)
            .mclk_dcoclk(DcoclkFreqSel::_16MHz, MclkDiv::_2)
            .smclk_on(SmclkDiv::_1)
            .aclk_refoclk()
            .freeze(&mut fram);

        let pmm = Pmm::new(periph.PMM);
        let p4 = Batch::new(periph.P4).split(&pmm);
        let (tx, rx) = SerialConfig::new(
            periph.E_USCI_A1,
            BitOrder::LsbFirst,
            BitCount::EightBits,
            StopBits::OneStopBit,
            Parity::NoParity,
            Loopback::NoLoop,
            256000,
        )
        .use_smclk(&smclk)
        .split(p4.pin3.to_alternate1(), p4.pin2.to_alternate1());
        init_serial(rx, tx);

        print_bytes(b"Serial started\n\nConfiguring USCI B1 for SPI...\n");

        // Launchpad lcd pins
        // P4.7: MISO
        // P4.6: MOSI
        // P4.5: SCLK
        // P4.4: CS
        // P3.2: rs
        let mut spi_config : SPIBusConfig<E_USCI_B1> =
            SPIBusConfig::new(periph.E_USCI_B1, MODE_0, true);
        spi_config.use_smclk(&smclk, 10);
        let periph_spi : SPIPins<E_USCI_B1> = spi_config.spi_pins(
            p4.pin7.to_alternate1(),
            p4.pin6.to_alternate1(),
            p4.pin5.to_alternate1(),
            p4.pin4.to_alternate1()
        );
        unsafe{interrupt::enable();}
        print_bytes(b"Config successful\n\nInitializing screen...\n");
        let p3 = Batch::new(periph.P3).split(&pmm);
        let lcd_rst = p4.pin0.to_output();
        let lcd_rs = p3.pin2.to_output();
        // periph_spi.write(&[0xC4,0x51]).ok();
        // stream::init_stream(periph_spi);
        let mut screen = ST7735::new(periph_spi, lcd_rs, lcd_rst, false, false, 128, 128);
        match screen.init(&mut delay) {
            Ok(_) => {
                screen.set_offset(2,3);
                screen.set_orientation(&st7735_lcd::Orientation::PortraitSwapped).ok();
                screen.clear(Rgb565::BLACK).ok();
                print_bytes(b"Screen initialized.\n");
                let num_imgs = stream::get_num_images();
                print_bytes(&serial_utils::u16_to_hex(num_imgs));
                print_bytes(b" images available.\nGetting images...\n");
                for idx in 0u16 .. num_imgs{
                    print_bytes(b"get img: ");
                    print_bytes(&serial_utils::u16_to_hex(idx));
                    print_bytes(b"\n");
                    stream::request_img(idx, &mut screen);
                    delay.delay_ms(100u16);
                }
                delay.delay_ms(2000u16);
                print_bytes(b"Image transfer complete\n");
                screen.clear(Rgb565::BLACK).ok();
                loop {
                    request_stream(&mut screen);
                    delay.delay_ms(10u16);
                }
            }
            Err(_) => {
                print_bytes(b"Screen init failed.\n")
            }
        }
    }
    loop{}
}

#[no_mangle]
extern "C" fn abort() -> ! {
    panic!();
}



















