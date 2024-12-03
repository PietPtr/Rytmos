use fixed::{
    traits::ToFixed,
    types::{extra::U15, I1F15, U4F4, U8F8},
    FixedI32,
};

use crate::commands::Command;

use super::{run_play_command, Synth};

pub struct SawtoothSynth {
    address: u32,
    settings: SawtoothSynthSettings,
    increment: I1F15, // Computed from frequency
    sample: I1F15, // store the sample way more precise than needed so we can do more exact increments
    velocity: U8F8,
    sample_counter: u32,
}

#[derive(Clone, Copy, Debug)]
pub struct SawtoothSynthSettings {}

impl Synth for SawtoothSynth {
    type Settings = ();

    fn make(address: u32, settings: Self::Settings) -> Self {
        Self {
            address,
            settings: SawtoothSynthSettings {},
            increment: I1F15::from_num(0),
            sample: I1F15::from_num(0),
            velocity: U8F8::from_num(0),
            sample_counter: 0,
        }
    }

    fn configure(&mut self, (): Self::Settings) {}

    fn play(&mut self, note: rytmos_engrave::staff::Note, velocity: U4F4) {
        self.velocity = velocity.into();
        self.increment = note.lookup_increment_24000().unwrap_or_else(|| {
            log::error!("Failed to lookup increment");
            I1F15::from_num(0)
        }) << 1;
    }

    fn next(&mut self) -> I1F15 {
        self.sample_counter = self.sample_counter.overflowing_add(1).0;
        let (next_sample, _) = self.sample.overflowing_add(self.increment);

        self.sample = next_sample;

        // Apply gain
        let out_sample = FixedI32::<U15>::from(next_sample)
            .saturating_mul(FixedI32::<U15>::from(self.velocity))
            .saturating_to_fixed::<I1F15>();

        out_sample
    }

    fn run_command(&mut self, command: Command) {
        run_play_command(self, command);
    }

    fn address(&self) -> u32 {
        self.address
    }
}
