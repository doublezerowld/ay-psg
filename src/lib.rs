/*
 *
 *
 *  █████ █████ ██████   ██████  ████████  ████  █████ █████   ████████
 * ░░███ ░░███ ░░██████ ██████  ███░░░░███░░███ ░░███ ░░███   ███░░░░███
 *  ░░███ ███   ░███░█████░███ ░░░    ░███ ░███  ░███  ░███ █░███   ░███
 *   ░░█████    ░███░░███ ░███    ███████  ░███  ░███████████░░█████████
 *    ░░███     ░███ ░░░  ░███   ███░░░░   ░███  ░░░░░░░███░█ ░░░░░░░███
 *     ░███     ░███      ░███  ███      █ ░███        ░███░  ███   ░███
 *     █████    █████     █████░██████████ █████       █████ ░░████████
 *    ░░░░░    ░░░░░     ░░░░░ ░░░░░░░░░░ ░░░░░       ░░░░░   ░░░░░░░░
 *
 *                   (c) vw.dvw 2026, MIT or Apache-2.0
 *
*/

//! Abstraction layer for YM2149-adjacent sound chips.
//!
//! The core crate contains ...
//!
//! **When in doubt, check the specsheet!**
#![no_std]
use core::convert::Into;

pub mod audio;
use audio::*;

/// One of the 16 registers (0-15) of the YM2149 sound chip.
///
/// Used to select which register to write / read.
/// Each register controls different aspects of tone generation, noise, mixing,
/// amplitude, and envelope.
///
/// Check the datasheet / docs for detailed information.
#[repr(u8)]
#[derive(Debug, Clone, Copy)]
pub enum Register {
    /// Frequency of channel A: 8 bit fine tone adjustment
    AFreq8bitFinetone,
    /// Frequency of channel A: 4 bit rough tone adjustment
    ///
    /// `Mask: 0x0F`
    AFreq4bitRoughtone,

    /// Frequency of channel B: 8 bit fine tone adjustment
    BFreq8bitFinetone,
    /// Frequency of channel B: 4 bit rough tone adjustment
    ///
    /// `Mask: 0x0F`
    BFreq4bitRoughtone,

    /// Frequency of channel C: 8 bit fine tone adjustment
    CFreq8bitFinetone,
    /// Frequency of channel C: 4 bit rough tone adjustment
    ///
    /// `Mask: 0x0F`
    CFreq4bitRoughtone,

    /// Frequency of noise: 5 bit noise frequency
    ///
    /// `Mask: 0x1F`
    NoiseFreq5bit,

    /// **I/O Port and mixer settings**
    ///
    /// From the datasheet:
    /// - Sound is output when '0' is written to the register.
    /// - Selection of input/output for the I/O ports is determined by bits B7 and B6 of register R7.
    /// - Input is selected when '0' is written to the register bits.
    ///
    /// Bit:    | B7  | B6  | B5  | B4  | B3  | B2  | B1  | B0  |
    /// --------|-----|-----|-----|-----|-----|-----|-----|-----|
    /// Type:   | I/O | I/O |Noise|Noise|Noise|Tone |Tone |Tone |
    /// Channel:| IOB | IOA |  C  |  B  |  A  |  C  |  B  |  A  |
    ///
    ///
    /// **Example:**
    /// ```no_run
    /// // Enables only channel A, with IOA and IOB functioning as outputs.
    /// chip.command(
    ///     Registers::IoPortMixerSettings,
    ///     0b11111110
    /// );
    /// ```
    IoPortMixerSettings,

    /// **Level of channel A**
    /// ---
    /// **Level control** (formats identical for ALevel, BLevel and CLevel)
    ///
    /// From the datasheet:
    /// - Mode M selects whether the level is fixed (when M = 0) or variable (M = 1).
    /// - When M = 0, the level is determined from one of 16 by level selection signals L3, L2, L1, and L0 which compromise the lower four bits.
    /// - When M = 1, the level is determined by the 5 bit output of E4, E3, E2, E1, and E0 of the envelope generator of the SSG.
    ///
    /// | B7 (MSB)  | B6  | B5  | B4  | B3  | B2  | B1  | B0  |
    /// |-----------|-----|-----|-----|-----|-----|-----|-----|
    /// | N/A       | N/A | N/A |  M  | L3  | L2  | L1  | L0  |
    ALevel,

    /// **Level of channel B**
    ///
    /// Same format as [ALevel](#alevel)
    BLevel,

    /// **Level of channel C**
    ///
    /// Same format as [ALevel](#alevel)
    CLevel,

    /// Frequency of envelope: 8 bit fine adjustment
    EFreq8bitFineAdj,
    /// Frequency of envelope: 8 bit rough adjustment
    EFreq8bitRoughAdj,
    /// Shape of envelope
    EShape,
    /// Data of I/O port A
    DataIoA,
    /// Data of I/O port B
    DataIoB,
}

/// Helper trait implemented for u8 and crate::Register to make writing to registers easier
pub trait ValidRegister {
    fn address(self) -> u8;
}

impl ValidRegister for u8 {
    fn address(self) -> u8 {
        self.clamp(0, 15)
    }
}

impl ValidRegister for Register {
    fn address(self) -> u8 {
        (self as u8).clamp(0, 15)
    }
}

/// A command contains a value to be written to a specific register of the YM2149.
#[derive(Debug)]
pub struct Command {
    pub register: u8,
    pub value: u8,
}

#[allow(unused)]
impl Command {
    fn new(register: u8, value: u8) -> Self {
        Self { register, value }
    }
    
