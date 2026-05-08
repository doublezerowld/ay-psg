/// PSG-related errors related to invalid parameters and chip state.
#[derive(Debug)]
pub enum Error {
    InvalidClockFrequency(u32),
    ToneFrequencyOutOfRange(f32),
    TonePeriodOutOfRange(u16),
    NoisePeriodOutOfRange(u8),
    NoiseFrequencyOutOfRange(f32),
    LevelOutOfRange(u8),
    OctaveOutOfRange(u8),
    RegisterOutOfRange(u8),
    DivisionByZero,
    #[cfg(feature = "read")]
    ReadError,
}
