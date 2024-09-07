use super::SAMPLE_RATE;

pub struct LowPassFilter {
    cutoff: f32,
    prev_output: f32,
}

impl LowPassFilter {
    pub fn new(cutoff: f32) -> Self {
        LowPassFilter {
            cutoff,
            prev_output: 0.0,
        }
    }

    pub fn alpha(&self) -> f32 {
        self.cutoff / (self.cutoff + SAMPLE_RATE)
    }

    pub fn next(&mut self, sample: i16) -> i16 {
        let sample_f32 = sample as f32;
        self.prev_output = self.alpha() * sample_f32 + (1.0 - self.alpha()) * self.prev_output;
        self.prev_output as i16
    }
}
