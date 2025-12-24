use crate::commands::{
    REFRESH_PANEL, RESET, SET_DATA_ENTRY_MODE, SET_RAM_X, SET_RAM_Y, SET_X_POINTER, SET_Y_POINTER,
    WRITE_RAM,
};
use crate::errors::DisplayError;
use embedded_hal::digital::{InputPin, OutputPin};
use embedded_hal_async::delay::DelayNs;
use embedded_hal_async::digital::Wait;
use embedded_hal_async::spi::SpiDevice;

pub trait DisplayDriver {
    type Error;
    const X: usize;
    const Y: usize;
    const BUF_LEN: usize = Self::X * Self::Y / 8;

    async fn enable_backlight(&mut self) -> Result<(), Self::Error>;
    async fn disable_backlight(&mut self) -> Result<(), Self::Error>;
    async fn draw_frame(&mut self, buffer: &[u8]) -> Result<usize, Self::Error>;
}

pub struct Ssd1681Builder<const X: usize, const Y: usize, BL: OutputPin> {
    backlight_pin: Option<BL>,
}

impl<const X: usize, const Y: usize, BL: OutputPin> Ssd1681Builder<X, Y, BL> {
    pub fn new() -> Self {
        Self {
            backlight_pin: None,
        }
    }

    pub fn with_backlight(&mut self, backlight_pin: BL) -> &mut Self {
        self.backlight_pin = Some(backlight_pin);
        self
    }

    pub async fn connect<
        SPI: SpiDevice,
        BUSY: InputPin + Wait,
        DC: OutputPin,
        RST: OutputPin,
        DELAY: DelayNs,
    >(
        self,
        spi: SPI,
        busy_pin: BUSY,
        dc_pin: DC,
        reset_pin: RST,
        delay: DELAY,
    ) -> Result<Ssd1681<X, Y, SPI, BUSY, DC, BL, RST, DELAY>, DisplayError> {
        Ssd1681::<X, Y, SPI, BUSY, DC, BL, RST, DELAY>::new(
            spi,
            busy_pin,
            dc_pin,
            self.backlight_pin,
            reset_pin,
            delay,
        )
        .await
    }
}

pub struct Ssd1681<const X: usize, const Y: usize, SPI, BUSY, DC, BL, RST, DELAY>
where
    SPI: SpiDevice,
    BUSY: InputPin + Wait,
    DC: OutputPin,
    BL: OutputPin,
    RST: OutputPin,
    DELAY: DelayNs,
{
    spi: SPI,
    busy_pin: BUSY,
    dc_pin: DC,
    backlight_pin: Option<BL>,
    reset_pin: RST,
    delay: DELAY,
}

impl<
    const X: usize,
    const Y: usize,
    SPI: SpiDevice,
    BUSY: InputPin + Wait,
    DC: OutputPin,
    BL: OutputPin,
    RST: OutputPin,
    DELAY: DelayNs,
> DisplayDriver for Ssd1681<X, Y, SPI, BUSY, DC, BL, RST, DELAY>
{
    type Error = DisplayError;
    const X: usize = X;
    const Y: usize = Y;

    async fn enable_backlight(&mut self) -> Result<(), DisplayError> {
        if let Some(pin) = self.backlight_pin.as_mut() {
            pin.set_high().map_err(|_| DisplayError::Backlight)?;
        }
        Ok(())
    }

    async fn disable_backlight(&mut self) -> Result<(), DisplayError> {
        if let Some(pin) = self.backlight_pin.as_mut() {
            pin.set_low().map_err(|_| DisplayError::Backlight)?;
        }
        Ok(())
    }

    async fn draw_frame(&mut self, buffer: &[u8]) -> Result<usize, DisplayError> {
        self.send_command(SET_X_POINTER, Some(&[0x00])).await?;
        self.send_command(SET_Y_POINTER, Some(&[0x00, 0x00]))
            .await?;
        let buf_size = (X * Y / 8);
        self.send_command(WRITE_RAM, Some(&buffer[0..buf_size]))
            .await?;
        self.send_command(REFRESH_PANEL, None).await?;

        Ok(buf_size)
    }
}

impl<
    const X: usize,
    const Y: usize,
    SPI: SpiDevice,
    BUSY: InputPin + Wait,
    DC: OutputPin,
    BL: OutputPin,
    RST: OutputPin,
    DELAY: DelayNs,
> Ssd1681<X, Y, SPI, BUSY, DC, BL, RST, DELAY>
{
    async fn new(
        spi: SPI,
        busy_pin: BUSY,
        dc_pin: DC,
        backlight_pin: Option<BL>,
        reset_pin: RST,
        delay: DELAY,
    ) -> Result<Self, DisplayError> {
        let mut new = Self {
            spi,
            busy_pin,
            dc_pin,
            backlight_pin,
            reset_pin,
            delay,
        };

        new.init().await?;

        Ok(new)
    }

    async fn init(&mut self) -> Result<(), DisplayError> {
        // self.delay.delay_ms(10).await;
        self.reset().await?;
        self.set_data_entry_mode().await?;
        self.set_ram_x().await?;
        self.set_ram_y().await?;
        Ok(())
    }

    async fn wait_for_idle(&mut self) -> Result<(), DisplayError> {
        self.busy_pin
            .wait_for_low()
            .await
            .map_err(|_| DisplayError::Busy)
    }

    async fn send_spi(&mut self, data: &[u8]) -> Result<(), DisplayError> {
        self.delay.delay_ms(10).await;
        self.spi.write(data).await.map_err(|_| DisplayError::Spi)
    }

    async fn send_command(&mut self, command: u8, data: Option<&[u8]>) -> Result<(), DisplayError> {
        self.wait_for_idle().await?;
        self.dc_pin
            .set_low()
            .map_err(|_| DisplayError::DataCommand)?;
        self.send_spi(&[command]).await?;

        if let Some(buf) = data {
            self.wait_for_idle().await?;
            self.dc_pin
                .set_high()
                .map_err(|_| DisplayError::DataCommand)?;
            self.send_spi(buf).await?;
        }

        Ok(())
    }

    async fn reset(&mut self) -> Result<(), DisplayError> {
        self.reset_pin.set_low().map_err(|_| DisplayError::Reset)?;
        self.delay.delay_ms(10).await;
        self.reset_pin.set_high().map_err(|_| DisplayError::Reset)?;
        self.delay.delay_ms(10).await;
        self.send_command(RESET, None).await?;
        self.delay.delay_ms(10).await;
        Ok(())
    }

    async fn set_data_entry_mode(&mut self) -> Result<(), DisplayError> {
        self.send_command(SET_DATA_ENTRY_MODE, Some(&[0x03])).await
    }

    async fn set_ram_x(&mut self) -> Result<(), DisplayError> {
        self.send_command(SET_RAM_X, Some(&[0x00, ((X - 1) / 8) as u8]))
            .await
    }

    async fn set_ram_y(&mut self) -> Result<(), DisplayError> {
        self.send_command(SET_RAM_Y, Some(&[0x00, 0x00, (Y - 1) as u8, 0x00]))
            .await
    }
}
