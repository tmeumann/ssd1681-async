use crate::driver::DisplayDriver;
use core::convert::Infallible;
use embedded_graphics::pixelcolor::BinaryColor;
use embedded_graphics::prelude::*;
use embedded_graphics::primitives::Rectangle;

pub struct BufferedDisplay<D: DisplayDriver, const N: usize> {
    driver: D,
    buffer: [u8; N],
}

impl<D: DisplayDriver, const N: usize> BufferedDisplay<D, N> {
    pub fn new(driver: D, buffer: [u8; N]) -> Self {
        const { assert!(N >= D::BUF_LEN) }
        Self { driver, buffer }
    }

    pub async fn flush(&mut self) -> Result<usize, D::Error> {
        self.driver.draw_frame(&self.buffer).await
    }
}

impl<D: DisplayDriver, const N: usize> Dimensions for BufferedDisplay<D, N> {
    fn bounding_box(&self) -> Rectangle {
        Rectangle::new(Point::new(0, 0), Size::new(D::X as u32, D::Y as u32))
    }
}

impl<D: DisplayDriver, const N: usize> DrawTarget for BufferedDisplay<D, N> {
    type Color = BinaryColor;
    type Error = Infallible;

    fn draw_iter<I>(&mut self, pixels: I) -> Result<(), Self::Error>
    where
        I: IntoIterator<Item = Pixel<Self::Color>>,
    {
        let bytes_per_row = D::X / 8;

        for Pixel(Point { x, y }, color) in pixels.into_iter() {
            let byte_index = y as usize * bytes_per_row + (x as usize / 8);
            match color {
                BinaryColor::Off => {
                    self.buffer[byte_index] &= !(0b1000_0000 >> (x % 8));
                }
                BinaryColor::On => {
                    self.buffer[byte_index] |= 0b1000_0000 >> (x % 8);
                }
            }
        }

        Ok(())
    }
}
