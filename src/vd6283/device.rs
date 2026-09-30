use super::types::{DeviceState, FlickerOutput, OtpData, CHANNEL_COUNT, UID_LEN};

pub struct Vd6283<I2C> {
    pub(crate) i2c:                 I2C,
    pub(crate) state:               DeviceState,
    pub(crate) device_id:           u8,
    pub(crate) revision_id:         u8,
    pub(crate) dc_channels_enabled: u8,
    pub(crate) ac_enabled:          bool,
    pub(crate) als_channels:        u8,
    pub(crate) flicker_channels:    u8,
    pub(crate) als_mode:            Option<super::types::Mode>,
    pub(crate) gains:               [u16; CHANNEL_COUNT],
    pub(crate) exposure_us:         u32,
    pub(crate) flicker_output:      FlickerOutput,
    pub(crate) otp_usage_enabled:   bool,
    pub(crate) output_dark_enabled: bool,
    pub(crate) otp_bits:            [u64; 2],
    pub(crate) otp:                 OtpData,
    pub(crate) uid:                 [u8; UID_LEN],
}

impl<I2C> Vd6283<I2C> {
    pub fn new(i2c: I2C) -> Self {
        Self {
            i2c,
            state:                  DeviceState::Free,
            device_id:              0,
            revision_id:            0,
            dc_channels_enabled:    0,
            ac_enabled:             false,
            als_channels:           0,
            flicker_channels:       0,
            als_mode:               None,
            gains:                  [0; CHANNEL_COUNT],
            exposure_us:            80_000,
            flicker_output:         FlickerOutput::Analog,
            otp_usage_enabled:      true,
            output_dark_enabled:    false,
            otp_bits:               [0; 2],
            otp:                    OtpData::default(),
            uid:                    [0; UID_LEN],
        }
    }

    pub fn into_inner(self) -> I2C {
        self.i2c
    }

    pub const fn state(&self) -> DeviceState {
        self.state
    }

    pub const fn device_id(&self) -> u8 {
        self.device_id
    }

    pub const fn revision_id(&self) -> u8 {
        self.revision_id
    }

    pub fn uid(&self) -> &[u8; UID_LEN] {
        &self.uid
    }
}

impl<I2C> Vd6283<I2C> {
    pub(crate) fn ensure_configurable(&self) -> super::error::Result<()> {
        match self.state {
            DeviceState::Init => Ok(()),
            DeviceState::Free => Err(super::error::Error::InvalidDeviceId),
            DeviceState::AlsRunning | DeviceState::FlickerRunning | DeviceState::BothRunning => {
                Err(super::error::Error::AlreadyStarted)
            }
        }
    }
}
