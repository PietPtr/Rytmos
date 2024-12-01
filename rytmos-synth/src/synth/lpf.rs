use fixed::types::I1F15;

pub struct LowPassFilter {
    pub alpha: I1F15,
    prev_output: I1F15,
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
    pub fn new(alpha: I1F15) -> Self {
        LowPassFilter {
            alpha,
            prev_output: I1F15::from_num(0.0),
        }
    }

    pub fn next(&mut self, sample: I1F15) -> I1F15 {
        self.prev_output = self.alpha * sample
            + (I1F15::MAX - self.alpha + I1F15::from_bits(1)) * self.prev_output;
        self.prev_output.to_num()
    }
}
