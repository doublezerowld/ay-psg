//! registers.rs
//!

use core::fmt::Display;

pub const READABLE_REG_NAMES: [&'static str; 16] = [
    "Frequency of channel A, 8 bit fine tone adjustment",
    "Frequency of channel A, 4 bit rough tone adjustment",
    "Frequency of channel B, 8 bit fine tone adjustment",
    "Frequency of channel B, 4 bit rough tone adjustment",
    "Frequency of channel C, 8 bit fine tone adjustment",
    "Frequency of channel C, 4 bit rough tone adjustment",
    "Frequency of noise, 5 bit noise frequency",
    "I/O port and mixer settings",
    "Level of channel A",
    "Level of channel B",
    "Level of channel C",
    "Frequency of envelope, 8 bit fine adjustment",
    "Frequency of envelope, 8 bit rough adjustment",
    "Shape of envelope",
    "Data of I/O port A",
    "Data of I/O port B",
];

/// One of the 16 registers (0-15) of an AY-3-8910.
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
    /// ---
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
    /// use ay_psg::{
    ///     command::{Command, CommandOutput},
    ///     register::Register,
    ///     chip::PSG
    /// };
    ///
    /// struct DebugWriter;
    ///
    /// impl CommandOutput for DebugWriter {
    ///     fn execute(&mut self, command: Command) {
    ///         let arr = command.as_array();
    ///         println!("Writing 0b{:08b} to register 0b{:08b}.", arr[0], arr[1]);
    ///     }
    /// }
    ///
    /// let mut chip = PSG::new(
    ///     DebugWriter{},
    ///     2_000_000,
    /// ).expect("Error building chip");
    ///
    /// chip.command(
    ///     Register::IoPortMixerSettings,
    ///     0b11111110
    /// );
    /// ```
    IoPortMixerSettings,

    /// **Level of channel A**
    /// ---
    ///
    /// From the datasheet (formats identical for ALevel, BLevel and CLevel):
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
    /// Same bit layout as [`ALevel`]
    BLevel,

    /// **Level of channel C**
    ///
    /// Same bit layout as [`ALevel`]
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

/// Helper const, used to get a level register from an index in range 0..2.
pub const LEVEL_REGS: [Register; 3] = [Register::ALevel, Register::BLevel, Register::CLevel];

/// Helper trait implemented for u8 and crate::Register to make writing to registers easier.
pub trait RegisterIndex {
    fn address(self) -> u8;
}

impl RegisterIndex for u8 {
    fn address(self) -> u8 {
        self.clamp(0, 15)
    }
}

impl RegisterIndex for Register {
    fn address(self) -> u8 {
        (self as u8).clamp(0, 15)
    }
}

impl Display for Register {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{}", READABLE_REG_NAMES[self.clone() as usize])
    }
}
