//! Draw the Rust logo centered on a 90 degree rotated 128x64px display
//!
//! Image was created with ImageMagick:
//!
//! ```bash
//! convert rust.png -depth 1 gray:rust.raw
//! ```
//!
//! This example is for the STM32F103 "Blue Pill" board using I2C1.
//!
//! Wiring connections are as follows for a CRIUS-branded display:
//!
//! ```
//!      Display -> Blue Pill
//! (black)  GND -> GND
//! (red)    +5V -> VCC
//! (yellow) SDA -> PB9
//! (green)  SCL -> PB8
//! ```
//!
//! Run on a Blue Pill with `cargo run --example rotation_i2c`, currently only works on nightly.

#![no_std]
#![no_main]

extern crate panic_semihosting;

use cortex_m_rt::ExceptionFrame;
use cortex_m_rt::{entry, exception};
use embedded_graphics::image::Image1BPP;
use embedded_graphics::prelude::*;
use ssh1106::prelude::*;
use ssh1106::Builder;
use stm32f103xx_hal::i2c::{BlockingI2c, DutyCycle, Mode};
use stm32f103xx_hal::prelude::*;

#[entry]
fn main() -> ! {
    let dp = stm32f103xx_hal::stm32f103xx::Peripherals::take().unwrap();

    let mut flash = dp.FLASH.constrain();
    let mut rcc = dp.RCC.constrain();

    let clocks = rcc.cfgr.freeze(&mut flash.acr);

    let mut afio = dp.AFIO.constrain(&mut rcc.apb2);

    let mut gpiob = dp.GPIOB.split(&mut rcc.apb2);

    let scl = gpiob.pb8.into_alternate_open_drain(&mut gpiob.crh);
    let sda = gpiob.pb9.into_alternate_open_drain(&mut gpiob.crh);

    let i2c = BlockingI2c::i2c1(
        dp.I2C1,
        (scl, sda),
        &mut afio.mapr,
        Mode::Fast {
            frequency: 400_000,
            duty_cycle: DutyCycle::Ratio2to1,
        },
        clocks,
        &mut rcc.apb1,
        1000,
        10,
        1000,
        1000,
    );

    let mut disp: GraphicsMode<_> = Builder::new()
        // Set initial rotation at 90 degrees clockwise
        .with_rotation(DisplayRotation::Rotate90)
        .connect_i2c(i2c)
        .into();

    disp.init().unwrap();
    disp.flush().unwrap();

    // Contrived example to test builder and instance methods. Sets rotation to 270 degress
    // or 90 degress counterclockwise
    disp.set_rotation(DisplayRotation::Rotate270).unwrap();

    let (w, h) = disp.get_dimensions();

    let im = Image1BPP::new(include_bytes!("./rust.raw"), 64, 64)
        .translate(Coord::new(w as i32 / 2 - 64 / 2, h as i32 / 2 - 64 / 2));

    disp.draw(im.into_iter());

    disp.flush().unwrap();

    loop {}
}

#[exception]
fn HardFault(ef: &ExceptionFrame) -> ! {
    panic!("{:#?}", ef);
}
