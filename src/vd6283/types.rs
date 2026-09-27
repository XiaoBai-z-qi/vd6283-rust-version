//! VD6283 驱动使用的公共数据类型。

/// VD6283 的 ALS 通道数量。
pub const CHANNEL_COUNT: usize = 6;

/// 设备 UID 缓冲区预留长度。
pub const UID_LEN: usize = 16;

/// 一个 VD6283 通道。
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
    /// 返回该通道在数组中的下标（0 到 5）。
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

    /// 返回该通道在通道掩码中对应的 bit。
    pub const fn mask(self) -> u8 {
        self as u8
    }

    /// 将只包含一个通道 bit 的掩码转换为通道枚举。
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

/// 一个 ALS 通道上安装的颜色滤波片。
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

/// ALS 或 Flicker 工作模式。
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(u8)]
pub enum Mode {
    AlsSingleShot = 0,
    AlsSynchronous = 1,
    Flicker = 2,
}

/// Flicker 输出方式。
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(u8)]
pub enum FlickerOutput {
    DigitalPdm = 1,
    Analog = 2,
}

/// 设备状态机当前状态。
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DeviceState {
    Free,
    Init,
    AlsRunning,
    FlickerRunning,
    BothRunning,
}

/// 六个通道的原始值和校准值。
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct AlsData {
    /// 本次结果包含的通道掩码。
    pub channels: u8,
    /// 校准后的计数值。
    pub count_value: [u32; CHANNEL_COUNT],
    /// 从器件读取的原始计数值。
    pub count_value_raw: [u32; CHANNEL_COUNT],
    /// 各通道测量使用的增益，采用 8.8 定点格式。
    pub gains: [u16; CHANNEL_COUNT],
}

/// Flicker 测量结果。
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct FlickerInfo {
    /// 检测到的主频，单位为 Hz。
    pub frequency_hz: u32,
    /// 测量算法给出的置信度。
    pub confidence_level: u8,
    /// 测量是否已经完成。
    pub measurement_finished: bool,
}

/// 初始化时从 OTP 缓存的参数。
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct OtpData {
    pub hf_trim: u16,
    pub lf_trim: u8,
    pub filter_config: u8,
    pub filter_index: u8,
    pub gains: [u8; CHANNEL_COUNT],
    pub version: u8,
}

/// RGB 转 XYZ 以及 Lux/CCT 计算结果。
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct LuxCct {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub lux: f32,
    pub cct: f32,
}

/// 当前板上滤波片配置对应的通道颜色映射。
pub const CHANNEL_COLORS: [Color; CHANNEL_COUNT] = [
    Color::Red,
    Color::ClearIrCut,
    Color::Blue,
    Color::Green,
    Color::Ir,
    Color::Clear,
];
