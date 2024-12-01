use fixed::{
    traits::ToFixed,
    types::{extra::U15, I1F15, U4F4},
    FixedI32,
};

use crate::{commands::Command, wavetables::SINE_WAVE};

use super::{run_play_command, Synth};

pub struct SineSynth {
    address: u32,
    settings: SineSynthSettings,
    phase: I1F15, // -1 => -PI, 1 => PI
    phase_inc: I1F15,
    // Value added to the phase inc. pub so other synths can influence frequency efficiently.
    pub bend: I1F15,
    gain: u8,
    velocity: U4F4,
    amplitude: I1F15,
    decay_counter: usize,
}

impl SineSynth {
    pub fn new(address: u32, settings: SineSynthSettings) -> Self {
        Self {
            address,
            settings,
            phase: settings.initial_phase,
            gain: 0,
            bend: I1F15::from_bits(0),
            phase_inc: I1F15::from_bits(0),
            amplitude: I1F15::from_bits(0),
            velocity: U4F4::from_num(1),
            decay_counter: 0,
        }
    }

    fn lerp(a: I1F15, b: I1F15, t: I1F15) -> I1F15 {
        (I1F15::MAX - t) * a + t * b
    }
}

const DECAY_EVERY: usize = 32;

#[derive(Clone, Copy, Debug, Default)]
pub struct SineSynthSettings {
    /// Before velocity of the note is applied, apply this gain to any note played.
    pub extra_attack_gain: U4F4,
    /// Initial phase of the sine wave
    pub initial_phase: I1F15,
    /// Decay subtracted from amplitude every `DECAY_EVERY` next() calls.
    pub decay: I1F15,
}

impl Synth for SineSynth {
    type Settings = SineSynthSettings;

    fn configure(&mut self, settings: Self::Settings) {
        self.settings = settings
    }

    fn play(&mut self, note: rytmos_engrave::staff::Note, velocity: U4F4) {
        self.velocity = velocity;

        self.phase_inc = note.lookup_increment_24000().unwrap_or_else(|| {
            log::error!("Failed to lookup increment");
            I1F15::from_num(0)
        });

        self.amplitude = I1F15::MAX;
    }

    fn next(&mut self) -> I1F15 {
        let table_size = SINE_WAVE.len();

        const OH_POINT_FIVE: I1F15 = I1F15::lit("0.5");

        let (sign, flip_index, modulo) = match self.phase {
            p if p >= I1F15::MIN && p < -0.5 => (-1, false, p + I1F15::MAX + I1F15::from_bits(1)), // +1
            p if p >= -0.5 && p < 0.0 => (-1, true, p + OH_POINT_FIVE),
            p if p >= 0.0 && p < 0.5 => (1, false, p),
            p if p >= 0.5 && p <= I1F15::MAX => (1, true, p - OH_POINT_FIVE),
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

        (self.phase, _) = self.phase.overflowing_add(self.phase_inc * 2 + self.bend);

        let out_sample = FixedI32::<U15>::from(sample)
            .saturating_mul(FixedI32::<U15>::from(self.settings.extra_attack_gain))
            .saturating_mul(FixedI32::<U15>::from(self.velocity))
            .saturating_to_fixed::<I1F15>();

        // TODO: this decay is generic enough that it can be its own processor.
        // define generic signal processors with 1 inp and 1 outp, and a way to connect them.
        // The low pass filter should also be such a processor
        self.decay_counter += 1;
        if self.decay_counter == DECAY_EVERY {
            self.amplitude = (self.amplitude - self.settings.decay).max(I1F15::from_bits(0));
            self.decay_counter = 0;
        }

        out_sample * self.amplitude
    }

    fn run_command(&mut self, command: Command) {
        run_play_command(self, command);
    }

    fn address(&self) -> u32 {
        self.address
    }
}
