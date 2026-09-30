use super::{
    device::Vd6283,
    error::Result,
    regs,
    types::{AlsData, Channel, CHANNEL_COUNT},
};

const GAIN_RANGE:           [u16; 15] = [
    0x42ab, 0x3200, 0x2154, 0x1900, 0x10ab, 0x0a00, 0x0723, 0x0500, 0x0354, 0x0280, 0x01ab, 0x0140,
    0x0100, 0x00d4, 0x00b5,
];

const GAIN_THRESHOLDS:      [u16; 14] = [
    0x3a56, 0x29ab, 0x1d2b, 0x14d6, 0x0d56, 0x0892, 0x0612, 0x042b, 0x02eb, 0x0216, 0x0176, 0x0121,
    0x00eb, 0x00c5,
];

const EXPOSURE_STEP_US:     u32 = 1_600;
const EXPOSURE_MAX_STEPS:   u32 = 0x3ff;

impl<I2C> Vd6283<I2C>
where
    I2C: embedded_hal::i2c::I2c,
{
    pub fn set_gain(&mut self, channel: Channel, requested: u16) -> Result<u16> {
        self.ensure_configurable()?;
        let index = gain_index(requested);
        self.write_reg(regs::channel_vref_reg(channel.index()), (index + 1) as u8)?;
        self.gains[channel.index()] = GAIN_RANGE[index];
        Ok(self.gains[channel.index()])
    }

    pub fn gain(&self, channel: Channel) -> u16 {
        self.gains[channel.index()]
    }

    pub fn set_exposure_time(&mut self, requested_us: u32) -> Result<u32> {
        self.ensure_configurable()?;
        let steps = ((u64::from(requested_us) + u64::from(EXPOSURE_STEP_US / 2))
            / u64::from(EXPOSURE_STEP_US))
        .clamp(1, u64::from(EXPOSURE_MAX_STEPS)) as u32;

        self.write_reg(regs::REG_EXPOSURE_LSB, (steps & 0xff) as u8)?;
        self.write_reg(regs::REG_EXPOSURE_MSB, ((steps >> 8) & 0x03) as u8)?;
        self.exposure_us = steps * EXPOSURE_STEP_US;
        Ok(self.exposure_us)
    }

    pub const fn exposure_time_us(&self) -> u32 {
        self.exposure_us
    }

    pub(crate) fn apply_calibration(&self, als: &mut AlsData) {
        for channel in 0..CHANNEL_COUNT {
            if self.als_channels & (1 << channel) == 0 {
                continue;
            }
            let otp_gain = if self.otp_usage_enabled {
                self.otp.gains[channel]
            } else {
                0x80
            };
            let coefficient = calibration_factor_1050(self.gains[channel], otp_gain);
            let calibrated =
                (u64::from(coefficient) * u64::from(als.count_value_raw[channel])) >> 7;
            let calibrated = calibrated.min(0x00ff_ffff);
            als.count_value[channel] = if calibrated < 0x100 {
                0
            } else {
                calibrated as u32
            };
            als.gains[channel] = self.gains[channel];
        }
    }
}

fn gain_index(requested: u16) -> usize {
    let mut index = GAIN_RANGE.len() - 1;
    while index > 0 && requested >= GAIN_THRESHOLDS[index - 1] {
        index -= 1;
    }
    index
}

fn calibration_factor(gain: u8) -> u8 {
    if gain & 0x08 != 0 {
        0x80 | ((gain & 0x07) << 2)
    } else {
        (0x20u8.saturating_sub(gain)) << 2
    }
}

fn calibration_factor_1050(channel_gain: u16, gain: u8) -> u8 {
    if channel_gain < 0x2154 {
        calibration_factor(gain >> 4)
    } else {
        calibration_factor(gain & 0x0f)
    }
}
