#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Error {
    /// 函数参数超出器件允许的范围。
    InvalidParams,
    /// 设备初始化流程失败。
    Init,
    /// 操作在规定时间内没有完成。
    Timeout,
    /// 器件 ID 或 revision 不受当前驱动支持。
    InvalidDeviceId,
    /// I2C 写事务失败。
    I2cWrite,
    /// I2C 读事务失败。
    I2cRead,
    /// 设备已经处于运行状态。
    AlreadyStarted,
    /// 设备当前未运行，但该操作要求设备正在运行。
    NotStarted,
    /// 当前实现不支持请求的功能。
    NotSupported,
    /// 为兼容原 C 接口而保留的废弃功能。
    Deprecated,
}

/// 驱动统一使用的结果类型。
pub type Result<T> = core::result::Result<T, Error>;