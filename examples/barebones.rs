// barebones.rs

use ay_psg::prelude::*;

struct NoOutput;
impl CommandOutput for NoOutput {
    fn execute(&mut self, _: Command) {} // do nothing
}

fn main() {
    let out = NoOutput {};
    PSG::new(out, 2_000_000);
}
