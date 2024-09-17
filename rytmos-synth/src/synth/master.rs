// Generic synth that can be reconfigured at runtime using commands.

use super::{overtone::OvertoneSynth, vibrato::VibratoSynth};

pub struct MasterSynth {
    synth: OvertoneSynth<VibratoSynth, 4>,
}
