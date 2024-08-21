use crate::wavetables::SINE_WAVE;

pub const SAMPLE_RATE: f32 = 44100.0;

pub struct SineSynth {
    phase: f32,
    phase_inc: f32,
    gain: f32,
    decay: f32,
}

// TODO: an actual pico crate that sets up I2S and tests this synth on embedded hardware

impl SineSynth {
    pub fn new(frequency: f32, gain: f32, initial_phase: f32, decay_per_second: f32) -> Self {
        let phase_inc = frequency / SAMPLE_RATE;
        let decay = decay_per_second.powf(1. / SAMPLE_RATE).min(1.0);
        Self {
            phase: initial_phase,
            phase_inc,
            gain,
            decay,
        }
    }

    // TODO: move these to a float lib
    fn lerp(a: i16, b: i16, t: f32) -> i16 {
        ((1.0 - t) * (a as f32) + t * (b as f32)) as i16
    }

    fn fract(x: f32) -> f32 {
        x - (x as i64 as f32)
    }

    fn round(x: f32) -> f32 {
        let floored = (x as u64) as f32;
        let decimal_part = x - floored;

        if decimal_part < 0.5 {
            floored
        } else {
            floored + 1.0
        }
    }
}

impl Iterator for SineSynth {
    type Item = i16;

    fn next(&mut self) -> Option<Self::Item> {
        let table_size = SINE_WAVE.len() as f32;

        let (sign, flip_index) = match self.phase {
            p if p >= 0.00 && p < 0.25 => (1, false),
            p if p >= 0.25 && p < 0.50 => (1, true),
            p if p >= 0.50 && p < 0.75 => (-1, false),
            p if p >= 0.75 && p < 1.00 => (-1, true),
            _ => panic!("Impossible phase"),
        };

        let idx_in_part = (4. * (Self::fract(self.phase) % 0.25)) * (table_size - 1.0);
        let idx_float = if flip_index {
            table_size - 1.0 - idx_in_part
        } else {
            idx_in_part
        };

        let idx = Self::round(idx_float) as usize;
        let next_idx = match (idx, flip_index) {
            (0, _) => 1,
            (idx, _) if idx == SINE_WAVE.len() - 1 => idx - 1,
            (idx, _) => idx + 1,
        };

        let a = SINE_WAVE[idx];
        let b = SINE_WAVE[next_idx];
        let t = idx_float - idx as f32;

        let sample = (Self::lerp(a, b, t) as f32 * self.gain) as i16 * sign;

        self.phase = (self.phase + self.phase_inc) % 1.0;

        self.gain *= self.decay;

        Some(sample)
    }
}
