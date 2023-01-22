use core::convert::TryInto;

use embedded_graphics::{pixelcolor::BinaryColor, prelude::*};

use embedded_hal::blocking::spi;
use embedded_hal::digital::v2::OutputPin;

mod commands;

pub struct GDEH0213B72<SPI, CS, CD>
where
    SPI: spi::Write<u8>,
    CS: OutputPin,
    CD: OutputPin,
{
    framebuffer: [u8; 250 * 122],
    spi: SPI,
    cs: CS,
    cd: CD,
}

impl<SPI: spi::Write<u8>, CS: OutputPin, CD: OutputPin> GDEH0213B72<SPI, CS, CD> {
    pub fn new(spi: SPI, cs: CS, cd: CD) -> Self {
        Self {
            framebuffer: [0; 250 * 122],
            spi,
            cs,
            cd,
        }
    }

    pub fn flush(&mut self) -> Result<(), ()> {
        Self::write_command_and_data_helper(
            &mut self.spi,
            &mut self.cs,
            &mut self.cd,
            0x24,
            &self.framebuffer,
        )?;
        self.write_command(0x26)?;
        self.write_command_and_data(0x37, &[0x00, 0x40, 0x20, 0x10, 0x00, 0x00, 0x00, 0x00])?;
        self.write_command_and_data(0x22, &[0xf4])?;
        self.write_command(0x20)?;
        Ok(())
    }

    pub fn init(&mut self) -> Result<(), ()> {
        self.cd.set_high().map_err(|_| ())?;
        self.cs.set_high().map_err(|_| ())?;
        self.write_command(0x12)?;
        self.write_command_and_data(0x01, &[0xf9, 0x00, 0x00])?;
        self.write_command_and_data(0x11, &[0x01])?;
        self.write_command_and_data(0x44, &[0x00, 0x0f])?;
        self.write_command_and_data(0x45, &[0xf9, 0x00, 0x00, 0x00])?;
        self.write_command_and_data(0x3c, &[0x05])?;
        self.write_command_and_data(0x21, &[0x00, 0x80])?;
        self.write_command_and_data(0x18, &[0x80])?;
        self.write_command_and_data(0x4e, &[0x00])?;
        self.write_command_and_data(0x4f, &[0xf9, 0x00])?;
        self.write_command_and_data(0x4f, &[0xf4, 0xf4, 0xf4, 0x0f])?;
        Ok(())
    }

    fn write_command_and_data(&mut self, cmd: u8, data: &[u8]) -> Result<(), ()> {
        Self::write_command_and_data_helper(&mut self.spi, &mut self.cs, &mut self.cd, cmd, data)
    }
    fn write_command(&mut self, cmd: u8) -> Result<(), ()> {
        Self::write_command_helper(&mut self.spi, &mut self.cs, &mut self.cd, cmd)
    }
    #[allow(dead_code)]
    fn write_data(&mut self, data: &[u8]) -> Result<(), ()> {
        Self::write_data_helper(&mut self.spi, &mut self.cs, &mut self.cd, data)
    }

    fn write_command_and_data_helper(
        spi: &mut SPI,
        cs: &mut CS,
        cd: &mut CD,
        cmd: u8,
        data: &[u8],
    ) -> Result<(), ()> {
        Self::write_command_helper(spi, cs, cd, cmd)?;
        Self::write_data_helper(spi, cs, cd, data)
    }

    fn write_command_helper(spi: &mut SPI, cs: &mut CS, cd: &mut CD, cmd: u8) -> Result<(), ()> {
        cs.set_low().map_err(|_| ())?;
        cd.set_low().map_err(|_| ())?;
        spi.write(&[cmd]).map_err(|_| ())?;
        cs.set_high().map_err(|_| ())?;
        Ok(())
    }

    fn write_data_helper(spi: &mut SPI, cs: &mut CS, cd: &mut CD, data: &[u8]) -> Result<(), ()> {
        cs.set_low().map_err(|_| ())?;
        cd.set_high().map_err(|_| ())?;
        spi.write(data).map_err(|_| ())?;
        cs.set_high().map_err(|_| ())?;
        Ok(())
    }
}

impl<SPI, CS, CD> DrawTarget for GDEH0213B72<SPI, CS, CD>
where
    SPI: spi::Write<u8>,
    CS: OutputPin,
    CD: OutputPin,
{
    type Color = BinaryColor;
    // `ExampleDisplay` uses a framebuffer and doesn't need to communicate with the display
    // controller to draw pixel, which means that drawing operations can never fail. To reflect
    // this the type `Infallible` was chosen as the `Error` type.
    type Error = core::convert::Infallible;

    fn draw_iter<I>(&mut self, pixels: I) -> Result<(), Self::Error>
    where
        I: IntoIterator<Item = Pixel<Self::Color>>,
    {
        for Pixel(coord, color) in pixels.into_iter() {
            // Check if the pixel coordinates are out of bounds (negative or greater than
            // (63,63)). `DrawTarget` implementation are required to discard any out of bounds
            // pixels without returning an error or causing a panic.
            if let Ok((x @ 0..=250, y @ 0..=122)) = coord.try_into() {
                // Calculate the index in the framebuffer.
                let index: u32 = x + (y * 122);
                self.framebuffer[index as usize] = color.is_on() as u8;
            }
        }

        Ok(())
    }
}

impl<SPI, CS, CD> OriginDimensions for GDEH0213B72<SPI, CS, CD>
where
    SPI: spi::Write<u8>,
    CS: OutputPin,
    CD: OutputPin,
{
    fn size(&self) -> Size {
        Size::new(250, 122)
    }
}
