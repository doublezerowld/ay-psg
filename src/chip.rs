// Imports
use crate::audio::{AudioChannel, Note};
use crate::command::{Command, CommandOutput};
use crate::envelopes::{Envelope, EnvelopeFrequency};
use crate::errors::Error;
use crate::io::{IoPort, IoPortMixerSettings};
use crate::register::{LEVEL_REGS, Register, ValidRegister};

// =========================================================
// ====================== CHIP STRUCT ======================
// =========================================================

/// A YM2149 chip struct.
///
/// The master_clock_frequency value is used to convert a frequency into a tone period by .tone_hz()
///
/// Example code:
/// ```no_run
/// use ym2149_core::{
///     command::{Command, CommandOutput},
///     io::IoPortMixerSettings,
///     chip::YM2149
/// };
///
/// struct DebugWriter;
///
/// impl CommandOutput for DebugWriter {
///     fn execute(&mut self, command: Command) {
///         print!("Register 0b{:08b} ({:?}), ", command.register, command.register);
///         println!("0b{:08b} ({:?})", command.value, command.value);
///     }
/// }
///
/// let mut chip = YM2149::new(
///     DebugWriter{},
///     2_000_000,
/// ).expect("Error building chip");
///
/// chip.setup_io_and_mixer(
///     IoPortMixerSettings {
///         tone_ch_a: true,
///         ..Default::default()
///     }
/// );
/// ```
#[derive(Debug)]
pub struct YM2149<C>
where
    C: CommandOutput,
{
    pub command_output: C,
    pub master_clock_frequency: u32,
    pub last_used_channel: Option<usize>,
}

impl<C> YM2149<C>
where
    C: CommandOutput,
{
    /// Create a new struct for the YM2149.
    pub fn new(command_output: C, master_clock_frequency: u32) -> Result<Self, Error> {
        match master_clock_frequency {
            1_000_000..=4_000_000 => Ok(Self {
                command_output,
                master_clock_frequency,
                last_used_channel: None
            }),
            _ => Err(Error::InvalidClockFrequency(master_clock_frequency)) //"The master_clock_frequency must be between 1MHz-4MHz!")
        }
    }

    /// Send a [Command](#Command).
    pub fn command<R: ValidRegister + Copy>(&mut self, register: R, value: u8) {
        self.command_output
            .execute(Command::new(register.address(), value));
    }

    /// Setup the IO ports and the internal mixer according to the IoPortMixerSettings specified.
    pub fn setup_io_and_mixer(&mut self, settings: IoPortMixerSettings) {
        self.command(Register::IoPortMixerSettings, settings.as_u8());
    }

    /// Write a value to one of the chip's [GPIO ports](#IoPort).
    /// Note: This is a simple helper function, equivalent to ``self.command(port as u8, value);``
    pub fn write_io(&mut self, port: IoPort, value: u8) {
        self.command(port as u8, value);
    }

    /// Set the envelope generator's frequency.
    pub fn set_envelope_frequency(&mut self, frequency: EnvelopeFrequency) -> Result<u16, Error> {
        let r: u16 = frequency.as_ep(self.master_clock_frequency)?;

        let rough: u8 = (r >> 8) as u8; // High byte
        let fine: u8 = r as u8; // Low byte

        self.command(Register::EFreq8bitRoughAdj, rough);
        self.command(Register::EFreq8bitFineAdj, fine);

        Ok(r)
    }

    /// Set the envelope generator's shape.
    pub fn set_envelope_shape(&mut self, envelope: &Envelope) {
        self.command(0xD, envelope.into());
    }

    /// Play a tone with a TP of `period` on an [AudioChannel](#AudioChannel).
    ///
    /// The formula for the frequency is
    /// ``f = fMaster / (16 * TP)``, where:
    ///     - f: target frequency
    ///     - fMaster: master clock frequency
    ///     - TP: tone period
    pub fn tone(&mut self, channel: AudioChannel, period: u16) -> Result<(), Error> {
        if period > 2_u16.pow(12) {
            return Err(Error::TonePeriodOutOfRange(period));
        }

        let bytes: [u8; 2] = period.to_le_bytes();
        let register_pair_index = channel as u8 * 2;

        self.command(register_pair_index, bytes[0]); // Fine adjustment, 8 bits
        self.command(register_pair_index + 1, bytes[1]); // Rough adjustment, 4 bits
        self.last_used_channel = Some(channel.index());

        Ok(())
    }

    /// Play a tone of a given frequency in Hz on an [AudioChannel](#AudioChannel).
    pub fn tone_hz(&mut self, channel: AudioChannel, frequency: u32) -> Result<(), Error> {
        if frequency == 0 {
            return Err(Error::DivisionByZero);
        }

        let tp: u32 = self.master_clock_frequency / (16 * frequency);
        self.tone(channel, tp as u16)?; // Take lowest 16 bits

        Ok(())
    }

    /// Play a [Note](#Note) on an [AudioChannel](#AudioChannel).
    pub fn play_note(&mut self, channel: AudioChannel, note: &Note) -> Result<(), Error> {
        self.tone_hz(
            channel,
            note.as_hz()
        )?;

        Ok(())
    }

    /// Play a [Note](#Note) on an [AudioChannel](#AudioChannel) with a given [Envelope](#Envelope).
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
    pub fn level(&mut self, channel: AudioChannel, level: u8) {
        self.command(LEVEL_REGS[channel.index()] as u8, level & 0x1F);
    }

    // ============================================================
    // ========================= THE VOID =========================
    // ============================================================
    // (All you'll find here is unimplemented / todo functionality)

    #[allow(unused)]
    /// Reads a value from a given register and outputs it to the data bus.
    ///
    /// ---
    /// # Warning!
    ///
    /// Mode::READ makes the chip output 5V to the data bus. It is **STRONGLY** recommended
    /// to use a level shifter in order to prevent permanent damage to your board.
    ///
    /// This method is **unimplemented** *(at least, not for now...)*
    ///
    /// Feel free to try implementing it yourself, at your own risk.
    fn read(&self, register: Register) -> u8 {
        unimplemented!("Mode::READ and any functions associated with it are not yet usable.");
    }

    #[allow(unused)]
    /// Reads a value from a given I/O port and outputs it to the data bus.
    ///
    /// ---
    /// # Warning!
    ///
    /// Mode::READ makes the chip output 5V to the data bus. It is **STRONGLY** recommended
    /// to use a level shifter in order to prevent permanent damage to your board.
    ///
    /// This method is **unimplemented** *(at least, not for now...)*
    ///
    /// Feel free to try implementing it yourself, at your own risk.
    fn read_io(&self, port: IoPort) -> u8 {
        unimplemented!("Mode::READ and any functions associated with it are not yet usable.");
    }
}
