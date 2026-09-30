use super::{
    device::Vd6283,
    error::Result,
    types::{Channel, FlickerOutput},
};

pub const FLK_DATA_SIZE:        usize = 1024;
pub const FLK_SAMPLING_FREQ_HZ: u32 = 8_000;
pub const FLK_CHANNEL:          Channel = Channel::Ch6;
const SATURATION_LIMIT:         u16 = 2;
const AUTOGAIN_GAINS:           [u16; 16] = [
    0x42ab, 0x42ab, 0x3200, 0x2154, 0x1900, 0x10ab, 0x0a00, 0x0723, 0x0500, 0x0354, 0x0280, 0x01ab,
    0x0140, 0x0100, 0x00d4, 0x00b5,
];

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct FlickerReading {
    pub frequency_hz: u32,
}

pub struct FlickerAnalyzer {
    twiddle_cos:    [f32; FLK_DATA_SIZE / 2],
    twiddle_sin:    [f32; FLK_DATA_SIZE / 2],
    real:           [f32; FLK_DATA_SIZE],
    imag:           [f32; FLK_DATA_SIZE],
    magnitude:      [f32; FLK_DATA_SIZE / 2],
    pending:        [i16; FLK_DATA_SIZE],
    pending_len:    usize,
}

impl FlickerAnalyzer {
    pub fn new() -> Self {
        let mut analyzer = Self {
            twiddle_cos:    [0.0; FLK_DATA_SIZE / 2],
            twiddle_sin:    [0.0; FLK_DATA_SIZE / 2],
            real:           [0.0; FLK_DATA_SIZE],
            imag:           [0.0; FLK_DATA_SIZE],
            magnitude:      [0.0; FLK_DATA_SIZE / 2],
            pending:        [0; FLK_DATA_SIZE],
            pending_len:    0,
        };
        for k in 0..FLK_DATA_SIZE / 2 {
            let angle = core::f32::consts::TAU * k as f32 / FLK_DATA_SIZE as f32;
            analyzer.twiddle_cos[k] = libm::cosf(angle);
            analyzer.twiddle_sin[k] = libm::sinf(angle);
        }
        analyzer
    }

    pub fn push_samples(&mut self, samples: &[i16]) -> Option<FlickerReading> {
        let mut consumed = 0;
        while self.pending_len < FLK_DATA_SIZE && consumed < samples.len() {
            self.pending[self.pending_len] = samples[consumed];
            self.pending_len += 1;
            consumed += 1;
        }
        if self.pending_len < FLK_DATA_SIZE {
            return None;
        }
        self.pending_len = 0;
        Some(self.analyze())
    }

    fn analyze(&mut self) -> FlickerReading {
        for i in 0..FLK_DATA_SIZE {
            self.real[i] = self.pending[i] as f32;
            self.imag[i] = 0.0;
        }
        Self::fft_in_place(
            &mut self.real,
            &mut self.imag,
            &self.twiddle_cos,
            &self.twiddle_sin,
        );

        for bin in 0..FLK_DATA_SIZE / 2 {
            self.magnitude[bin] =
                libm::sqrtf(self.real[bin] * self.real[bin] + self.imag[bin] * self.imag[bin]);
        }

        let mut index_max = 1;
        let mut max_value = self.magnitude[1];
        for bin in 2..FLK_DATA_SIZE / 2 {
            if self.magnitude[bin] > max_value {
                index_max = bin;
                max_value = self.magnitude[bin];
            }
        }
        FlickerReading {
            frequency_hz: (index_max as u32 * FLK_SAMPLING_FREQ_HZ) / FLK_DATA_SIZE as u32,
        }
    }

    fn fft_in_place(real: &mut [f32], imag: &mut [f32], twiddle_cos: &[f32], twiddle_sin: &[f32]) {
        let n = real.len();
        let bits = n.trailing_zeros();

        for i in 0..n {
            let j = ((i as u32).reverse_bits() >> (32 - bits)) as usize;
            if j > i {
                real.swap(i, j);
                imag.swap(i, j);
            }
        }

        let mut len = 2;
        while len <= n {
            let half = len / 2;
            let stride = n / len;
            let mut start = 0;
            while start < n {
                let mut k = 0;
                for offset in 0..half {
                    let a = start + offset;
                    let b = a + half;
                    let wr = twiddle_cos[k];
                    let wi = -twiddle_sin[k];
                    let t_re = real[b] * wr - imag[b] * wi;
                    let t_im = real[b] * wi + imag[b] * wr;
                    real[b] = real[a] - t_re;
                    imag[b] = imag[a] - t_im;
                    real[a] += t_re;
                    imag[a] += t_im;
                    k += stride;
                }
                start += len;
            }
            len *= 2;
        }
    }
}

impl Default for FlickerAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

impl<I2C> Vd6283<I2C>
where
    I2C: embedded_hal::i2c::I2c,
{
    pub fn flicker_autogain<D>(&mut self, delay: &mut D, timeout_ms: u32) -> Result<u16>
    where
        D: embedded_hal::delay::DelayNs,
    {
        let timeout_ms = timeout_ms.clamp(1, 100);
        let mut idx: i32 = 7;
        let mut saturation;

        for step in 0..4 {
            self.set_gain(FLK_CHANNEL, AUTOGAIN_GAINS[idx as usize])?;
            self.start_flicker(FLK_CHANNEL, FlickerOutput::Analog)?;

            saturation = 0;
            for _ in 0..timeout_ms {
                delay.delay_ms(1);
                saturation = self.saturation_value()?;
                if saturation > SATURATION_LIMIT {
                    break;
                }
            }

            self.stop_flicker()?;

            if step == 0 {
                if saturation > SATURATION_LIMIT {
                    idx += 1;
                }
            } else {
                let move_by = 1 << (step - 1);
                idx += if saturation > SATURATION_LIMIT {
                    move_by
                } else {
                    -move_by
                };
            }
            idx = idx.clamp(0, 15);
        }

        self.set_gain(FLK_CHANNEL, AUTOGAIN_GAINS[idx as usize])
    }
}