    fn as_array(&self) -> [u8; 2] {
        [self.register, self.value]
    }
}

/// Helper trait that lets you implement an "output" for the commands that the driver generates.
/// 
/// Example:
/// ```no_run
/// struct DebugWriter {};
/// impl CommandOutput for DebugWriter {
///     fn execute(&mut self, command: Command) {
///         let arr = command.as_array();
///         println!("Writing 0b{:08b} to register 0b{:08b}.", arr[0], arr[1]); 
///     };
/// }
/// ```
pub trait CommandOutput {
    fn execute(&mut self, command: Command);
}

/// One of the two modes of the I/O ports.
#[repr(u8)]
pub enum IoMode {
    Input = 0,
    Output = 1,
}

/// IO port and mixer settings.
///
/// Note: Whereas the YM2149 enables tone / noise generators when the register stores
/// a value of 0 (false), I wrote the code in a way to seem more logical. The fields 
/// that take a `bool` argument instead enable a generator when its value is `true`.
pub struct IoPortMixerSettings {
    pub gpio_port_a_mode: IoMode,
    pub gpio_port_b_mode: IoMode,
    pub noise_ch_c: bool,
    pub noise_ch_b: bool,
    pub noise_ch_a: bool,
    pub tone_ch_c: bool,
    pub tone_ch_b: bool,
    pub tone_ch_a: bool,
}

impl IoPortMixerSettings {
    /// Returns a u8 containing the settings that can be written directly to register 7 of the YM2149.
    pub fn as_u8(self) -> u8 {
        let self_array = [
            self.gpio_port_a_mode as u8 == 0,
            self.gpio_port_b_mode as u8 == 0,
            self.noise_ch_c,
            self.noise_ch_b,
            self.noise_ch_a,
            self.tone_ch_c,
            self.tone_ch_b,
            self.tone_ch_a,
        ];

        let mut byte = 0_u8;

        for i in 0..8 {
            byte += (!(self_array[i as usize]) as u8) << 7 - i;
        }

        byte
    }
}

impl Default for IoPortMixerSettings {
    fn default() -> Self {
        IoPortMixerSettings {
            gpio_port_a_mode: IoMode::Output,
            gpio_port_b_mode: IoMode::Output,
            noise_ch_c: false,
            noise_ch_b: false,
            noise_ch_a: false,
            tone_ch_c: false,
            tone_ch_b: false,
            tone_ch_a: false,
        }
    }
}

/// The four modes of the bus control decoder.
///
/// Bus control decoder table, no redundancy:
///
/// | Mode         | BDIR | BC2 | BC1 |
/// | ------------ | ---- | --- | --- |
/// | **INACTIVE** |  0   |  1  |  0  |
/// | **READ**     |  0   |  1  |  1  |
/// | **WRITE**    |  1   |  1  |  0  |
/// | **ADDRESS**  |  1   |  1  |  1  |
#[repr(u8)]
pub enum Mode {
    /// DA7~DA0 has high impedance.
    INACTIVE,
    /// DA7~DA0 set to output mode, and contents of register currently being addressed are output.
    ///
    /// ---
    /// ### Warning!
    ///
    /// Mode::READ makes the chip output 5V to the data bus. It is **STRONGLY** recommended
    /// to use a level shifter in order to prevent permanent damage to your board.
    READ,
    /// DA7~DA0 set to input mode, and data is written to register currently being addressed.
    WRITE,
    /// DA7~DA0 set to input mode, and address is fetched from register array.
    ADDRESS,
}

/// One of the two GPIO ports of the YM2149.
#[repr(u8)]
pub enum IoPort {
    A = 0xE,
    B = 0xF,
}

// =========================================================
// ====================== CHIP STRUCT ======================
// =========================================================

/// A YM2149 chip struct.
/// 
/// The master_clock_frequency value is used to convert a frequency into a tone period by .tone_hz()
///
/// Example code:
/// ```no_run
/// let chip = YM2149::new(
///     DebugWriter,
///     2_000_000,
/// )
/// 
/// chip.setup_io_and_mixer(
///     IoPortMixerSettings {
//         tone_ch_a: true,
//         ..Default::default(),
///     }
/// );
/// ```
pub struct YM2149<C>
where
    C: CommandOutput,
{
    command_output: C,
    master_clock_frequency: u32,
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
        let bytes: [u8; 2] = period.to_le_bytes();
        let register_pair_index = channel as u8 * 2;

        self.command(register_pair_index, bytes[0]); // Fine tone, 8 bits
        self.command(register_pair_index + 1, bytes[1]); // Rough tone, 4 bits
    }

    /// Play a tone of a given frequency in Hz on an [AudioChannel](#AudioChannel).
    pub fn tone_hz(&mut self, channel: AudioChannel, frequency: u32) {
        let tp: u32 = self.master_clock_frequency / (16 * frequency);
        self.tone(channel, tp as u16); // Take lowest 16 bits
    }

    /// Play a [Note](#Note) on an [AudioChannel](#AudioChannel).
    pub fn play_note(&mut self, channel: AudioChannel, note: &Note) {
        let note_f = note.as_hz();
        self.tone_hz(channel, note_f);
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
        self.command(8 + channel as u8, level & 0x1F);
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
    fn read(&mut self, register: Register) -> u8 {
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
    fn read_io(&mut self, port: IoPort) -> u8 {
        unimplemented!("Mode::READ and any functions associated with it are not yet usable.");
    }
}
