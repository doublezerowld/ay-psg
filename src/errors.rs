/// YM2149-specific errors from invalid parameters and chip state.
#[derive(Debug)]
pub enum Error {
    InvalidClockFrequency(u32),
    TonePeriodOutOfRange(u16),
    NoiseFrequencyOutOfRange(u8),
    OctaveOutOfRange(u8),
    RegisterOutOfRange(u8),
    DivisionByZero,
    NoLastNote
}
