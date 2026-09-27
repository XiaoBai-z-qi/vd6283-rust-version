pub mod error;
pub mod regs;
pub mod types;
pub mod device;
pub mod bus;
pub mod init;
pub mod otp;
pub mod control;
pub mod gain;

pub use device::Vd6283;
pub use error::{Error, Result};
pub use types::{
    AlsData, Channel, Color, DeviceState, 
    FlickerInfo, FlickerOutput, LuxCct, OtpData,
};