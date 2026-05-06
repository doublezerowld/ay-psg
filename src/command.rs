// command.rs
//! Provides the Command struct and the CommandOutput trait.

use crate::register::{READABLE_REG_NAMES, RegisterIndex};
use core::fmt::Display;

/// A command contains a value to be written to a specific register of the chip.
#[derive(Debug, Clone, Copy)]
pub struct Command {
    pub register: u8,
    pub value: u8,
}

#[allow(unused)]
impl Command {
    /// Creates a new [`Command`].
    pub fn new<R: RegisterIndex>(register: R, value: u8) -> Self {
        Self {
            register: register.address(),
            value,
        }
    }

    /// Returns self as a tuple (register-value pair) containing two bytes `0` for register, and `1` for value.
    pub fn as_tuple(&self) -> (u8, u8) {
        (self.register, self.value)
    }
}

impl Display for Command {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let readable_name: &'static str = READABLE_REG_NAMES[self.register as usize];
        write!(f, "{} set to 0b{:08b}", readable_name, self.value)
    }
}

/// This trait lets you define how [`Commands`](Command) should be dealt with.
///
/// Example:
/// ```rust
/// use ay_psg::command::{Command, CommandOutput};
///
/// struct DebugWriter;
///
/// impl CommandOutput for DebugWriter {
///     fn execute(&mut self, command: Command) {
///         let arr = command.as_array();
///         println!("Writing 0b{:08b} to register 0b{:08b}.", arr[0], arr[1]);
///     }
/// }
/// ```
pub trait CommandOutput {
    fn execute(&mut self, command: Command);
}
