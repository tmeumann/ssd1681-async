#[derive(Debug)]
pub enum DisplayError {
    Spi,
    Busy,
    DataCommand,
    Backlight,
    Reset,
}
