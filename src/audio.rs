/// Reference pitch of A=440.0Hz.
const REFERENCE_PITCH: f32 = 440.0;

/// One of the YM2149's 3 audio channels.
#[derive(Debug, Clone, Copy)]
pub enum AudioChannel {
    A,
    B,
    C
}

impl AudioChannel {
    pub fn index(&self) -> usize {
        self.clone() as usize
    }
}

/// This helper struct contains data about the YM2149's channels' current states.
#[derive(Debug, Clone, Copy)]
pub struct AudioChannelData {
    pub address: u8,
    pub enabled: bool,
    pub level: u8,
    pub pitch_bend: f32,
    pub last_note: Option<Note>,
}

impl AudioChannelData {
    /// Create self.
    pub fn new(address: u8) -> Self {
        Self {
            address: address,
            enabled: false,
            level: 0,
            pitch_bend: 0.0,
            last_note: None,
        }
    }

    /// Set a channel's pitch bend.
    #[allow(unused)]
    pub fn set_pitch_bend(&mut self, byte1: u8, byte2: u8) {
        let new: f32 = (((byte2 as u16) << 7) + byte1 as u16).into();
        let as_semitones: f32 = (new - 8192.0) / 1024.0;
        self.pitch_bend = as_semitones;
    }
}

/// An accidental, represented by an i8 value that corresponds to the offset in quarter tones.
#[derive(Debug, Clone, Copy)]
#[repr(i8)]
pub enum Accidental {
    Natural = 0,
    Sharp = 2,
    Flat = -2,
    MicroSharp = 1,
    MicroFlat = -1,
}

impl From<Accidental> for f32 {
    fn from(acc: Accidental) -> f32 {
        (acc as i8) as f32 / 2.0
    }
}

/// One of the 7 white keys in the C Major scale.
#[derive(Debug, Clone, Copy)]
#[repr(i8)]
pub enum BaseNote {
    C = -9,
    D = -7,
    E = -5,
    F = -4,
    G = -2,
    A = 0,
    B = 2,
}

impl From<BaseNote> for f32 {
    fn from(bn: BaseNote) -> f32 {
        bn as i8 as f32
    }
}

/// A musical note.
///
/// Example code:
/// ```no_run
/// use ym2149_core::audio::{Note, BaseNote};
///
/// let a_4 = Note::new(
///     BaseNote::A,
///     4,
///     None
/// );
/// ```
#[derive(Debug, Clone, Copy)]
pub struct Note {
    base_note: BaseNote,
    octave: u8,
    accidental: Option<Accidental>,
    offset: f32,
}

impl Note {
    /// Creates a new [Note](#Note) from a [BaseNote](#BaseNote), octave, and optionally an [Accidental](#Accidental)
    pub fn new(base_note: BaseNote, octave: u8, accidental: Option<Accidental>) -> Self {
        Self {
            base_note: base_note,
            octave: octave,
            accidental: accidental,
            offset: 0.0,
        }
    }

    /// Transposes a note +`semitones` semitones. The type of semitones is f32 because I wanted to
    /// allow precise control of the pitch to cover all use cases.
    pub fn transpose(self, semitones: f32) -> Self {
        Self {
            offset: self.offset + semitones,
            ..self
        }
    }

    /// Returns the frequency of this note in Hertz.
    pub fn as_hz(&self) -> u32 {
        // f = f0 * 2 ^ (n / 12) | f0 - reference pitch, n - semitones away from ref.
        use libm::{powf, roundf};

        let distance_a4: f32 = f32::from(self.base_note)
            + f32::from(self.accidental.unwrap_or(Accidental::Natural))
            + (self.octave.clamp(0, 14) as f32 - 4.0) * 12.0
            + self.offset;

        roundf(REFERENCE_PITCH * powf(2.0, distance_a4 / 12.0)) as u32
    }
}
