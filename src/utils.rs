use core::convert::Infallible;
use embedded_hal::digital::{ErrorType, OutputPin};

pub enum NeverPin {}

impl ErrorType for NeverPin {
    type Error = Infallible;
}

impl OutputPin for NeverPin {
    fn set_low(&mut self) -> Result<(), Self::Error> {
        Ok(())
    }
    fn set_high(&mut self) -> Result<(), Self::Error> {
        Ok(())
    }
}
