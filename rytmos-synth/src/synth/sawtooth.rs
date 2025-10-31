use core::u32;

use fixed::{
    traits::{LossyInto, ToFixed},
    types::{extra::U15, I1F15, U12F20, U15F17, U24F8, U44F20, U4F4, U8F8},
    FixedI32,
};

use crate::commands::{Command, CommandMessage};

use super::{run_play_command, Synth};

pub struct SawtoothSynth {
    address: u32,
    increment: I1F15, // Computed from frequency
    sample: I1F15, // store the sample way more precise than needed so we can do more exact increments
    velocity: U8F8,
    sample_counter: u32,
}

// TODO: hard coded at 24000Hz sample rate, that should somehow be nicer

#[derive(Clone, Copy, Debug, Default)]
pub struct SawtoothSynthSettings {}

impl Synth for SawtoothSynth {
    type Settings = SawtoothSynthSettings;

    fn make(address: u32, _: Self::Settings) -> Self {
        Self {
            address,
            increment: I1F15::from_num(0),
            sample: I1F15::from_num(0),
            velocity: U8F8::from_num(0),
            sample_counter: 0,
        }
    }

    fn configure(&mut self, _: Self::Settings) {}

    fn play(&mut self, note: rytmos_engrave::staff::Note, velocity: U4F4) {
        self.velocity = velocity.into();
        self.increment = note.lookup_increment_24000().unwrap_or_else(|| {
            log::error!("Failed to lookup increment");
            I1F15::from_num(0)
        }) << 1;
    }

    fn freq(&mut self, freq: fixed::types::U12F4) {
        // TODO: determine performance somehow, this is probably slow
        let freq: U15F17 = freq.into();
        let per_sample: U15F17 = freq.wrapping_div(U15F17::from_num(24000));
        let per_sample_fracs_cut = per_sample.to_bits() >> 2;
        self.increment = I1F15::from_bits(per_sample_fracs_cut as i16);
        // panic!(
        //     "{:?} {:?} {:?}",
        //     per_sample, per_sample_fracs_cut, self.increment
        // );
    }

    fn attack(&mut self, attack: U4F4) {
        self.velocity = attack.into()
    }

    fn next(&mut self) -> I1F15 {
        self.sample_counter = self.sample_counter.overflowing_add(1).0;
        let (next_sample, _) = self.sample.overflowing_add(self.increment);

        self.sample = next_sample;

        // Apply gain
        FixedI32::<U15>::from(next_sample)
            .saturating_mul(FixedI32::<U15>::from(self.velocity))
            .saturating_to_fixed::<I1F15>()
    }

    fn run_command(&mut self, command: Command) {
        run_play_command(self, command);

        match command.message {
            CommandMessage::Frequency(freq, velocity) => {
                self.freq(freq);
                self.attack(velocity);
            }
            _ => {}
        }
    }

    fn address(&self) -> u32 {
        self.address
    }
}
