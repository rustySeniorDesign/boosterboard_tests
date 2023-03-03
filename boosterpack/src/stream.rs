use embedded_graphics::pixelcolor::raw::RawU16;
use embedded_graphics::pixelcolor::Rgb565;
use embedded_graphics::prelude::RgbColor;
use msp430::asm;
use crate::serial_utils;

enum Command{
    GetNumImg = 0x1,
    GetImg = 0x2,
    ESCAPED = 0xFF,
}

impl From<Command> for u8 {
    fn from(value: Command) -> Self {
        value as u8
    }
}

enum ImageColors {

}

pub struct  ImageContainer {
    pub x: u8,
    pub y: u8,
    pub colors: [Rgb565;64]
}

fn to_u8(num: u16) -> [u8;2]{
    [(num & 0x00FF) as u8, ((num&0xFF00)>>8) as u8]
}

#[inline]
fn to_u16(arr: &[u8]) -> u16{
    ((arr[1] as u16) << 8) | (arr[0] as u16)
}

impl ImageContainer{
    pub fn new() -> Self{
        ImageContainer {
            x: 0,
            y: 0,
            colors: [Rgb565::BLACK;64]
        }
    }

    pub fn request_img(&mut self, num: u16){
        let split = to_u8(num);
        serial_utils::print_bytes(&[0xFFu8, Command::GetImg.into(), split[0], split[1]]);
        let mut byte_buf = [0u8];

        serial_utils::get_bytes(&mut byte_buf).expect("a");
        serial_utils::print_bytes(&[0xAAu8]);
        self.x = byte_buf[0];
        serial_utils::get_bytes(&mut byte_buf).expect("b");
        serial_utils::print_bytes(&[0xAAu8]);
        self.y = byte_buf[0];
        let mut rd_buf = [0u8;8];
        for i in (0..self.colors.len()).step_by(4){
            serial_utils::print_bytes(&[0xAAu8]);
            serial_utils::get_bytes(&mut rd_buf).expect("c");
            self.colors[i] = Rgb565::from(RawU16::new(to_u16(&rd_buf[0..2])));
            self.colors[i+1] = Rgb565::from(RawU16::new(to_u16(&rd_buf[2..4])));
            self.colors[i+2] = Rgb565::from(RawU16::new(to_u16(&rd_buf[4..6])));
            self.colors[i+3] = Rgb565::from(RawU16::new(to_u16(&rd_buf[6..8])));
            // serial_utils::print_bytes(&[0xAAu8]);
            // serial_utils::print_bytes(&[0xAAu8]);
        }
    }
}

pub fn get_num_images() -> u16{
    let mut rd_buf = [0u8;2];
    serial_utils::print_bytes(&[0xFFu8, Command::GetNumImg.into()]);
    serial_utils::get_bytes(&mut rd_buf).ok();
    to_u16(&rd_buf)
}