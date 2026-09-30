pub const CHANNEL_COUNT:    usize = 6;

pub const UID_LEN:          usize = 16;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(u8)]
pub enum Channel {
    Ch1 = 0x01,
    Ch2 = 0x02,
    Ch3 = 0x04,
    Ch4 = 0x08,
    Ch5 = 0x10,
    Ch6 = 0x20,
}

impl Channel {
    pub const fn index(self) -> usize {
        match self {
            Self::Ch1 => 0,
            Self::Ch2 => 1,
            Self::Ch3 => 2,
            Self::Ch4 => 3,
            Self::Ch5 => 4,
            Self::Ch6 => 5,
        }
    }

    pub const fn mask(self) -> u8 {
        self as u8
    }

    pub const fn from_mask(mask: u8) -> Option<Self> {
        match mask {
            0x01 => Some(Self::Ch1),
            0x02 => Some(Self::Ch2),
            0x04 => Some(Self::Ch3),
            0x08 => Some(Self::Ch4),
            0x10 => Some(Self::Ch5),
            0x20 => Some(Self::Ch6),
            _ => None,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(u8)]
pub enum Color {
    Ir = 0x01,
    Red = 0x02,
    Green = 0x03,
    Blue = 0x04,
    Uv = 0x05,
    Clear = 0x06,
    ClearIrCut = 0x07,
    Dark = 0x08,
    Invalid = 0xff,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(u8)]
pub enum Mode {
    AlsSingleShot = 0,
    AlsSynchronous = 1,
    Flicker = 2,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(u8)]
pub enum FlickerOutput {
    DigitalPdm = 1,
    Analog = 2,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DeviceState {
    Free,
    Init,
    AlsRunning,
    FlickerRunning,
    BothRunning,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct AlsData {
    pub channels:           u8,
    pub count_value:        [u32; CHANNEL_COUNT],
    pub count_value_raw:    [u32; CHANNEL_COUNT],
    pub gains:              [u16; CHANNEL_COUNT],
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct FlickerInfo {
    pub frequency_hz:           u32,
    pub confidence_level:       u8,
    pub measurement_finished:   bool,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct OtpData {
    pub hf_trim:        u16,
    pub lf_trim:        u8,
    pub filter_config:  u8,
    pub filter_index:   u8,
    pub gains:          [u8; CHANNEL_COUNT],
    pub version:        u8,
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct LuxCct {
    pub x:      f32,
    pub y:      f32,
    pub z:      f32,
    pub lux:    f32,
    pub cct:    f32,
}

pub const CHANNEL_COLORS: [Color; CHANNEL_COUNT] = [
    Color::Red,
    Color::ClearIrCut,
    Color::Blue,
    Color::Green,
    Color::Ir,
    Color::Clear,
];
