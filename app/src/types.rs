use max3010x::{marker::ic::Max30102, Max3010x};
use stm32f1::stm32f107::I2C1;
use stm32f1xx_hal::{gpio::*, i2c::BlockingI2c};

pub type TestPin = gpiob::PB5<Output<PushPull>>;

pub type BeeperPin = gpioa::PA2<Output<PushPull>>;

pub type Max30102Sensor = Max3010x<
    BlockingI2c<
        I2C1,
        (
            stm32f1xx_hal::gpio::Pin<Alternate<OpenDrain>, CRL, 'B', 6_u8>,
            stm32f1xx_hal::gpio::Pin<Alternate<OpenDrain>, CRL, 'B', 7_u8>,
        ),
    >,
    Max30102,
    max3010x::marker::mode::Oximeter,
>;
