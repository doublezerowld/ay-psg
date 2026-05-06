#[cfg(feature = "read")]
use core::error::Error;

/// PSG-related errors related to invalid parameters and chip state.
#[derive(Debug)]
pub enum Error {
    InvalidClockFrequency(u32),
    FrequencyOutOfRange(f32),
    TonePeriodOutOfRange(u16),
    NoiseFrequencyOutOfRange(u8),
    LevelOutOfRange(u8),
    OctaveOutOfRange(u8),
    RegisterOutOfRange(u8),
    DivisionByZero,
    #[cfg(feature = "read")]
    ReadError(Error),
}
