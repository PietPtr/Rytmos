use fixed::types::{I1F15, U4F4, U8F8};
use log::*;

use crate::{commands::Command, wavetables::SINE_WAVE};

use super::{run_play_command, Synth, SAMPLE_RATE};

pub struct SineSynth {
    address: u32,
    settings: SineSynthSettings,
    frequency: f32,
    phase: I1F15, // -1 => -PI, 1 => PI
    phase_inc: I1F15,
    gain: u8,
}

impl SineSynth {
    pub fn new(address: u32, settings: SineSynthSettings) -> Self {
        Self {
            address,
            settings,
            frequency: 0.,
            phase: settings.initial_phase,
            gain: 0,
            phase_inc: I1F15::from_bits(0),
        }
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

    fn lerp(a: I1F15, b: I1F15, t: I1F15) -> I1F15 {
        (I1F15::MAX - t) * a + t * b
    }
}

#[derive(Clone, Copy, Debug)]
pub struct SineSynthSettings {
    pub attack_gain: U4F4,
    pub initial_phase: I1F15,
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

        self.phase_inc = note.lookup_increment_24000().unwrap_or_else(|| {
            log::error!("Failed to lookup increment");
            I1F15::from_num(0)
        });
    }

    fn next(&mut self) -> I1F15 {
        let table_size = SINE_WAVE.len();

        const OH_POINT_FIVE: I1F15 = I1F15::lit("0.5");

        let (sign, flip_index, modulo) = match self.phase {
            p if p >= I1F15::MIN && p < -0.5 => (1, false, p + I1F15::MAX + I1F15::from_bits(1)), // +1
            p if p >= -0.5 && p < 0.0 => (1, true, p + OH_POINT_FIVE),
            p if p >= 0.0 && p < 0.5 => (-1, false, p),
            p if p >= 0.5 && p <= I1F15::MAX => (-1, true, p - OH_POINT_FIVE),
            p => panic!("Impossible phase: {}", p),
        };

        // Scale the table size by the phase: phase * table_size.
        // Table size is 64, so a multiplication like that constitutes a 6 bit left shift.
        // We can do the multplication with the fixed point phase by converting it to a u32
        // and do a regular multiplication and then shifting 15 bits to the right.
        let scaled_lut_size = ((2 * modulo).to_bits() as usize) << 6;
        let idx_in_part = scaled_lut_size >> 15;
        let fractional_part = I1F15::from_bits((scaled_lut_size & 0x7fff) as i16);

        let idx = if flip_index {
            table_size - 1 - idx_in_part
        } else {
            idx_in_part as usize
        };

        let next_idx = match (idx, flip_index) {
            (0, _) => 1,
            (idx, _) if idx == SINE_WAVE.len() - 1 => idx - 1,
            (idx, _) => idx + 1,
        };

        let a = SINE_WAVE[idx];
        let b = SINE_WAVE[next_idx];
        let t = if flip_index {
            I1F15::MAX - fractional_part
        } else {
            fractional_part
        };

        let sample = (Self::lerp(a, b, t) << self.gain) * sign;
        // let sample = a * sign;

        (self.phase, _) = self.phase.overflowing_add(self.phase_inc);

        sample
    }

    fn run_command(&mut self, command: Command) {
        run_play_command(self, command);
    }

    fn address(&self) -> u32 {
        self.address
    }
}
