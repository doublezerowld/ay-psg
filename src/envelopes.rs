use crate::errors::Error;

// An enum for all EnvelopeShape types
#[derive(Debug, Clone, Copy)]
pub enum Envelope {
    Builtin(BuiltinEnvelopeShape),
    InvertedBuiltin(BuiltinEnvelopeShape),
    CustomBuiltin(u8),
    RawEnvelope(RawEnvelope),
}

impl From<&Envelope> for u8 {
    fn from(value: &Envelope) -> Self {
        use Envelope::{Builtin, CustomBuiltin, InvertedBuiltin, RawEnvelope};

        match value {
            Builtin(builtin) => *builtin as u8,
            InvertedBuiltin(builtin) => (*builtin as u8) ^ (0b00000100),
            CustomBuiltin(n) => *n,

            #[allow(unused)]
            RawEnvelope(raw) => unimplemented!(),
        }
    }
}

/// A helper enum for setting the envelope's shape.
///
/// To invert the shape use Envelope::InvertedBuiltin(BuiltinEnvelopeShape).
#[derive(Debug, Clone, Copy)]
#[repr(u8)]
pub enum BuiltinEnvelopeShape {
    /// Fade out and hold low
    FadeOut = 0b00001001,
    /// Fade in and hold high
    FadeIn = 0b00001101,
    /// Fade in then hold low
    Tooth = 0b00001111,
    /// Fade in every repetition
    Saw = 0b00001100,
    /// Alternate between fade out and fade in
    Triangle = 0b00001110,
}

/// A raw envelope for more precise control of channel levels.
///
/// It consists of a `data` field - which is an array of u8 values with length 4096,
/// and a length given in beats.
#[allow(unused)]
#[derive(Debug, Clone, Copy)]
pub struct RawEnvelope {
    data: [u8; 4096],
    length_beats: u8,
}

#[allow(unused)]
impl RawEnvelope {
    fn invert(&mut self) {
        for i in 0..4096 {
            self.data[i] = 0xF - self.data[i];
        }
    }

    fn scale(&mut self, scale: f32) {
        for i in 0..4096 {
            let scaled = (self.data[i] as f32 * scale).clamp(0.0, 255.0) as u8;
            self.data[i] = scaled;
        }
    }

    fn offset(&mut self, offset: i8) {
        for i in 0..4096 {
            self.data[i] += offset as u8;
        }
    }
}

/// A helper enum for setting the envelope repetition frequency f_e.
#[derive(Debug)]
pub enum EnvelopeFrequency {
    Hertz(u16),
    BeatsPerMinute(u16),
    Integer(u16),
}

impl EnvelopeFrequency {
    /// Returns the EnvelopeFrequency as a u16 value for registers 11 (LSB) and 12 (MSB).
    pub fn as_ep(self, master_clock_frequency: u32) -> Result<u16, Error> {
        match self {
            Self::Hertz(f_e) => {
                if f_e == 0 {
                    return Err(Error::DivisionByZero);
                }
                let period = master_clock_frequency / (256 * f_e as u32);
                period.try_into().map_err(|_| Error::TonePeriodOutOfRange(period as u16))
            }
            Self::BeatsPerMinute(bpm) => {
                let hz = Self::Hertz(bpm).as_ep(master_clock_frequency)?;
                (60 * hz).try_into().map_err(|_| Error::TonePeriodOutOfRange(0))
            }
            Self::Integer(x) => Ok(x),
        }
    }
}
