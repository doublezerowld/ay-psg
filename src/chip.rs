// Imports
use crate::audio::{AudioChannel, AudioChannelData, Note};
use crate::command::{Command, CommandOutput};
use crate::envelopes::{Envelope, EnvelopeFrequency};
use crate::io::{IoPort, IoPortMixerSettings};
use crate::registers::{LEVEL_REGS, Register, ValidRegister};

// =========================================================
// ====================== CHIP STRUCT ======================
// =========================================================

/// A YM2149 chip struct.
///
/// The master_clock_frequency value is used to convert a frequency into a tone period by .tone_hz()
///
/// Example code:
/// ```no_run
/// use ym2149_core::{Command, CommandOutput, IoPortMixerSettings, YM2149};
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
/// );
///
/// chip.setup_io_and_mixer(
///     IoPortMixerSettings {
///         tone_ch_a: true,
///         ..Default::default()
///     }
/// );
/// ```
pub struct YM2149<C>
where
    C: CommandOutput,
{
    command_output: C,
    master_clock_frequency: u32,
    pub channel_data: [AudioChannelData; 3],
    pub last_used_channel: Option<usize>,
}

impl<C> YM2149<C>
where
    C: CommandOutput,
{
    /// Create a new struct for the YM2149.
    pub fn new(command_output: C, master_clock_frequency: u32) -> Self {
        Self {
            command_output,
            master_clock_frequency,
            channel_data: [
                AudioChannelData::new(0),
                AudioChannelData::new(1),
                AudioChannelData::new(2)
            ],
            last_used_channel: None
        }
    }

    /// Send a [Command](#Command).
    pub fn command<R: ValidRegister + Copy>(&mut self, register: R, value: u8) {
        self.command_output
            .execute(Command::new(register.address(), value));
    }

    /// Setup the IO ports and the internal mixer according to the IoPortMixerSettings specified.
    pub fn setup_io_and_mixer(&mut self, settings: IoPortMixerSettings) {
        self.channel_data[0].enabled = settings.tone_ch_a;
        self.channel_data[1].enabled = settings.tone_ch_b;
        self.channel_data[2].enabled = settings.tone_ch_c;

        self.command(Register::IoPortMixerSettings, settings.as_u8());
    }

    /// Write a value to one of the chip's [GPIO ports](#IoPort).
    /// Note: This is a simple helper function, equivalent to ``self.command(port as u8, value);``
    pub fn write_io(&mut self, port: IoPort, value: u8) {
        self.command(port as u8, value);
    }

    /// Set the envelope generator's frequency.
    pub fn set_envelope_frequency(&mut self, frequency: EnvelopeFrequency) {
        let r: u16 = frequency.as_ep(self.master_clock_frequency);

        let rough: u8 = (r >> 8) as u8; // High byte
        let fine: u8 = r as u8; // Low byte

        self.command(Register::EFreq8bitRoughAdj, rough);
        self.command(Register::EFreq8bitFineAdj, fine);
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
    pub fn tone(&mut self, channel: AudioChannel, period: u16) {
        if !self.channel_data[channel.index()].enabled {
            return;
        }

        let bytes: [u8; 2] = period.to_le_bytes();
        let register_pair_index = self.channel_data[channel.index()].address * 2;

        self.command(register_pair_index, bytes[0]); // Fine tone, 8 bits
        self.command(register_pair_index + 1, bytes[1]); // Rough tone, 4 bits
        self.last_used_channel = Some(channel.index())
    }

    /// Play a tone of a given frequency in Hz on an [AudioChannel](#AudioChannel).
    pub fn tone_hz(&mut self, channel: AudioChannel, frequency: u32) {
        if !self.channel_data[channel.index()].enabled {
            return;
        }
        let tp: u32 = self.master_clock_frequency / (16 * frequency);
        self.tone(channel, tp as u16); // Take lowest 16 bits
    }

    /// Play a [Note](#Note) on an [AudioChannel](#AudioChannel).
    pub fn play_note(&mut self, channel: AudioChannel, note: &Note) {
        if !self.channel_data[channel.index()].enabled {
            return;
        }
        self.channel_data[channel.index()].last_note = Some(note.clone());

        self.tone_hz(
            channel,
            note.transpose(self.channel_data[channel.index()].pitch_bend).as_hz()
        );
    }

    /// Set an AudioChannel's pitch bend (takes a MIDI command).
    pub fn pitch_bend(&mut self, channel: AudioChannel, byte1: u8, byte2: u8) -> f32 {
        self.channel_data[channel.index()].set_pitch_bend(byte1, byte2);
        self.replay_last_note(channel);
        self.channel_data[channel.index()].pitch_bend
    }

    /// Replay the last played note on a given channel.
    pub fn replay_last_note(&mut self, channel: AudioChannel) {
        if self.channel_data[channel.index()].last_note.is_some() {
            self.play_note(channel, &self.channel_data[channel.index()].last_note.unwrap());
        }
    }

    /// Play a [Note](#Note) on an [AudioChannel](#AudioChannel) with a given [Envelope](#Envelope).
    pub fn play_note_with_envelope(
        &mut self,
        channel: AudioChannel,
        note: &Note,
        with_envelope: &Envelope,
    ) {
        self.play_note(channel, note);
        self.set_envelope_shape(with_envelope);
    }

    /// Set the frequency of the noise generator.
    ///
    /// Mask: 0x1F
    pub fn set_noise_freq(&mut self, frequency: u8) {
        self.command(Register::NoiseFreq5bit, frequency & 0x1F);
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
        self.channel_data[channel.index()].level = level;
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
