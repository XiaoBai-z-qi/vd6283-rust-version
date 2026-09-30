#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Error {
    InvalidParams,
    Init,
    Timeout,
    InvalidDeviceId,
    I2cWrite,
    I2cRead,
    AlreadyStarted,
    NotStarted,
    NotSupported,
    Deprecated,
}

pub type Result<T> = core::result::Result<T, Error>;
