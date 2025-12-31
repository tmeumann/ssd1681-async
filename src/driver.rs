use crate::commands::{
    REFRESH_PANEL, RESET, SET_DATA_ENTRY_MODE, SET_RAM_X, SET_RAM_Y, SET_TEMPERATURE_SENSOR,
    SET_UPDATE_SEQUENCE, SET_X_POINTER, SET_Y_POINTER, WRITE_RAM,
};
use crate::config::Ssd1681Config;
use crate::errors::DisplayError;
use embedded_hal::digital::{InputPin, OutputPin};
use embedded_hal_async::delay::DelayNs;
use embedded_hal_async::digital::Wait;
use embedded_hal_async::spi::SpiDevice;

pub trait DisplayDriver {
    type Error;

    const X: usize;
    const Y: usize;

    async fn draw_frame(&mut self, buffer: &[u8]) -> Result<(), Self::Error>;
    async fn enable_backlight(&mut self) -> Result<(), Self::Error>;
    async fn disable_backlight(&mut self) -> Result<(), Self::Error>;
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
    config: Ssd1681Config<X, Y>,
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
    pub async fn new(
        spi: SPI,
        busy_pin: BUSY,
        dc_pin: DC,
        reset_pin: RST,
        backlight_pin: Option<BL>,
        delay: DELAY,
        config: Ssd1681Config<X, Y>,
    ) -> Result<Self, DisplayError> {
        let mut new = Self {
            spi,
            busy_pin,
            dc_pin,
            backlight_pin,
            reset_pin,
            delay,
            config,
        };

        new.init().await?;

        Ok(new)
    }

    async fn init(&mut self) -> Result<(), DisplayError> {
        self.delay.delay_ms(10).await; // ensure 10ms has passed since powerup
        self.reset().await?;
        self.set_data_entry_mode().await?;
        self.set_ram_x().await?;
        self.set_ram_y().await?;
        self.set_internal_temp_sensor().await?;
        Ok(())
    }

    async fn wait_while_busy(&mut self) -> Result<(), DisplayError> {
        self.delay.delay_ms(self.config.busy_settle_ms).await;
        self.busy_pin
            .wait_for_low()
            .await
            .map_err(|_| DisplayError::BusyPinFailure)
    }

    async fn send_spi(&mut self, data: &[u8]) -> Result<(), DisplayError> {
        self.spi
            .write(data)
            .await
            .map_err(|_| DisplayError::SpiFailure)
    }

    async fn send_command(&mut self, command: u8, data: Option<&[u8]>) -> Result<(), DisplayError> {
        if self.busy()? {
            return Err(DisplayError::DeviceBusy);
        }

        self.dc_pin
            .set_low()
            .map_err(|_| DisplayError::DataCommandPinFailure)?;
        self.delay.delay_us(self.config.dc_settle_us).await;

        self.send_spi(&[command]).await?;

        if let Some(buf) = data {
            self.dc_pin
                .set_high()
                .map_err(|_| DisplayError::DataCommandPinFailure)?;
            self.delay.delay_us(self.config.dc_settle_us).await;
            self.send_spi(buf).await?;
        }
        self.wait_while_busy().await?;

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

    async fn set_internal_temp_sensor(&mut self) -> Result<(), DisplayError> {
        self.send_command(SET_TEMPERATURE_SENSOR, Some(&[0x80]))
            .await
    }

    async fn set_update_sequence(&mut self) -> Result<(), DisplayError> {
        self.send_command(SET_UPDATE_SEQUENCE, Some(&[0xF7])).await
    }

    fn busy(&mut self) -> Result<bool, DisplayError> {
        self.busy_pin
            .is_high()
            .map_err(|_| DisplayError::BusyPinFailure)
    }

    async fn reset(&mut self) -> Result<(), DisplayError> {
        self.reset_pin
            .set_low()
            .map_err(|_| DisplayError::ResetPinFailure)?;

        self.delay.delay_ms(10).await;

        self.reset_pin
            .set_high()
            .map_err(|_| DisplayError::ResetPinFailure)?;

        self.delay.delay_ms(10).await;

        self.send_command(RESET, None).await?;

        Ok(())
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
> DisplayDriver for Ssd1681<X, Y, SPI, BUSY, DC, BL, RST, DELAY>
{
    type Error = DisplayError;

    const X: usize = X;
    const Y: usize = Y;

    async fn draw_frame(&mut self, buffer: &[u8]) -> Result<(), DisplayError> {
        self.send_command(SET_X_POINTER, Some(&[0x00])).await?;
        self.send_command(SET_Y_POINTER, Some(&[0x00, 0x00]))
            .await?;
        self.send_command(WRITE_RAM, Some(buffer)).await?;
        self.set_update_sequence().await?;
        self.send_command(REFRESH_PANEL, None).await
    }

    async fn enable_backlight(&mut self) -> Result<(), DisplayError> {
        if let Some(pin) = self.backlight_pin.as_mut() {
            pin.set_high()
                .map_err(|_| DisplayError::BacklightPinFailure)?;
        }
        Ok(())
    }

    async fn disable_backlight(&mut self) -> Result<(), DisplayError> {
        if let Some(pin) = self.backlight_pin.as_mut() {
            pin.set_low()
                .map_err(|_| DisplayError::BacklightPinFailure)?;
        }
        Ok(())
    }
}
