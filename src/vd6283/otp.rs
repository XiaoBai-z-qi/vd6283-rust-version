//! VD6283 OTP 读取和 UID 生成。

use super::{
    device::Vd6283,
    error::{Error, Result},
    regs,
    types::{OtpData, CHANNEL_COUNT},
};

impl<I2C> Vd6283<I2C>
where
    I2C: embedded_hal::i2c::I2c,
{
    /// 复位 OTP 控制器，并等待两个 OTP bank 都准备好。
    pub(crate) fn otp_reset(&mut self) -> Result<()> {
        self.write_reg(regs::REG_OTP_CONTROL_1, 0)?;

        for _ in 0..100 {
            let status = self.read_reg(regs::REG_OTP_STATUS)?;
            if status & regs::OTP_DATA_READY == regs::OTP_DATA_READY {
                return Ok(());
            }
        }

        Err(Error::Timeout)
    }

    /// 读取两个 OTP bank，并转换成便于位域读取的两个 u64。
    pub(crate) fn otp_read_init(&mut self) -> Result<()> {
        let mut bank0 = [0u8; 8];
        let mut bank1 = [0u8; 8];

        self.read_regs(regs::REG_OTP_BANK_0, &mut bank0)?;
        self.read_regs(regs::REG_OTP_BANK_1, &mut bank1)?;

        self.otp_bits[0] = 0;
        for (index, byte) in bank0[..7].iter().enumerate() {
            self.otp_bits[0] |= u64::from(reverse_bits(*byte)) << (index * 8);
        }
        self.otp_bits[0] |= u64::from(reverse_bits(bank0[7]) >> 4) << 56;

        let first = reverse_bits(bank1[0]);
        self.otp_bits[0] |= u64::from(first & 0x0f) << 60;
        self.otp_bits[1] = u64::from(first >> 4);

        for (index, byte) in bank1[1..7].iter().enumerate() {
            self.otp_bits[1] |= u64::from(reverse_bits(*byte)) << (index * 8 + 4);
        }
        self.otp_bits[1] |= u64::from(reverse_bits(bank1[7]) >> 4) << 52;

        Ok(())
    }

    /// 从两个 OTP bank 中读取一段最多 24 bit 的数据。
    fn otp_read(&self, bit_start: usize, bit_count: usize, bit_swap: bool) -> Result<u32> {
        if bit_count == 0 || bit_count > 24 || bit_start + bit_count > 120 {
            return Err(Error::InvalidParams);
        }

        let mut value = 0u32;
        for offset in 0..bit_count {
            let bit_index = bit_start + offset;
            let source = if bit_index < 64 {
                self.otp_bits[0] >> bit_index
            } else {
                self.otp_bits[1] >> (bit_index - 64)
            };
            let bit = (source & 1) as u32;
            let destination = if bit_swap {
                bit_count - 1 - offset
            } else {
                offset
            };
            value |= bit << destination;
        }

        Ok(value)
    }

    /// 读取 OTP 参数；无法识别具体 OTP 版本时使用默认值。
    pub(crate) fn otp_read_param(&mut self) -> Result<()> {
        self.otp = OtpData {
            hf_trim: 0x0e3,
            lf_trim: 0x07,
            filter_config: 2,
            filter_index: 2,
            gains: [0x80; CHANNEL_COUNT],
            version: 0x15,
        };

        let version = self.otp_read(113, 3, true)? + 0x10;
        self.otp.version = version as u8;

        self.otp.hf_trim = self.otp_read(51, 9, true)? as u16;
        self.otp.lf_trim = self.otp_read(116, 4, true)? as u8;
        self.otp.filter_config = self.otp_read(48, 3, true)? as u8;

        for channel in 0..CHANNEL_COUNT {
            let gain = self.otp_read(channel * 8, 8, true)? as u8;
            self.otp.gains[channel] = if gain == 0 { 0x80 } else { gain };
        }

        let check0 = self.otp_read(80, 16, false)?;
        let check1 = self.otp_read(96, 14, false)?;
        let check2 = self.otp_read(75, 5, true)?;
        if check0 == 0x56c1 && check1 == 0x0364 && (1..=6).contains(&check2) {
            self.otp.filter_config = 0x03;
        }

        Ok(())
    }

    /// 按原 C 驱动的 XOR 链编码规则生成 14 字符 UID。
    pub(crate) fn otp_generate_uid(&mut self) -> Result<()> {
        let mut xor_register = 0u8;
        let mut index = 0usize;

        let version = self.otp_read(113, 3, true)? as u8;
        let lowest_bit = self.otp_read(60, 1, true)? as u8;
        let first_nibble = (version << 1) | lowest_bit;
        self.uid[index] = encode_nibble(first_nibble, &mut xor_register);
        index += 1;

        for bit_start in (61..112).step_by(4) {
            let nibble = self.otp_read(bit_start, 4, true)? as u8;
            self.uid[index] = encode_nibble(nibble, &mut xor_register);
            index += 1;
        }

        for byte in self.uid[index..].iter_mut() {
            *byte = 0;
        }
        Ok(())
    }
}

/// 将一个字节的 bit 顺序完全翻转。
fn reverse_bits(mut value: u8) -> u8 {
    value = (value >> 1 & 0x55) | (value << 1 & 0xaa);
    value = (value >> 2 & 0x33) | (value << 2 & 0xcc);
    (value >> 4) | (value << 4)
}

/// 将四 bit 半字节转换成小写十六进制字符，并更新 XOR 状态。
fn encode_nibble(nibble: u8, xor_register: &mut u8) -> u8 {
    let previous = *xor_register;
    *xor_register ^= nibble;
    match nibble ^ previous {
        value @ 0..=9 => b'0' + value,
        value => b'a' + value - 10,
    }
}
