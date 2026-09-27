pub mod error;
pub mod regs;
pub mod types;
pub mod device;
pub mod bus;
pub mod init;
pub mod otp;
pub mod control;
pub mod gain;
pub mod mode;
pub mod lux;

pub use device::Vd6283;
pub use error::{Error, Result};
pub use lux::{get_lux_cct, normalize_als, LUX_REFERENCE_EXPOSURE_US};
pub use types::{
    AlsData, Channel, Color, DeviceState, 
    FlickerInfo, FlickerOutput, LuxCct, OtpData,
};