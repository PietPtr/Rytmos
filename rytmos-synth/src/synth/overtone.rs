use rytmos_engrave::staff::Note;

use super::Synth;

/// Synthesizer that overlays a single synth N times in its integer overtone series.
pub struct OvertoneSynth<S: Synth, const N: usize> {
    settings: OvertoneSynthSettings<N>,
    synths: [S; N],
}

pub struct OvertoneSynthSettings<const N: usize> {}

impl<S: Synth, const N: usize> OvertoneSynth<S, N> {
    pub fn new(settings: OvertoneSynthSettings<N>, synths: [S; N]) -> Self {
        Self { settings, synths }
    }
}

impl<S: Synth, const N: usize> Synth for OvertoneSynth<S, N> {
    type Settings = OvertoneSynthSettings<N>;

    fn configure(&mut self, settings: Self::Settings) {
        self.settings = settings
    }

    fn play(&mut self, mut note: Note, velocity: f32) {
        for (i, synth) in self.synths.iter_mut().enumerate() {
            synth.play(note.map_octave(|n| n + i as i32), velocity)
        }
    }

    fn next(&mut self) -> i16 {
        self.synths
            .iter_mut()
            .fold(0, |acc, synth| acc.saturating_add(synth.next()))
    }
}
