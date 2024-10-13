// Generic synth that can be reconfigured at runtime using commands.

use super::{metronome::MetronomeSynth, overtone::OvertoneSynth, vibrato::VibratoSynth};

/// Contains a simple overtone synth defined at construction and a metronome.
/// Provides functions to change metronome settings.
pub struct OvertoneAndMetronomeSynthManager {
    synth: OvertoneSynth<VibratoSynth, 4>,
    metronome: MetronomeSynth,
}
