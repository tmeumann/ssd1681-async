use crate::driver::DisplayDriver;
use core::convert::Infallible;
use embedded_graphics::pixelcolor::BinaryColor;
use embedded_graphics::prelude::*;
use embedded_graphics::primitives::Rectangle;

#[derive(Default)]
pub enum Rotation {
    #[default]
    _0,
    _90,
    _180,
    _270,
}

pub struct BufferedDisplay<D: DisplayDriver, const N: usize> {
    driver: D,
    buffer: [u8; N],
    rotation: Rotation,
}

impl<D: DisplayDriver, const N: usize> BufferedDisplay<D, N> {
    pub fn new(driver: D, buffer: [u8; N], rotation: Rotation) -> Self {
        const {
            assert!(
                N == D::X * D::Y / 8,
                "unexpected buffer size (expected `X * Y / 8` bytes)"
            )
        }
        Self {
            driver,
            buffer,
            rotation,
        }
    }

    pub async fn flush(&mut self) -> Result<(), D::Error> {
        self.driver.draw_frame(&self.buffer).await
    }
}

impl<D: DisplayDriver, const N: usize> Dimensions for BufferedDisplay<D, N> {
    fn bounding_box(&self) -> Rectangle {
        let size = match self.rotation {
            Rotation::_0 | Rotation::_180 => Size::new(D::X as u32, D::Y as u32),
            Rotation::_90 | Rotation::_270 => Size::new(D::Y as u32, D::X as u32),
        };

        Rectangle::new(Point::new(0, 0), size)
    }
}

impl<D: DisplayDriver, const N: usize> DrawTarget for BufferedDisplay<D, N> {
    type Color = BinaryColor;
    type Error = Infallible;

    fn draw_iter<I>(&mut self, pixels: I) -> Result<(), Self::Error>
    where
        I: IntoIterator<Item = Pixel<Self::Color>>,
    {
        let bytes_per_scan = D::X / 8;

        for Pixel(Point { x, y }, color) in pixels.into_iter() {
            let x = x as usize;
            let y = y as usize;

            let [hw_x, hw_y] = match self.rotation {
                Rotation::_0 => [x, y],
                Rotation::_90 => [D::X - 1 - y, x],
                Rotation::_180 => [D::X - 1 - x, D::Y - 1 - y],
                Rotation::_270 => [y, D::Y - 1 - x],
            };

            let byte_index = hw_y * bytes_per_scan + hw_x / 8;
            let bit_index = hw_x % 8;

            if let Some(byte) = self.buffer.get_mut(byte_index) {
                match color {
                    BinaryColor::Off => {
                        *byte &= !(0b1000_0000 >> bit_index);
                    }
                    BinaryColor::On => {
                        *byte |= 0b1000_0000 >> bit_index;
                    }
                }
            }
        }

        Ok(())
    }
}
