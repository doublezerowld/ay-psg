// barebones.rs

use ay_psg::prelude::*;

struct NoOutput;
impl CommandOutput for NoOutput {
    fn execute(&mut self, _: Command) {} // do nothing
}

struct NoRead;
impl Read for NoRead {
    fn read<R>(&self, _: R) -> Result<u8, ay_psg::errors::Error> {
        Ok(0)
    }
}

fn main() {
    let out = NoOutput {};

    #[cfg(not(feature = "read"))]
    PSG::new(out, 2_000_000);
    #[cfg(feature = "read")]
    PSG::new(out, 2_000_000, ReadDriver(NoRead));
}
