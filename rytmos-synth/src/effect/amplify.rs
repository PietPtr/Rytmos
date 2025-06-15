use derivative::Derivative;
use fixed::types::{I17F15, I1F15, U4F4, U8F8};

use crate::effect::Effect;

/// Simple linear amplification of the input that clips on overflow.
pub struct Amplify {
    pub settings: AmplifySettings,
}

#[derive(Derivative)]
#[derivative(Default)]
#[derive(Debug, Clone)]
pub struct AmplifySettings {
    #[derivative(Default(value = "U8F8::ONE"))]
    pub amplification: U8F8,
}

impl Effect for Amplify {
    type Settings = AmplifySettings;

    fn make(_address: u32, settings: Self::Settings) -> Self {
        Self { settings }
    }

    fn configure(&mut self, settings: Self::Settings) {
        self.settings = settings
    }

    fn next(&mut self, input: I1F15) -> I1F15 {
        let big_input: I17F15 = input.into();
        let big_amp: I17F15 = self.settings.amplification.into();

        let amplified = big_input.saturating_mul(big_amp);

        if amplified > I17F15::ONE {
            I1F15::MAX
        } else if amplified < I17F15::NEG_ONE {
            I1F15::NEG_ONE
        } else {
            I1F15::from_bits(amplified.to_bits() as i16)
        }
    }

    fn play(&mut self, _note: rytmos_engrave::staff::Note, _velocity: U4F4) {}

    fn run_command(&mut self, _command: crate::commands::Command) {}

    fn address(&self) -> u32 {
        u32::MAX
    }
}
