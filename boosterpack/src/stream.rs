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

pub const SQUARE_LEN : usize = 16;
const BUF_SIZE : usize = SQUARE_LEN*SQUARE_LEN;

pub struct  ImageContainer {
    pub x: u8,
    pub y: u8,
    pub colors: [Rgb565;BUF_SIZE]
}

fn to_u8(num: u16) -> [u8;2]{
    [(num & 0x00FF) as u8, ((num&0xFF00)>>8) as u8]
}

#[inline]
fn to_u16(arr: &[u8]) -> u16{
    ((arr[1] as u16) << 8) | (arr[0] as u16)
}

#[inline]
fn to_u16_msb(arr: &[u8]) -> u16{
    ((arr[0] as u16) << 8) | (arr[1] as u16)
}

impl ImageContainer{
    pub fn new() -> Self{
        ImageContainer {
            x: 0,
            y: 0,
            colors: [Rgb565::BLACK;BUF_SIZE]
        }
    }

    pub fn request_img(&mut self, num: u16){
        let split = to_u8(num);
        serial_utils::print_bytes(&[0xFFu8, Command::GetImg.into(), split[0], split[1]]);
        let mut byte_buf = [0u8];

        serial_utils::get_bytes(&mut byte_buf).ok();
        serial_utils::print_bytes(&[0xAAu8]);
        self.x = byte_buf[0];
        serial_utils::get_bytes(&mut byte_buf).ok();
        self.y = byte_buf[0];
        let mut rd_buf = [0u8;256];
        for i in (0..self.colors.len()).step_by(128){
            serial_utils::print_bytes(&[0xAAu8]);
            serial_utils::get_bytes(&mut rd_buf).ok();
            for j in (0..256).step_by(2){
                let idx= j >> 1;
                self.colors[i+idx] = Rgb565::from(RawU16::new(to_u16(&rd_buf[j..j+2])));
            }
        }
    }
}

pub fn get_num_images() -> u16{
    let mut rd_buf = [0u8;2];
    serial_utils::print_bytes(&[0xFFu8, Command::GetNumImg.into()]);
    serial_utils::get_bytes(&mut rd_buf).ok();
    to_u16(&rd_buf)
}