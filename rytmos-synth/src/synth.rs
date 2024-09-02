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
        let decay = libm::powf(decay_per_second, 1. / SAMPLE_RATE).min(1.0);
        Self {
            phase: initial_phase,
            phase_inc,
            gain,
            decay,
        }
    }

    fn lerp(a: i16, b: i16, t: f32) -> i16 {
        ((1.0 - t) * (a as f32) + t * (b as f32)) as i16
    }
}

impl Iterator for SineSynth {
    type Item = i16;

    fn next(&mut self) -> Option<Self::Item> {
        let table_size = SINE_WAVE.len() as f32;

        let (sign, flip_index) = match self.phase {
            p if (0.00..0.25).contains(&p) => (1, false),
            p if (0.25..0.50).contains(&p) => (1, true),
            p if (0.50..0.75).contains(&p) => (-1, false),
            p if (0.75..1.00).contains(&p) => (-1, true),
            _ => panic!("Impossible phase"),
        };

        let idx_in_part = (4. * (libm::modff(self.phase).0 % 0.25)) * (table_size - 1.0);
        let idx_float = if flip_index {
            table_size - 1.0 - idx_in_part
        } else {
            idx_in_part
        };

        let idx = libm::roundf(idx_float as f32) as usize;
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

// TODO: Add metronome synth, given a tempo generates ticks with emphasis on 1.
