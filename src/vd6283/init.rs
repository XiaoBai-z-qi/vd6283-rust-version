use super::{
    device::Vd6283,
    error::{Error, Result},
    regs,
    types::{DeviceState, CHANNEL_COUNT},
};

const DEFAULT_HF_TRIM:  u16 = 0x0e3;
const DEFAULT_LF_TRIM:  u8 = 0x07;
const DEFAULT_VREF:     u8 = 13;

impl<I2C> Vd6283<I2C>
where
    I2C: embedded_hal::i2c::I2c,
{
    pub fn init(&mut self) -> Result<()> {
        if self.state != DeviceState::Free {
            return Err(Error::AlreadyStarted);
        }

        self.state = DeviceState::Init;
        let result = self.init_inner();
        if result.is_err() {
            self.state = DeviceState::Free;
        }
        result
    }

    fn init_inner(&mut self) -> Result<()> {
        self.device_id = self.read_reg(regs::REG_DEVICE_ID)?;
        self.revision_id = self.read_reg(regs::REG_REVISION_ID)?;
        if self.device_id != regs::DEVICE_ID || self.revision_id != regs::REVISION_ID {
            return Err(Error::InvalidDeviceId);
        }

        self.software_reset()?;
        self.otp_reset()?;
        self.otp_read_init()?;
        self.otp_read_param()?;
        self.otp_generate_uid()?;
        if self.otp.version != 0x15 {
            return Err(Error::InvalidDeviceId);
        }

        self.configure_oscillators()?;
        self.configure_clamps()?;
        self.configure_photodiodes()?;
        self.configure_default_gains()?;
        self.write_reg(regs::REG_AC_PEDESTAL, 3)?;
        self.configure_dithering()?;
        self.write_reg(regs::REG_SPARE_0, regs::SPARE_0_INPUT_GPIO2)?;

        Ok(())
    }

    pub(crate) fn software_reset(&mut self) -> Result<()> {
        self.write_reg(regs::REG_GLOBAL_RESET, 1)?;
        self.write_reg(regs::REG_GLOBAL_RESET, 0)
    }

    fn configure_oscillators(&mut self) -> Result<()> {
        let hf_trim = if self.otp_usage_enabled {
            self.otp.hf_trim
        } else {
            DEFAULT_HF_TRIM
        };
        let lf_trim = if self.otp_usage_enabled {
            self.otp.lf_trim
        } else {
            DEFAULT_LF_TRIM
        };

        self.write_reg(regs::REG_OSCILLATOR_10M, 1)?;
        self.write_reg(
            regs::REG_OSCILLATOR_10M_TRIM_MSB,
            ((hf_trim >> 8) & 1) as u8,
        )?;
        self.write_reg(regs::REG_OSCILLATOR_10M_TRIM_LSB, hf_trim as u8)?;
        self.write_reg(regs::REG_OSCILLATOR_50K_TRIM, lf_trim & 0x0f)
    }

    fn configure_clamps(&mut self) -> Result<()> {
        self.write_reg(regs::REG_AC_CLAMP_ENABLE, 1)?;
        self.write_reg(regs::REG_DC_CLAMP_ENABLE, 0x1f)?;
        self.write_reg(regs::REG_SPARE_1, 0x3f)
    }

    fn configure_photodiodes(&mut self) -> Result<()> {
        let mut pd_values = [0x07, 0x07, 0x07, 0x0d, 0x0f, 0x1f];
        for (channel, value) in pd_values.iter_mut().enumerate() {
            self.write_reg(regs::select_pd_reg(channel), *value)?;
        }
        Ok(())
    }

    fn configure_default_gains(&mut self) -> Result<()> {
        for channel in 0..CHANNEL_COUNT {
            self.write_reg(regs::channel_vref_reg(channel), DEFAULT_VREF)?;
            self.gains[channel] = 0x0100;
        }
        Ok(())
    }

    fn configure_dithering(&mut self) -> Result<()> {
        self.write_reg(regs::REG_DITHERING_STEP_VALUE, 1)?;
        self.write_reg(regs::REG_DITHERING_STEP_DURATION, 1)?;
        self.write_reg(regs::REG_DITHERING_STEP_NUMBER, 0x09)?;
        self.write_reg(regs::REG_DITHERING_CONTROL, 1)
    }

    pub fn terminate(&mut self) -> Result<()> {
        if self.state == DeviceState::Free {
            return Err(Error::InvalidDeviceId);
        }

        let reset_result = self.software_reset();
        let otp_result = self.otp_reset();
        let spare_result = self.write_reg(regs::REG_SPARE_0, regs::SPARE_0_INPUT_GPIO2);
        self.state = DeviceState::Free;
        reset_result.and(otp_result).and(spare_result)
    }
}
