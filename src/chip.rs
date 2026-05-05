// Imports
use crate::audio::{AudioChannel, Envelope, EnvelopeFrequency};
use crate::command::{Command, CommandOutput};
use crate::errors::Error;
use crate::io::{IoPort, IoPortMixerSettings};
#[cfg(feature = "notes")]
use crate::notes::Note;
use crate::register::{LEVEL_REGS, Register, RegisterIndex};

// =========================================================
// ====================== CHIP STRUCT ======================
// =========================================================

/// A YM2149 chip struct.
///
/// The master_clock_frequency value is used to convert a frequency into a tone period by .tone_hz()
///
/// Example code:
/// ```rust
/// use ym2149_core::io::IoPortMixerSettings;
/// use ym2149_core::{audio::AudioChannel, prelude::*};
///
/// struct DisplayWriter;
/// impl CommandOutput for DisplayWriter {
///     fn execute(&mut self, command: Command) {
///         println!("{}", command)
///     }
/// }
///
/// fn main() {
///     let out = DisplayWriter {};
///     let mut chip = YM2149::new(out, 2_000_000);
///
///     chip.setup_io_and_mixer(IoPortMixerSettings {
///         tone_ch_a: true,
///         ..Default::default()
///     });
///
///     chip.level(AudioChannel::A, 0xF).expect("Failed to set level");
///     chip.tone_hz(AudioChannel::A, 440.0).expect("Failed to set frequency");
/// }
/// ```
#[derive(Debug)]
pub struct YM2149<C>
where
    C: CommandOutput,
{
    pub command_output: C,
    pub master_clock_frequency: u32,
}

