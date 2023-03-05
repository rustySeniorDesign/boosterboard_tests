//! Usage example for st7735s driver

#![no_main]
#![no_std]

use core::panic::PanicInfo;
use embedded_hal::digital::v2::OutputPin;
use embedded_hal::prelude::{
    _embedded_hal_blocking_delay_DelayMs,
    _embedded_hal_blocking_spi_Write,
};
use embedded_hal::spi::MODE_0;
use embedded_graphics::{
    image::Image,
    prelude::*,
    primitives::{Rectangle},
    pixelcolor::{Rgb565, RgbColor}
};
use msp430::{asm, interrupt};
use msp430_rt::entry;
use msp430fr2355::{E_USCI_A1, E_USCI_B0, E_USCI_B1};
use msp430fr2355_boosterpack::{
    opt3001::DeviceOpt3001,
    serial_utils::{print_bytes, u32_to_dec, byte_to_dec, init_serial, u16_to_dec},
    serial_utils,
    stream,
};
use msp430fr2x5x_hal::{
    clock::{ClockConfig, DcoclkFreqSel, MclkDiv, SmclkDiv, Aclk},
    fram::Fram,
    gpio::Batch,
    pmm::Pmm,
    serial::*,
    watchdog::Wdt,
    spi::{SPIPins, SPIBusConfig},
    pac
};
use st7735_lcd::ST7735;


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
    print_bytes(b"Panic handler was called.\n");
    // }
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
        let (tx, mut rx) = SerialConfig::new(
            periph.E_USCI_A1,
            BitOrder::LsbFirst,
            BitCount::EightBits,
            StopBits::OneStopBit,
            Parity::NoParity,
            Loopback::NoLoop,
            115200,
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
        let mut periph_spi : SPIPins<E_USCI_B1> = spi_config.spi_pins(
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
                    delay.delay_ms(1000u16);
                    // screen.clear(Rgb565::BLUE).ok();
                    // screen.clear(Rgb565::BLUE).ok();
                    // let rect = Rectangle::new(
                    //     Point::new(img_buf.x as i32, img_buf.y as i32),
                    //     Size::new(stream::SQUARE_LEN as u32, stream::SQUARE_LEN as u32)
                    // );
                    // screen.fill_contiguous(&rect, img_buf.colors).ok();
                }
                // screen.clear(Rgb565::BLACK).ok();
                print_bytes(b"Image transfer complete\n");
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



















