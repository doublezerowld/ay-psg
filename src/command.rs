/// A command contains a value to be written to a specific register of the YM2149.
#[derive(Debug)]
pub struct Command {
    pub register: u8,
    pub value: u8,
}

#[allow(unused)]
impl Command {
    /// Creates a new Command from a register (u8) and value (u8).
    pub fn new(register: u8, value: u8) -> Self {
        Self { register, value }
    }

    /// Returns self as an array containing two bytes, [0] for register, and [1] for value.
    pub fn as_array(&self) -> [u8; 2] {
        [self.register, self.value]
    }
}

/// Helper trait that lets you implement an "output" for the commands that the driver generates.
///
/// Example:
/// ```no_run
/// use ym2149_core::{Command, CommandOutput};
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
