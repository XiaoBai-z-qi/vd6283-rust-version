use embedded_hal::i2c::I2c;

use super::{
    device::Vd6283,
    error::{Error, Result},
    regs::I2C_ADDRESS,
};

impl<I2C> Vd6283<I2C>
where
    I2C: I2c,
{
    pub(crate) fn write_reg(&mut self, register: u8, value: u8) -> Result<()> {
        self.i2c
            .write(I2C_ADDRESS, &[register, value])
            .map_err(|_| Error::I2cWrite)
    }

    pub(crate) fn read_reg(&mut self, register: u8) -> Result<u8> {
        let mut value = [0u8; 1];
        self.i2c
            .write_read(I2C_ADDRESS, &[register], &mut value)
            .map_err(|_| Error::I2cRead)?;
        Ok(value[0])
    }

    pub(crate) fn read_regs(&mut self, register: u8, buffer: &mut [u8]) -> Result<()> {
        self.i2c
            .write_read(I2C_ADDRESS, &[register], buffer)
            .map_err(|_| Error::I2cRead)
    }
}
