use core::u32;

use fixed::types::{I1F15, U4F4};
use rytmos_engrave::staff::Note;

use super::Effect;

pub struct LowPassFilter {
    pub settings: LowPassFilterSettings,
    prev_output: I1F15,
}

pub struct LowPassFilterSettings {
    pub alpha: I1F15,
}

/// SLOW
/// Computes the alpha parameter needed in a low pass filter.
pub fn compute_alpha(cutoff: f32, sample_rate: usize) -> I1F15 {
    let alpha = cutoff / (cutoff + sample_rate as f32);
    let fixed_alpha = I1F15::from_num(alpha);
    log::trace!("compute_alpha(cutoff={cutoff}, sample_rate={sample_rate}) -> alpha={alpha} fixed={fixed_alpha})");
    fixed_alpha
}

impl LowPassFilter {
    pub fn new(settings: LowPassFilterSettings) -> Self {
        LowPassFilter {
            settings,
            prev_output: I1F15::from_num(0.0),
        }
    }
}

impl Effect for LowPassFilter {
    type Settings = LowPassFilterSettings;

    fn make(_address: u32, settings: Self::Settings) -> Self {
        Self {
            settings,
            prev_output: I1F15::from_num(0.0),
        }
    }

    fn configure(&mut self, settings: Self::Settings) {
        self.settings = settings;
    }

    fn next(&mut self, input: I1F15) -> I1F15 {
        self.prev_output = self.settings.alpha * input
            + (I1F15::MAX - self.settings.alpha + I1F15::from_bits(1)) * self.prev_output;
        self.prev_output.to_num()
    }

    fn play(&mut self, _note: Note, _velocity: U4F4) {}

    fn run_command(&mut self, _command: crate::commands::Command) {}

    fn address(&self) -> u32 {
        u32::MAX
    }
}
