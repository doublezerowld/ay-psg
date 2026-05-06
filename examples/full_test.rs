//! # Test code that verifies the proper function of the chip / library.
//!
//! You will probably want to run these tests on actual hardware instead of just writing the output to the stdout.
//! Take a look in `examples/embedded/...` for embedded implementations.
//!
//! The tests include (in chronological order):
//! - Full register write test (read-back test if the "read" feature is enabled)
//! - Register 7 test (I/O & mixer settings)
//! - I/O Port write test (read-back test if the "read" feature is enabled)
//! - Tone generation (full sweep)
//! - Noise generation (full sweep)
//! - Envelope generation (using `Hz`, `BPM`, `Raw` enum variants)
//! - Mixer (A, B, C level registers)
//! - Notes (if the "notes" feature is enabled)
//! - Argument validation test (errors.rs)

fn main() {}
