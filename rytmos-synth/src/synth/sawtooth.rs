use fixed::{
    traits::ToFixed,
    types::{extra::U15, I1F15, U8F8},
    FixedI32, FixedI64,
};

use crate::commands::Command;

use super::{run_play_command, Synth, SAMPLE_RATE};

pub struct SawtoothSynth {
    settings: SawtoothSynthSettings,
    increment: I1F15, // Computed from frequency
    sample: I1F15, // store the sample way more precise than needed so we can do more exact increments
    velocity: U8F8,
    sample_counter: u32,
}

impl SawtoothSynth {
    pub fn new(settings: SawtoothSynthSettings) -> Self {
        log::info!("Sawtooth Settings: {settings:?}");
        Self {
            settings,
            increment: I1F15::from_num(0),
            sample: I1F15::from_num(0),
            velocity: U8F8::from_num(0),
            sample_counter: 0,
        }
    }

    pub fn set_frequency(&mut self, frequency_in_hertz: f32) {
        self.increment = I1F15::from_num(frequency_in_hertz / SAMPLE_RATE);
    }
}

#[derive(Clone, Copy, Debug)]
pub struct SawtoothSynthSettings {
    pub decay: U8F8,
}

impl Synth for SawtoothSynth {
    type Settings = SawtoothSynthSettings;

    fn configure(&mut self, settings: Self::Settings) {
        self.settings = settings
    }

    // TODO: convert this float in the trait to some sort of fixed point value
    fn play(&mut self, note: rytmos_engrave::staff::Note, velocity: U8F8) {
        self.set_frequency(note.frequency());
        self.velocity = velocity;
    }

    fn next(&mut self) -> I1F15 {
        self.sample_counter = self.sample_counter.overflowing_add(1).0;
        let (next_sample, _) = self.sample.overflowing_add(self.increment);

        self.sample = next_sample;

        // Apply gain
        let out_sample = FixedI32::<U15>::from(next_sample)
            .saturating_mul(FixedI32::<U15>::from(self.velocity))
            .saturating_to_fixed::<I1F15>();

        if self.sample_counter == 1000 {
            self.sample_counter = 0;
            self.velocity *= self.settings.decay;
        }

        out_sample
    }

    fn run_command(&mut self, command: Command) {
        run_play_command(self, command);

        #[allow(clippy::single_match)]
        match command {
            Command::SetAttack(attack) => {
                // self.settings.attack_gain = (attack as u32 * 256) as f32 * scale as f32
            }
            _ => (),
        }
    }
}
