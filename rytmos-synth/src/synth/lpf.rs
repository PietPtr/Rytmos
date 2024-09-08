use super::SAMPLE_RATE;

pub struct LowPassFilter {
    alpha: f32,
    prev_output: f32,
}

impl LowPassFilter {
    pub fn new(cutoff: f32) -> Self {
        LowPassFilter {
            alpha: cutoff / (cutoff + SAMPLE_RATE),
            prev_output: 0.0,
        }
    }

    pub fn set_cutoff(&mut self, cutoff: f32) {
        self.alpha = cutoff / (cutoff / SAMPLE_RATE);
    }

    pub fn next(&mut self, sample: i16) -> i16 {
        self.prev_output = self.alpha * (sample as f32) + (1.0 - self.alpha) * self.prev_output;
        self.prev_output as i16
    }
}
