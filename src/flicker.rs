#![no_std]
#![no_main]

//! VD6283 模拟频闪测量程序。
//!
//! 只做频闪检测：通道 6 输出模拟频闪信号，接到 PA0 / ADC1_IN0，
//! TIM2 以 8 kHz 触发 ADC 采样，攒满 1024 点后做 FFT 找主频，
//! 结果通过串口 1 输出。不测量环境光（ALS）。

use core::{cell::RefCell, fmt::Write};
use cortex_m::interrupt::{free, Mutex};
use cortex_m_rt::entry;
use panic_halt as _;
use stm32f4xx_hal::{
    adc::{config::AdcConfig, config::SampleTime, Adc},
    gpio::{Analog, PA0},
    i2c::I2c,
    pac::{self, interrupt, ADC1, TIM2},
    prelude::*,
    rcc::Config,
    serial::{config::Config as SerialConfig, Serial},
    timer::{CounterUs, Event, Flag},
};

use vd6283::vd6283::{
    FlickerOutput, Vd6283,
    flicker::{FlickerAnalyzer, FLK_CHANNEL, FLK_SAMPLING_FREQ_HZ},
};

/// 单帧 ADC 采样数。两帧凑满一次 1024 点分析。
const ADC_FRAME_SIZE: usize = 512;

/// 频闪 ADC 采样双缓冲：ISR 填充，主循环取走后做 FFT。
struct SampleQueue {
    buffers: [[i16; ADC_FRAME_SIZE]; 2],
    write_buffer: usize,
    write_position: usize,
    ready_mask: u8,
    dropped_samples: u32,
}

impl SampleQueue {
    const fn new() -> Self {
        Self {
            buffers: [[0; ADC_FRAME_SIZE]; 2],
            write_buffer: 0,
            write_position: 0,
            ready_mask: 0,
            dropped_samples: 0,
        }
    }

    fn push(&mut self, sample: i16) {
        // 两帧都被主循环占用时丢弃新样本，避免覆盖未处理数据。
        if self.ready_mask & (1 << self.write_buffer) != 0 {
            self.dropped_samples = self.dropped_samples.saturating_add(1);
            return;
        }

        self.buffers[self.write_buffer][self.write_position] = sample;
        self.write_position += 1;
        if self.write_position != ADC_FRAME_SIZE {
            return;
        }

        self.ready_mask |= 1 << self.write_buffer;
        let next_buffer = self.write_buffer ^ 1;
        if self.ready_mask & (1 << next_buffer) == 0 {
            self.write_buffer = next_buffer;
            self.write_position = 0;
        }
    }

    /// 取走一个就绪的帧，返回是否有帧可取。
    fn take(&mut self, output: &mut [i16; ADC_FRAME_SIZE]) -> bool {
        let Some(buffer) = (0..2).find(|buffer| self.ready_mask & (1 << buffer) != 0) else {
            return false;
        };
        output.copy_from_slice(&self.buffers[buffer]);
        self.ready_mask &= !(1 << buffer);
        true
    }
}

static ADC_STATE: Mutex<RefCell<Option<(Adc<ADC1>, PA0<Analog>)>>> = Mutex::new(RefCell::new(None));
static TIMER_STATE: Mutex<RefCell<Option<CounterUs<TIM2>>>> = Mutex::new(RefCell::new(None));
static SAMPLE_QUEUE: Mutex<RefCell<SampleQueue>> = Mutex::new(RefCell::new(SampleQueue::new()));

/// TIM2 更新中断以 8 kHz 触发一次 ADC 转换。
#[interrupt]
fn TIM2() {
    free(|cs| {
        if let Some((adc, pin)) = ADC_STATE.borrow(cs).borrow_mut().as_mut() {
            let sample = adc.convert(pin, SampleTime::Cycles_3);
            SAMPLE_QUEUE.borrow(cs).borrow_mut().push(sample as i16);
        }
        if let Some(timer) = TIMER_STATE.borrow(cs).borrow_mut().as_mut() {
            timer.clear_flags(Flag::Update);
        }
    });
}

