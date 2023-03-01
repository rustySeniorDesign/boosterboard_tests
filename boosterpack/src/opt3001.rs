//! Driver for the OPT3001 Ambient Light Sensor
use embedded_hal::prelude::{_embedded_hal_blocking_i2c_Read, _embedded_hal_blocking_i2c_Write};
use msp430fr2x5x_hal::i2c::{EUsciI2CBus, I2CErr, SDL};


static OPT3001_ADDRESS:u8 = 0x44;

/// For interacting with a OPT3001 sensor
pub struct DeviceOpt3001<USCI: EUsciI2CBus>{
    i2c_pin: SDL<USCI>,
    address: u8,
    active_reg: u8,
}

/// Sensor result converted into units of lux
pub struct Lux{
    /// integer component of lux measurement, range: 83865-0
    pub whole:u32,
    /// fractional component of lux measurement, range: 99-0
    pub frac: u8
}

#[inline]
fn reading_to_lux(val: u16) -> Lux{
    let exp = (val & 0xF000) >> 12;
    let lsb_size : u32 = 0x1 << exp;
    let lux_raw : u32 = lsb_size * ((val & 0x0FFF) as u32);
    let whole = lux_raw / 100;
    Lux{
        whole,
        frac: ((lux_raw - (100 * whole)) as u8)
    }
}

impl<USCI: EUsciI2CBus> DeviceOpt3001<USCI> {
    pub fn new(mut sdl_pin: SDL<USCI>) -> Result<DeviceOpt3001<USCI>, I2CErr>{
        // address:  1000100 (0x44)
        // config reg (0x1):
        // 15:12 : 1100
        // 11 : 0
        // 10:9 : 10
        // 8:5 : dc
        // 4 : 1
        // 3 : 0
        // 2 : 0
        // 1:0 : 00
        let config_cmd: [u8; 3] = [0x1, 0xC4, 0x10];
        let res : Result<(), I2CErr> = sdl_pin.write(OPT3001_ADDRESS, &config_cmd)
            .and_then(|_| {sdl_pin.write(OPT3001_ADDRESS, &[0x00u8])});
        return match res {
            Ok(()) => {
                Ok(DeviceOpt3001 {
                    i2c_pin: sdl_pin,
                    address: OPT3001_ADDRESS,
                    active_reg: 0,
                })
            }
            Err(err) => {
                Err(err)
            }
        }
    }

    /// Blocking I2C read to get light value from sensor
    pub fn read_light(&mut self) -> Result<Lux, I2CErr>{
        if self.active_reg != 0 {
            match self.i2c_pin.write(self.address, &[0x0u8]) {
                Ok(_) => {self.active_reg = 0;}
                Err(err) => {return Err(err)},
            }
        }
        let mut read_buf : [u8;2] = [0,0];
        return match self.i2c_pin.read(self.address, &mut read_buf) {
            Ok(_) => {
                Ok(reading_to_lux(((read_buf[1] as u16) << 8) | (read_buf[0] as u16)))
            }
            Err(err) => { Err(err) }
        }
    }
}


