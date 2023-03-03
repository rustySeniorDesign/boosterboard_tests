//! Usage example for st7735s driver

#![no_main]
#![no_std]

use core::panic::PanicInfo;
use msp430_rt::entry;
use msp430fr2355::{E_USCI_A1, E_USCI_B0, E_USCI_B1};
use msp430fr2x5x_hal::{
    clock::{ClockConfig, DcoclkFreqSel, MclkDiv, SmclkDiv, Aclk},
    fram::Fram,
    gpio::Batch,
    pmm::Pmm,
    serial::*,
    watchdog::Wdt,
    spi::{SPIPins},
    pac
};
use embedded_hal::digital::v2::OutputPin;
use embedded_hal::prelude::{
    _embedded_hal_blocking_delay_DelayMs,
    _embedded_hal_blocking_spi_Write
};
use embedded_hal::spi::MODE_0;
use msp430::interrupt;
use msp430fr2x5x_hal::spi::SPIBusConfig;
use msp430fr2355_boosterpack::{opt3001::DeviceOpt3001, serial_utils::{print_bytes, u32_to_dec, byte_to_dec, init_serial, u16_to_dec}, serial_utils, stream};
use st7735_lcd::ST7735;
use embedded_graphics::{
    image::Image,
    prelude::*,
    primitives::{Rectangle}
};
use embedded_graphics::pixelcolor::{Rgb565, RgbColor};
// use tinybmp::Bmp;
use msp430fr2355_boosterpack::stream::ImageContainer;
use msp430::asm;

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
            .mclk_dcoclk(DcoclkFreqSel::_16MHz, MclkDiv::_1)
            .smclk_on(SmclkDiv::_2)
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
        let mut spi_config : SPIBusConfig<E_USCI_B1> = SPIBusConfig::new(periph.E_USCI_B1, MODE_0, true);
        spi_config.use_smclk(&smclk, 5);
        let mut periph_spi : SPIPins<E_USCI_B1> = spi_config.spi_pins(
            p4.pin7.to_alternate1(),
            p4.pin6.to_alternate1(),
            p4.pin5.to_alternate1(),
            p4.pin4.to_alternate1()
        );
        unsafe{interrupt::enable();}
        print_bytes(b"Config successful\n\nInitializing screen...\n");
        let p3 = Batch::new(periph.P3).split(&pmm);
        let mut lcd_rst = p4.pin0.to_output();
        let mut lcd_rs = p3.pin2.to_output();
        // periph_spi.write(&[0xC4,0x51]).ok();
        let mut screen = ST7735::new(periph_spi, lcd_rs, lcd_rst, false, false, 128, 128);
        match screen.init(&mut delay) {
            Ok(_) => {
                screen.set_offset(2,3);
                screen.set_orientation(&st7735_lcd::Orientation::PortraitSwapped).ok();
                screen.clear(Rgb565::BLACK).ok();
                print_bytes(b"Screen initialized.\n");
                let mut img_buf = ImageContainer::new();
                let num_imgs = stream::get_num_images();
                print_bytes(&serial_utils::u16_to_hex(num_imgs));
                print_bytes(b" images available.\n");
                for idx in 0u16 .. num_imgs{
                    print_bytes(b"get img: ");
                    print_bytes(&serial_utils::u16_to_hex(idx));
                    print_bytes(b"\n");
                    asm::barrier();
                    img_buf.request_img(idx);
                    let rect = Rectangle::new(
                        Point::new(img_buf.x as i32, img_buf.y as i32),
                        Size::new(stream::SQUARE_LEN as u32, stream::SQUARE_LEN as u32)
                    );
                    screen.fill_contiguous(&rect, img_buf.colors).ok();
                }


                // let bmp_data = include_bytes!("../assets/rusty.bmp");
                // let bmp = Bmp::from_slice(bmp_data).unwrap();
                // Image::new(&bmp, Point::new(0, 0)).draw(&mut screen).ok();

                // loop {
                //     screen.clear(Rgb565::BLUE).ok();
                //     delay.delay_ms(1000u16);
                //     screen.clear(Rgb565::RED).ok();
                // }
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
    // panic!();
    loop {
        // Prevent optimizations that can remove this loop.
        msp430::asm::barrier();
    }

}



