#[entry]
fn main() -> ! {
    let dp = pac::Peripherals::take().unwrap();
    let cp = cortex_m::Peripherals::take().unwrap();
    let mut rcc = dp
        .RCC
        .freeze(Config::hsi().sysclk(100.MHz()));
    let mut delay = cp.SYST.delay(&rcc.clocks);
    let gpio_a = dp.GPIOA.split(&mut rcc);
    let gpio_b = dp.GPIOB.split(&mut rcc);

    let mut serial1 = Serial::new(
        dp.USART1, 
        (gpio_a.pa9, gpio_a.pa10),
        SerialConfig::default().baudrate(115_200.bps()), 
        &mut rcc,
    )
    .unwrap()
    .with_u8_data();

    let i2c1 = I2c::new(
        dp.I2C1, 
        (gpio_b.pb6, gpio_b.pb7), 
        100.kHz(), 
        &mut rcc,
    );

    let mut sensor = Vd6283::new(i2c1);

    if let Err(error) = sensor.init() {
        writeln!(serial1, "VD6283 Init is Failed: {:?} ! ! !", error).ok();
        loop{
            delay.delay_ms(1_000_u32);
        }
    }
    writeln!(serial1, "VD6283 is normal working").ok();
    writeln!(serial1, "VD6283 id = 0x{:02x}, revision = 0x{:02x}", 
            sensor.device_id(), sensor.revision_id()).ok();

    match sensor.flicker_autogain(&mut delay, 1) {
        Ok(applied) => {
            writeln!(serial1, "flicker ch6 gain set to {:.2}x", 
                    applied as f32 / 256.0).ok();
        }
        Err(error) => {
            writeln!(serial1, "flicker autogain failed: {:?}", error).ok();
            loop {
                delay.delay_ms(1_000_u32);
            }
        }
    }

    if let Err(error) = sensor.start_flicker(FLK_CHANNEL, FlickerOutput::Analog) {
        writeln!(serial1, "start flicker failed: {:?}", error).ok();
        loop {
            delay.delay_ms(1_000_u32);
        }
    }

    let analog_input = gpio_a.pa0.into_analog();
    let adc = Adc::new(
        dp.ADC1,
        true,
        AdcConfig::default(),
        &mut rcc,
    );
    let mut timer = dp.TIM2.counter(&mut rcc);
    timer
        .start((1_000_000_u32 / FLK_SAMPLING_FREQ_HZ).micros())
        .ok();

    free(|cs| {
        ADC_STATE.borrow(cs).replace(Some((adc, analog_input)));
        TIMER_STATE.borrow(cs).replace(Some(timer));
    });
    free(|cs| {
        let mut timer = TIMER_STATE.borrow(cs).borrow_mut();
        let timer = timer.as_mut().unwrap();
        timer.listen(Event::Update);
    });
    unsafe { cortex_m::peripheral::NVIC::unmask(interrupt::TIM2) };
    writeln!(serial1, "ADC sampling at {} Hz", FLK_SAMPLING_FREQ_HZ).ok();

    let mut analyzer = FlickerAnalyzer::new();
    let mut frame = [0_i16; ADC_FRAME_SIZE];

    loop {
        // 取走一帧采样（512 点），两帧凑满 1024 点后由分析器给出频闪频率。
        let frame_ready = free(|cs| SAMPLE_QUEUE.borrow(cs).borrow_mut().take(&mut frame));
        if frame_ready {
            if let Some(reading) = analyzer.push_samples(&frame) {
                writeln!(serial1, "flicker={}Hz", reading.frequency_hz).ok();
                continue;
            }
        }

        // 没有待处理的帧时休眠，等 TIM2 中断唤醒。
        if !frame_ready {
            cortex_m::asm::wfi();
        }
    }
}

