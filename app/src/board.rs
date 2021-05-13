//! Board initialization

use max3010x::Max3010x;
use stm32f1xx_hal::prelude::*;

use crate::{consts::*, delay::*, lcd::*, types::*};

use stm32f1xx_hal::i2c;
use stm32f1xx_hal::i2c::blocking::BlockingI2c;

pub struct Board {
    pub test_pin: TestPin,
    pub beeper: BeeperPin,
    pub max30102_sensor: Max30102Sensor,
    pub lcd: Lcd<AsmDelay, 0>,
}

impl Board {
    pub fn init(
        core: &mut stm32f1::stm32f107::CorePeripherals,
        device: stm32f1::stm32f107::Peripherals,
    ) -> Self {
        let mut flash = device.FLASH.constrain();

        let mut gpioa = device.GPIOA.split();
        let mut gpiob = device.GPIOB.split();
        let mut gpioc = device.GPIOC.split();
        let mut gpiod = device.GPIOD.split();

        let lcd = Lcd::new(
            AsmDelay,
            device.GPIOE,
            &device.RCC,
            gpiod.pd14.into_push_pull_output(&mut gpiod.crh),
            gpioc.pc8.into_push_pull_output(&mut gpioc.crh),
            gpiod.pd13.into_push_pull_output(&mut gpiod.crh),
            gpiob.pb14.into_push_pull_output(&mut gpiob.crh),
            gpiod.pd15.into_push_pull_output(&mut gpiod.crh),
        )
        .unwrap();

        let mut afio = device.AFIO.constrain();

        // See stm32cube config for these.
        // Manual prediv2 config results in the 8MHz
        // frequency on the 1st pll that library code
        // can already deal with.
        device.RCC.cfgr2.modify(|_, w| {
            w.prediv2()
                .div5()
                .pll2mul()
                .mul8()
                .pll3mul()
                .mul8()
                .prediv1src()
                .pll2()
                .prediv1()
                .div5()
        });

        // turn on PLL2
        device.RCC.cr.modify(|_, w| w.pll2on().set_bit());

        let rcc = device.RCC.constrain();

        // https://github.com/stm32-rs/stm32f1xx-hal/issues/338
        // this clock calculation doesn't include cfgr2,
        // fake it, pretend that crystal is 8Mhz, which is the
        // frequency we'll have at PLL input with the PLL2
        // cfgr2 config above
        let clocks = rcc
            .cfgr
            .use_hse(8.mhz()) // fake it, system clock is 25Mhz
            .hclk(SYS_FREQ)
            .pclk1(36.mhz())
            .pclk2(SYS_FREQ)
            .sysclk(SYS_FREQ)
            .adcclk(12.mhz())
            .freeze(&mut flash.acr);

        let test_pin = gpiob.pb5.into_push_pull_output(&mut gpiob.crl);
        let beeper = gpioa.pa2.into_push_pull_output(&mut gpioa.crl);

        // Initialize (enable) the monotonic timer (CYCCNT)
        core.DCB.enable_trace();

        // required on Cortex-M7 devices that software lock the DWT (e.g. STM32F7)
        cortex_m::peripheral::DWT::unlock();
        core.DWT.enable_cycle_counter();

        let scl = gpiob.pb6.into_alternate_open_drain(&mut gpiob.crl);
        let sda = gpiob.pb7.into_alternate_open_drain(&mut gpiob.crl);

        let i2c = BlockingI2c::i2c1(
            device.I2C1,
            (scl, sda),
            &mut afio.mapr,
            i2c::Mode::Fast {
                frequency: 400_000.hz(),
                duty_cycle: i2c::DutyCycle::Ratio2to1,
            },
            clocks,
            1000,
            10,
            1000,
            1000,
        );

        //
        // With this config:
        // Fs = 25Hz
        // Fs/2 = 12.5Hz
        // buffer = 100samples
        //
        // Fhrmax = 260bpm = 4Hz
        // Fhrmin = 40Hbpm = 0.65Hz
        //
        // Fstop_norm = 0.026
        // Fmax_norm = 0.16
        // Ftypical_norm = 0.04
        //
        let max30102_sensor = Max3010x::new_max30102(i2c);
        let mut max30102_sensor = max30102_sensor.into_oximeter().unwrap();
        max30102_sensor
            .set_pulse_amplitude(max3010x::Led::All, 200)
            .unwrap();
        max30102_sensor
            .set_pulse_width(max3010x::LedPulseWidth::Pw411)
            .unwrap();
        max30102_sensor
            .set_sampling_rate(max3010x::SamplingRate::Sps400)
            .unwrap();
        max30102_sensor
            .set_sample_averaging(max3010x::SampleAveraging::Sa16)
            .unwrap();
        max30102_sensor
            .set_adc_range(max3010x::AdcRange::Fs16k)
            .unwrap();
        max30102_sensor.enable_fifo_rollover().unwrap();
        max30102_sensor.clear_fifo().unwrap();

        Board {
            test_pin,
            beeper,
            max30102_sensor,
            lcd,
        }
    }
}
