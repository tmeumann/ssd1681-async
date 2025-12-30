#[derive(Debug)]
pub enum DisplayError {
    SpiFailure,
    BusyPinFailure,
    DataCommandPinFailure,
    BacklightPinFailure,
    ResetPinFailure,
    DeviceBusy,
}
