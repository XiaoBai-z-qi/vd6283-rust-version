#![no_std]
#![no_main]

use cortex_m_rt::entry;
use panic_halt as _;
use rtt_target::{rprintln, rtt_init_print};
//use rtt_target::{rprintln, rtt_init_print};
use stm32f4xx_hal::{i2c::I2c, pac, prelude::*, rcc::Config};

use vd6283_rust_version::vd6283::{get_lux_cct, Channel, Vd6283};

const I2C_BUS_HZ: u32 = 100_000;
const ALS_CHANNELS: u8 = 0x3f;
const EXPOSURE_US: u32 = 100_000;
#[entry]
fn main() -> ! {
    rtt_init_print!();
    rprintln!("vd6283-rust-version application start!!!");

    let dp = pac::Peripherals::take().unwrap();
    let cp = cortex_m::Peripherals::take().unwrap();

    let mut rcc = dp
        .RCC
        .freeze(Config::hsi().sysclk(100.MHz()));

    rprintln!("SYSCLK = {} Hz", rcc.clocks.sysclk().raw());
    let gpio_b = dp.GPIOB.split(&mut rcc);
    let i2c = I2c::new(
        dp.I2C1,
        (gpio_b.pb6, gpio_b.pb7),
        I2C_BUS_HZ.Hz(),
        &mut rcc,
    );

    let mut delay = cp.SYST.delay(&rcc.clocks);
    let mut sensor = Vd6283::new(i2c);

    if let Err(error) = sensor.init() {
        rprintln!("VD6283 init failed: {:?}", error);
        loop {
            delay.delay_ms(1_000_u32);
        }
    }
    rprintln!(
        "VD6283 id = 0x{:02x}, revision = 0x{:02x}",
        sensor.device_id(),
        sensor.revision_id()
    );
    if let Err(error) = sensor.set_exposure_time(EXPOSURE_US) {
        rprintln!("set exposure failed: {:?}", error);
    }
    for channel in [
        Channel::Ch1,
        Channel::Ch2,
        Channel::Ch3,
        Channel::Ch4,
        Channel::Ch5,
        Channel::Ch6,
    ] {
        if let Err(error) = sensor.set_gain(channel, 0x0100) {
            rprintln!("set gain failed: {:?}", error);
        }
    }
    loop {
        if let Err(error) = sensor.start_single_shot(ALS_CHANNELS) {
            rprintln!("start ALS failed: {:?}", error);
            delay.delay_ms(200_u32);
            continue;
        }

        loop {
            match sensor.read_als(ALS_CHANNELS) {
                Ok(Some(als)) => {
                    let result = get_lux_cct(&als, sensor.exposure_time_us());
                    rprintln!(
                        "Lux={:.2}, CCT={:.2} K, raw=[{}, {}, {}, {}, {}, {}]",
                        result.lux,
                        result.cct,
                        als.count_value_raw[0],
                        als.count_value_raw[1],
                        als.count_value_raw[2],
                        als.count_value_raw[3],
                        als.count_value_raw[4],
                        als.count_value_raw[5]
                    );
                    break;
                }
                Ok(None) => delay.delay_ms(1_u32),
                Err(error) => {
                    rprintln!("read ALS failed: {:?}", error);
                    break;
                }
            }
        }

        if let Err(error) = sensor.stop_als() {
            rprintln!("stop ALS failed: {:?}", error);
        }
        delay.delay_ms(200_u32);
    }
}