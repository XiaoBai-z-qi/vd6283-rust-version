use super::{
    device::Vd6283,
    error::{Error, Result},
    regs,
};

impl<I2C> Vd6283<I2C>
where
    I2C: embedded_hal::i2c::I2c,
{
    pub fn pedestal_enabled(&mut self) -> Result<bool> {
        Ok(self.read_reg(regs::REG_AC_MODE)? & regs::AC_PEDESTAL == 0)
    }

    pub fn set_pedestal_enabled(&mut self, enabled: bool) -> Result<()> {
        self.ensure_configurable()?;
        self.update_ac_mode(
            regs::AC_PEDESTAL,
            if enabled {
                regs::AC_PEDESTAL_ENABLE
            } else {
                regs::AC_PEDESTAL_DISABLE
            },
        )
    }

    pub fn pedestal_value(&mut self) -> Result<u8> {
        Ok(self.read_reg(regs::REG_AC_PEDESTAL)? & 0x07)
    }

    pub fn set_pedestal_value(&mut self, value: u8) -> Result<()> {
        self.ensure_configurable()?;
        self.write_reg(regs::REG_AC_PEDESTAL, value & 0x07)
    }

    pub const fn otp_usage_enabled(&self) -> bool {
        self.otp_usage_enabled
    }

    pub fn set_otp_usage_enabled(&mut self, enabled: bool) -> Result<()> {
        self.ensure_configurable()?;
        self.otp_usage_enabled = enabled;
        let hf_trim = if enabled { self.otp.hf_trim } else { 0x0e3 };
        let lf_trim = if enabled { self.otp.lf_trim } else { 0x07 };
        self.write_reg(
            regs::REG_OSCILLATOR_10M_TRIM_MSB,
            ((hf_trim >> 8) & 1) as u8,
        )?;
        self.write_reg(regs::REG_OSCILLATOR_10M_TRIM_LSB, hf_trim as u8)?;
        self.write_reg(regs::REG_OSCILLATOR_50K_TRIM, lf_trim & 0x0f)
    }

    pub const fn output_dark_enabled(&self) -> bool {
        self.output_dark_enabled
    }

    pub fn set_output_dark_enabled(&mut self, enabled: bool) -> Result<()> {
        self.ensure_configurable()?;
        self.write_reg(regs::select_pd_reg(1), if enabled { 0x18 } else { 0x07 })?;
        self.output_dark_enabled = enabled;
        Ok(())
    }

    pub fn set_sda_drive_ma(&mut self, milliamps: u8) -> Result<()> {
        self.ensure_configurable()?;
        if !(4..=20).contains(&milliamps) || milliamps % 4 != 0 {
            return Err(Error::InvalidParams);
        }
        let mut value = self.read_reg(regs::REG_SDA_DRIVE)?;
        value = (value & !regs::SDA_DRIVE_MASK) | ((milliamps / 4) - 1);
        self.write_reg(regs::REG_SDA_DRIVE, value)
    }

    pub fn sda_drive_ma(&mut self) -> Result<u8> {
        let value = self.read_reg(regs::REG_SDA_DRIVE)? & regs::SDA_DRIVE_MASK;
        Ok((value + 1) * 4)
    }

    pub fn saturation_value(&mut self) -> Result<u16> {
        loop {
            let before = self.read_reg(regs::REG_AC_SATURATION_MSB)?;
            let low = self.read_reg(regs::REG_AC_SATURATION_LSB)?;
            let after = self.read_reg(regs::REG_AC_SATURATION_MSB)?;
            if before == after {
                return Ok(u16::from_be_bytes([before, low]));
            }
        }
    }

    pub(crate) fn update_ac_mode(&mut self, mask: u8, value: u8) -> Result<()> {
        let current = self.read_reg(regs::REG_AC_MODE)?;
        self.write_reg(regs::REG_AC_MODE, (current & !mask) | (value & mask))
    }
}
