#![cfg_attr(not(doc), no_main)]
#![no_std]

use panic_halt as _;

#[rtic::app(device = stm32f1::stm32f107,
            peripherals = true,
            dispatchers = [EXTI4, FSMC, TAMPER], // Full list in  stm32f1::stm32f103::Interrupt
            )]
mod app {
    use cardiac_monitor::board::Board;
    use cardiac_monitor::model::{Max3012Sample, UIModel};
    use cardiac_monitor::{consts::*, lcdui::*, types::*};
    use cardiac_monitor_shared::circ::Circ;

    use rtic::Monotonic;
    use systick_monotonic::*;

    #[shared]
    struct Shared {
        max30102_samples: Circ<Max3012Sample, MAX30102_NUM_SAMPLES>,
    }

    #[local]
    struct Local {
        test_pin: TestPin,
        _beeper: BeeperPin,
        lcdui: LcdUI,
        max30102_sensor: Max30102Sensor,
        ui_model: UIModel,
    }

    // https://github.com/rtic-rs/cortex-m-rtic/blob/master/examples/schedule.rs
    #[monotonic(binds = SysTick, default = true)]
    type MyMono = Systick<100>; // 100 Hz / 10 ms granularity

    #[init]
    fn init(cx: init::Context) -> (Shared, Local, init::Monotonics) {
        let mut core = cx.core;
        let device = cx.device;

        let Board {
            test_pin,
            beeper,
            max30102_sensor,
            lcd,
        } = Board::init(&mut core, device);

        let mono = Systick::new(core.SYST, SYS_FREQ.0);
        sample::spawn_after(500.millis()).unwrap();

        (
            Shared {
                max30102_samples: Circ::new(Max3012Sample::zero()),
            },
            Local {
                test_pin,
                _beeper: beeper,
                lcdui: LcdUI::new(lcd),
                max30102_sensor,
                ui_model: UIModel::new(),
            },
            init::Monotonics(mono),
        )
    }

    #[idle(shared = [max30102_samples], local = [lcdui,ui_model,test_pin])]
    fn idle(mut ctx: idle::Context) -> ! {
        let lcdui = ctx.local.lcdui;
        let ui_model = ctx.local.ui_model;
        lcdui.init().unwrap();

        let test_pin = ctx.local.test_pin;

        let mut oxi_r_samples = [0.0; MAX30102_NUM_SAMPLES];
        let mut oxi_ir_samples = [0.0; MAX30102_NUM_SAMPLES];

        loop {
            test_pin.set_high();
            ctx.shared.max30102_samples.lock(|ss| {
                for (i, Max3012Sample { r, ir }) in ss.iter().enumerate() {
                    oxi_r_samples[i] = r;
                    oxi_ir_samples[i] = ir;
                }
            });
            test_pin.set_low();

            ui_model.update_from_samples(&oxi_r_samples, &oxi_ir_samples);
            lcdui.render(ui_model).unwrap();
        }
    }

    #[task(shared = [max30102_samples], local = [max30102_sensor], priority = 1)]
    fn sample(mut ctx: sample::Context) {
        sample::spawn_at(monotonics::now() + 40.millis()).unwrap();

        let mut max3012_data = [0; 2];
        let max30102_sensor = ctx.local.max30102_sensor;
        let samples_read = max30102_sensor.read_fifo(&mut max3012_data).unwrap();

        if samples_read > 0 {
            ctx.shared.max30102_samples.lock(|ss| {
                // TODO: docs indicate R,IR sequence
                // but that gives nonsensical SPO2 values,
                // something is flipped or off by 1 somewhere
                let si = Max3012Sample {
                    r: max3012_data[1] as f32,
                    ir: max3012_data[0] as f32,
                };
                ss.add(si)
            });
        }
    }
}
