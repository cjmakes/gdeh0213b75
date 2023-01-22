use embedded_graphics::{
    mono_font::{ascii::FONT_6X10, MonoTextStyle},
    pixelcolor::BinaryColor,
    prelude::*,
    text::Text,
};

use rppal::gpio::Gpio;
use rppal::spi::{Bus, Mode, SlaveSelect, Spi};

use tracing::trace;

const CS: u8 = 27;
const CD: u8 = 17;

pub fn main() {
    tracing_subscriber::fmt::init();
    trace!("start");

    let gpio = Gpio::new().expect("gpio failed");
    trace!("got gpio");
    let cs = gpio.get(CS).unwrap().into_output();
    let cd = gpio.get(CD).unwrap().into_output();

    let spi = Spi::new(Bus::Spi0, SlaveSelect::Ss0, 16_000_000, Mode::Mode0).expect("spi failed");
    trace!("got spi");
    let mut display = gdeh0213b72::GDEH0213B72::new(spi, cs, cd);
    display.init().expect("init failed");
    display.flush().expect("flush failed");

    let style = MonoTextStyle::new(&FONT_6X10, BinaryColor::On);
    Text::new(&format!("Hello world"), Point::new(0, 10), style)
        .draw(&mut display)
        .map_err(|_| ())
        .unwrap();

    display.flush().expect("flush failed");
}
