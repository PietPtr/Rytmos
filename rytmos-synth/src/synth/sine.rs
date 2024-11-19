use fixed::types::{I1F15, U4F4, U8F8};
use log::*;

use crate::{commands::Command, wavetables::SINE_WAVE};

use super::{run_play_command, Synth, SAMPLE_RATE};

pub struct SineSynth {
    address: u32,
    settings: SineSynthSettings,
    frequency: f32,
    phase: f32,
    gain: f32,
}

impl SineSynth {
    pub fn new(address: u32, settings: SineSynthSettings) -> Self {
        Self {
            address,
            settings,
            frequency: 0.,
            phase: settings.initial_phase,
            gain: 0.,
        }
    }

    fn phase_inc(&self) -> f32 {
        self.frequency / SAMPLE_RATE
    }

    pub fn set_frequency(&mut self, frequency: f32) {
        if frequency < 0. {
            log::error!("sub zero freq: {frequency}");
            return;
        }
        self.frequency = frequency
    }

    fn decay(&self) -> f32 {
        libm::powf(self.settings.decay_per_second, 1. / SAMPLE_RATE).min(1.0)
    }

    fn lerp(a: i16, b: i16, t: f32) -> f32 {
        (1.0 - t) * (a as f32) + t * (b as f32)
    }
}

#[derive(Clone, Copy, Debug)]
pub struct SineSynthSettings {
    pub attack_gain: U4F4,
    pub initial_phase: f32,
    pub decay_per_second: f32,
}

impl Synth for SineSynth {
    type Settings = SineSynthSettings;

    fn configure(&mut self, settings: Self::Settings) {
        self.settings = settings
    }

    fn play(&mut self, note: rytmos_engrave::staff::Note, velocity: U4F4) {
        self.frequency = note.frequency();
        // self.gain = velocity * self.settings.attack_gain;
    }

    fn next(&mut self) -> I1F15 {
        todo!()
        // let table_size = SINE_WAVE.len() as f32;

        // let (sign, flip_index) = match self.phase {
        //     p if (0.00..0.25).contains(&p) => (1, false),
        //     p if (0.25..0.50).contains(&p) => (1, true),
        //     p if (0.50..0.75).contains(&p) => (-1, false),
        //     p if (0.75..1.00).contains(&p) => (-1, true),
        //     p => panic!("Impossible phase: {}", p),
        // };

        // let idx_in_part = (4. * (libm::modff(self.phase).0 % 0.25)) * (table_size - 1.0);
        // let idx_float = if flip_index {
        //     table_size - 1.0 - idx_in_part
        // } else {
        //     idx_in_part
        // };

        // let idx = libm::roundf(idx_float as f32) as usize;
        // let next_idx = match (idx, flip_index) {
        //     (0, _) => 1,
        //     (idx, _) if idx == SINE_WAVE.len() - 1 => idx - 1,
        //     (idx, _) => idx + 1,
        // };

        // let a = SINE_WAVE[idx];
        // let b = SINE_WAVE[next_idx];
        // let t = idx_float - idx as f32;

        // let sample = (Self::lerp(a, b, t) * self.gain) as i16 * sign;

        // self.phase = (self.phase + self.phase_inc()) % 1.0;
        // self.gain *= self.decay();

        // sample
    }

    fn run_command(&mut self, command: Command) {
        run_play_command(self, command);
    }

    fn address(&self) -> u32 {
        self.address
    }
}