impl<C> YM2149<C>
where
    C: CommandOutput,
{
    /// Create a new struct for the YM2149.
    ///
    /// The datasheet specifies a master clock frequency range of 1-2MHz (2-4MHz with SEL low on YM2149 chips)
    pub fn new(command_output: C, master_clock_frequency: u32) -> Self {
        Self {
            command_output,
            master_clock_frequency,
        }
    }

    /// Send a [`command::Command`].
    pub fn command<R: RegisterIndex>(&mut self, register: R, value: u8) {
        self.command_output
            .execute(Command::new(register.address(), value));
    }

    /// Setup the IO ports and the internal mixer according to the [`IoPortMixerSettings`] specified.
    pub fn setup_io_and_mixer(&mut self, settings: IoPortMixerSettings) {
        self.command(Register::IoPortMixerSettings, settings.as_u8());
    }

    /// Write a value to one of the chip's [GPIO ports](IoPort).
    /// Note: This is a simple helper function, equivalent to ``self.command(port as u8, value);``
    pub fn write_io(&mut self, port: IoPort, value: u8) {
        self.command(port as u8, value);
    }

    /// Set the envelope generator's frequency using an [`EnvelopeFrequency`].
    pub fn set_envelope_frequency(&mut self, frequency: EnvelopeFrequency) -> Result<u16, Error> {
        let r: u16 = frequency.as_ep(self.master_clock_frequency)?;

        let rough: u8 = (r >> 8) as u8; // high byte
        let fine: u8 = r as u8; // low byte

        self.command(Register::EFreq8bitRoughAdj, rough);
        self.command(Register::EFreq8bitFineAdj, fine);

        Ok(r)
    }

    /// Set the envelope generator's shape from an [`Envelope`].
    pub fn set_envelope_shape(&mut self, envelope: &Envelope) {
        self.command(Register::EShape, envelope.into());
    }

    /// Play a tone with a TP of `period` on an [`AudioChannel`].
    ///
    /// The formula for the frequency is
    /// ``f = f_Master / (16 * TP)``, where:
    ///     - f: target frequency
    ///     - f_Master: master clock frequency
    ///     - TP: tone period
    pub fn tone(&mut self, channel: AudioChannel, period: u16) -> Result<(), Error> {
        if period > 2_u16.pow(12) {
            return Err(Error::TonePeriodOutOfRange(period));
        }

        let bytes: [u8; 2] = period.to_le_bytes();
        let register_pair_index = channel as u8 * 2;

        self.command(register_pair_index, bytes[0]); // Fine adjustment, 8 bits
        self.command(register_pair_index + 1, bytes[1]); // Rough adjustment, 4 bits
        Ok(())
    }

    /// Play a tone of a given frequency in Hz on an [AudioChannel](#AudioChannel).
    ///
    /// ***Basic usage:***
    /// ```no_run
    /// use ym2149_core::audio::AudioChannel;
    ///
    /// chip.tone_hz(AudioChannel::A, 440)?; // Ok::<(), crate::errors::Error>(())
    /// ```
    pub fn tone_hz(&mut self, channel: AudioChannel, frequency: f32) -> Result<u16, Error> {
        match frequency {
            // safety checks
            0.0 => Err(Error::DivisionByZero),
            ..0.0 => Err(Error::FrequencyOutOfRange(frequency)),
            _ => {
                // for valid frequencies
                let tp: f32 = self.master_clock_frequency as f32 / (16.0 * frequency);
                self.tone(channel, tp as u16)?; // Take lowest 16 bits

                Ok(tp as u16)
            }
        }
    }

    #[cfg(feature = "notes")]
    /// Play a [`Note`] on an [`AudioChannel`].
    pub fn play_note(&mut self, channel: AudioChannel, note: &Note) -> Result<f32, Error> {
        let hz = note.as_hz();
        self.tone_hz(channel, hz)?;
        Ok(hz)
    }

    #[cfg(feature = "notes")]
    /// Play a [`Note`] on an [`AudioChannel`] with a given [`Envelope`].
    pub fn play_note_with_envelope(
        &mut self,
        channel: AudioChannel,
        note: &Note,
        with_envelope: &Envelope,
    ) -> Result<(), Error> {
        self.play_note(channel, note)?;
        self.set_envelope_shape(with_envelope);
        Ok(())
    }

    /// Set the frequency of the noise generator.
    ///
    /// Mask: 0x1F
    pub fn set_noise_freq(&mut self, frequency: u8) -> Result<(), Error> {
        if frequency <= 0x1F {
            self.command(Register::NoiseFreq5bit, frequency);
            Ok(())
        } else {
            Err(Error::NoiseFrequencyOutOfRange(frequency))
        }
    }

    /// Set the volume of an [AudioChannel](#AudioChannel).
    ///
    /// **Note:** The channel level registers store 5 bits of data per channel.
    ///
    /// ---
    ///
    /// From the datasheet:
    /// - Mode M selects whether the level is fixed (when M = 0) or variable (M = 1).
    /// - When M = 0, the level is determined from one of 16 by level selection signals L3, L2, L1, and L0 which compromise the lower four bits.
    /// - When M = 1, the level is determined by the 5 bit output of E4, E3, E2, E1, and E0 of the envelope generator of the SSG.
    ///
    /// | B7 (MSB)  | B6  | B5  | B4  | B3  | B2  | B1  | B0  |
    /// |-----------|-----|-----|-----|-----|-----|-----|-----|
    /// | N/A       | N/A | N/A |  M  | L3  | L2  | L1  | L0  |
    pub fn level(&mut self, channel: AudioChannel, level: u8) -> Result<(), Error> {
        if level <= 0x1F {
            self.command(LEVEL_REGS[channel.index()] as u8, level & 0x1F);
            Ok(())
        } else {
            Err(Error::LevelOutOfRange(level))
        }
    }

    // ============================================================
    // ========================= THE VOID =========================
    // ============================================================
    // (All you'll find here is unimplemented / todo functionality)

    #[allow(unused)]
    #[cfg(feature = "read")]
    /// Reads a value from a given register and outputs it to the data bus.
    ///
    /// ---
    /// # Warning!
    ///
    /// Mode::READ makes the chip output 5V to the data bus. Make absolute sure your
    /// microcontroller can handle TTL voltager, or use a level shifter!
    ///
    /// This method is **unimplemented** *(at least, not for now...)*
    ///
    /// Feel free to try implementing it yourself, at your own risk.
    fn read(&self, register: Register) -> u8 {
        unimplemented!("Mode::READ and any functions associated with it are not yet usable.");
    }

    #[allow(unused)]
    #[cfg(feature = "read")]
    /// Reads a value from a given I/O port and outputs it to the data bus.
    ///
    /// ---
    /// # Warning!
    ///
    /// Mode::READ makes the chip output 5V to the data bus. Make absolute sure your
    /// microcontroller can handle TTL voltager, or use a level shifter!
    ///
    /// This method is **unimplemented** *(at least, not for now...)*
    ///
    /// Feel free to try implementing it yourself, at your own risk.
    fn read_io(&self, port: IoPort) -> u8 {
        unimplemented!("Mode::READ and any functions associated with it are not yet usable.");
    }
}
