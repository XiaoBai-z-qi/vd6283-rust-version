pub const I2C_ADDRESS:                  u8 = 0x40;

pub const DEVICE_ID:                    u8 = 0x70;
pub const REVISION_ID:                  u8 = 0xbd;

pub const REG_DEVICE_ID:                u8 = 0x00;
pub const REG_REVISION_ID:              u8 = 0x01;
pub const REG_IRQ_CTRL_STATUS:          u8 = 0x02;
pub const REG_ALS_CONTROL:              u8 = 0x03;
pub const REG_CONTINUOUS_PERIOD:        u8 = 0x04;

pub const ALS_START:                    u8 = 1 << 0;
pub const ALS_CONTINUOUS:               u8 = 1 << 1;
pub const ALS_CONTINUOUS_SLAVED:        u8 = 1 << 2;

pub const REG_CHANNEL_BASE:             u8 = 0x05;
pub const REG_EXPOSURE_MSB:             u8 = 0x1d;
pub const REG_EXPOSURE_LSB:             u8 = 0x1e;
pub const REG_CHANNEL_VREF_BASE:        u8 = 0x25;

pub const DC_CHANNELS_MASK:             u8 = 0x1f;
pub const AC_CHANNELS_MASK:             u8 = 0x20;

pub const REG_AC_ENABLE:                u8 = 0x2d;
pub const REG_DC_ENABLE:                u8 = 0x2e;
pub const REG_AC_CLAMP_ENABLE:          u8 = 0x2f;
pub const REG_DC_CLAMP_ENABLE:          u8 = 0x30;
pub const REG_AC_MODE:                  u8 = 0x31;
pub const REG_AC_PEDESTAL:              u8 = 0x32;
pub const REG_AC_SATURATION_MSB:        u8 = 0x33;
pub const REG_AC_SATURATION_LSB:        u8 = 0x34;

pub const AC_EXTRACTOR:                 u8 = 1 << 0;
pub const AC_CHANNEL_SELECT_MASK:       u8 = 0x0e;
pub const AC_PEDESTAL:                  u8 = 1 << 6;
pub const AC_EXTRACTOR_ENABLE:          u8 = 1 << 0;
pub const AC_EXTRACTOR_DISABLE:         u8 = 0;
pub const AC_PEDESTAL_ENABLE:           u8 = 0;
pub const AC_PEDESTAL_DISABLE:          u8 = 1 << 6;

pub const PDM_SELECT_OUTPUT:            u8 = 1 << 4;
pub const PDM_SELECT_CLOCK:             u8 = 1 << 5;
pub const PDM_OUTPUT_GPIO1:             u8 = 0;
pub const PDM_OUTPUT_GPIO2:             u8 = 1 << 4;
pub const PDM_CLOCK_INTERNAL:           u8 = 0;
pub const PDM_CLOCK_EXTERNAL:           u8 = 1 << 5;

pub const REG_SDA_DRIVE:                u8 = 0x3c;
pub const SDA_DRIVE_MASK:               u8 = 0x07;
pub const REG_OSCILLATOR_10M:           u8 = 0x3d;
pub const REG_OSCILLATOR_10M_TRIM_MSB:  u8 = 0x3e;
pub const REG_OSCILLATOR_10M_TRIM_LSB:  u8 = 0x3f;
pub const REG_OSCILLATOR_50K_TRIM:      u8 = 0x40;
pub const REG_INTERRUPT_CONFIG:         u8 = 0x41;
pub const REG_DTEST_SELECTION:          u8 = 0x47;

pub const REG_OTP_CONTROL_1:            u8 = 0x58;
pub const REG_OTP_STATUS:               u8 = 0x5a;
pub const REG_OTP_BANK_0:               u8 = 0x5b;
pub const REG_OTP_BANK_1:               u8 = 0x63;
pub const OTP_BANK_0_READY:             u8 = 1 << 1;
pub const OTP_BANK_1_READY:             u8 = 1 << 3;
pub const OTP_DATA_READY:               u8 = OTP_BANK_0_READY | OTP_BANK_1_READY;

pub const REG_SELECT_PD_BASE:           u8 = 0x6b;

pub const REG_SPARE_0:                  u8 = 0x71;
pub const REG_SPARE_1:                  u8 = 0x72;
pub const SPARE_0_OUTPUT_GPIO2:         u8 = 0;
pub const SPARE_0_INPUT_GPIO2:          u8 = 1 << 0;
pub const SPARE_0_ZERO_CROSS_GPIO2:     u8 = 1 << 1;
pub const SPARE_0_ZERO_CROSS_GPIO1:     u8 = 1 << 2;
pub const REG_DITHERING_CONTROL:        u8 = 0x76;
pub const REG_DITHERING_STEP_VALUE:     u8 = 0x77;
pub const REG_DITHERING_STEP_DURATION:  u8 = 0x78;
pub const REG_DITHERING_STEP_NUMBER:    u8 = 0x79;

pub const REG_GLOBAL_RESET:             u8 = 0xfe;

pub const fn channel_data_reg(channel: usize) -> u8 {
    REG_CHANNEL_BASE + (channel as u8) * 4
}

pub const fn channel_vref_reg(channel: usize) -> u8 {
    REG_CHANNEL_VREF_BASE + channel as u8
}

pub const fn select_pd_reg(channel: usize) -> u8 {
    REG_SELECT_PD_BASE + channel as u8
}
