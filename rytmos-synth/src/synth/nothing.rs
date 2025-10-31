use fixed::types::{I1F15, U4F4};
use rytmos_engrave::staff::Note;

use crate::commands::Command;

const ZERO: I1F15 = I1F15::from_bits(0);

use super::Synth;

pub struct NothingSynth {}

/// Synth that outputs a constant zero. Used for benchmarking.
impl Synth for NothingSynth {
    type Settings = ();

    fn make(_address: u32, _settingss: Self::Settings) -> Self {
        Self {}
    }

    fn configure(&mut self, (): Self::Settings) {}

    fn play(&mut self, _note: Note, _velocity: U4F4) {}

    fn next(&mut self) -> I1F15 {
        ZERO
    }

    fn run_command(&mut self, _command: Command) {}

    fn address(&self) -> u32 {
        0
    }

    fn freq(&mut self, _freq: fixed::types::U12F4) {}

    fn attack(&mut self, _attack: U4F4) {}
}
