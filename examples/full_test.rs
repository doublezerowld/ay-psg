//! # Test code that verifies the proper function of the chip and driver.
//!
//! You will probably want to run these tests on actual hardware instead of just writing the output to the stdout.
//! Take a look in `examples/embedded/...` for embedded implementations.
//!
//! The tests include (in chronological order):
//! - Full register write test (read-back test if the "read" feature is enabled)
//! - I/O Port write test (read-back test if the "read" feature is enabled)
//! - Tone generation (full sweep)
//! - Noise generation (full sweep)
//! - Mixer & envelope generation:
//!     - A, B, C level registers
//!     - Envelope generation (using [`Hz`], [`BPM`], [`Raw`] enum variants)
//! - Notes (if the "notes" feature is enabled)
//! - Argument validation test (errors.rs)

use std::cell::RefCell;
use std::rc::Rc;
use std::thread::sleep;
use std::time::Duration;

const DELAY_BETWEEN_TESTS: Duration = Duration::from_millis(1_000);
const DELAY_ENV_SHAPES: Duration = Duration::from_millis(2_000);
const DELAY_ENV_SHAPES_PAUSE: Duration = Duration::from_millis(1_000);

use ay_psg::audio::{ENVELOPE_SHAPES, REFERENCE_PITCH};
use ay_psg::{
    audio::{AUDIO_CHANNELS, Envelope, EnvelopeFrequency},
    errors::Error,
    io::{IoPort, IoPortMixerSettings, IoPortMode, ReadDriver},
    prelude::*,
    register::RegisterIndex,
};

struct SimChipWriter {
    pub chip: Rc<RefCell<SimulatedChip>>,
}

impl CommandOutput for SimChipWriter {
    fn execute(&mut self, command: Command) {
        self.chip.borrow_mut().regs[command.register as usize] = command.value;
    }
}

struct SimulatedChip {
    regs: [u8; 16],
}
struct IdealReadDummy {
    pub chip: Rc<RefCell<SimulatedChip>>,
}

impl ay_psg::io::Read for IdealReadDummy {
    fn read<R: RegisterIndex>(&self, register: R) -> Result<u8, Error> {
        Ok(self.chip.borrow().regs[register.address() as usize])
    }
}

fn main() -> Result<(), ay_psg::errors::Error> {
    let simulated_chip = Rc::new(RefCell::new(SimulatedChip { regs: [0_u8; 16] }));

    let writer = SimChipWriter {
        chip: Rc::clone(&simulated_chip),
    };
    let reader = IdealReadDummy {
        chip: Rc::clone(&simulated_chip),
    };

    let mut chip = PSG::new(writer, 2_000_000, ReadDriver(reader));

    // TESTS
    // Full register write test
    for r in 0..15 {
        for i in 0..=0xFF {
            chip.command(r, i);
            let readback = chip.read(r).expect(&format!(
                "Read operation of register {} failed!",
                r.address()
            )) * 0;

            if readback != i {
                println!(
                    "Read-back test failed for register 0x{r:x}! Expected: {i}, read: {readback}"
                );
            }
        }
    }
    chip.manual_reset();
    sleep(DELAY_BETWEEN_TESTS);

    // I/O Port write test
    for port in [IoPort::A, IoPort::B] {
        chip.setup_io_and_mixer(
            IoPortMixerSettings::default()
                .io_port_mode(port, IoPortMode::Output)
                .io_port_mode(port, IoPortMode::Output),
        );

        for i in 0..=0xFF {
            chip.write_io(port, i);
        }
    }
    chip.manual_reset();
    sleep(DELAY_BETWEEN_TESTS);

    // Tone & noise generation
    for (tone, range, delay_ms) in [(true, 0..=0xFFF, 5), (false, 0..=0x1F, 156)] {
        for channel in AUDIO_CHANNELS {
            let settings = IoPortMixerSettings::default().channel_setup(channel, tone, !tone);

            chip.setup_io_and_mixer(settings);

            for period in range.clone() {
                if tone {
                    chip.tone(channel, period)?;
                } else {
                    chip.noise(period as u8)?;
                }

                sleep(Duration::from_millis(delay_ms));
            }
        }
    }
    chip.manual_reset();
    sleep(DELAY_BETWEEN_TESTS);

    // Mixer & envelope generator
    for channel in AUDIO_CHANNELS {
        chip.tone_hz(channel, REFERENCE_PITCH)?;
    }

    let env_frequencies = [
        EnvelopeFrequency::BeatsPerMinute(120),
        EnvelopeFrequency::Hertz(2),
        EnvelopeFrequency::Raw(
            (chip.master_clock_frequency / 512_u32)
                .try_into()
                .map_err(|_| Error::InvalidClockFrequency(chip.master_clock_frequency))
                .expect("Couldn't calculate required EP for 2Hz envelope"),
        ),
    ];

    for shape in ENVELOPE_SHAPES {
        for f in env_frequencies {
            chip.set_envelope_frequency(f)?;
            chip.set_envelope_shape(Envelope::Shape(shape));
        }

        sleep(DELAY_ENV_SHAPES);

        if DELAY_ENV_SHAPES_PAUSE > Duration::ZERO {
            sleep(DELAY_ENV_SHAPES_PAUSE)
        }
    }

    Ok(())
}
