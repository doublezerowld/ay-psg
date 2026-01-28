/// One of the two modes of the I/O ports.
#[repr(u8)]
#[derive(Debug, Clone, Copy)]
pub enum IoPortMode {
    Input = 0,
    Output = 1,
}

/// One of the two GPIO ports of the YM2149.
#[repr(u8)]
#[derive(Debug, Clone, Copy)]
pub enum IoPort {
    A = 0xE,
    B = 0xF,
}

/// IO port and mixer settings.
///
/// Note: Whereas the YM2149 enables tone / noise generators when the register stores
/// a value of 0 (false), I wrote the code in a way to seem more logical. The fields
/// that take a `bool` argument instead enable a generator when its value is `true`.
#[derive(Debug)]
pub struct IoPortMixerSettings {
    pub gpio_port_a_mode: IoPortMode,
    pub gpio_port_b_mode: IoPortMode,
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
            gpio_port_a_mode: IoPortMode::Output,
            gpio_port_b_mode: IoPortMode::Output,
            noise_ch_c: false,
            noise_ch_b: false,
            noise_ch_a: false,
            tone_ch_c: false,
            tone_ch_b: false,
            tone_ch_a: false,
        }
    }
}

/// The four modes of the YM2149's bus control decoder.
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
#[derive(Debug)]
pub enum ChipMode {
    /// DA7~DA0 has high impedance.
    INACTIVE,
    /// DA7~DA0 set to output mode, and contents of register currently being addressed are output.
    ///
    /// ---
    /// ### Warning!
    ///
    /// ``Mode::READ`` makes the chip output 5V to the data bus. If you're using this crate in an embedded project,
    /// make sure that 5V isn't too high for your board! If it is, you can use a level shifter to prevent damage to your board.
    READ,
    /// DA7~DA0 set to input mode, and data is written to register currently being addressed.
    WRITE,
    /// DA7~DA0 set to input mode, and address is fetched from register array.
    ADDRESS,
}
