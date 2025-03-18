use fixed::types::{I1F15, U4F4};
use rytmos_engrave::staff::{Accidental, Note};

use crate::commands::Command;

use super::{
    sample::{SampleSynth, SampleSynthSettings},
    samples::{self},
    Synth,
};

pub struct DrumSynth {
    address: u32,
    kick_synth: SampleSynth<'static>,
    snare_synth: SampleSynth<'static>,
    hihat_synth: SampleSynth<'static>,
}

pub const KICK_NOTE: Note = Note::C(Accidental::Natural, 2);
pub const SNARE_NOTE: Note = Note::D(Accidental::Natural, 2);
pub const HIHAT_NOTE: Note = Note::F(Accidental::Sharp, 2);
// TODO: add these to the drum synth
pub const STICKS_NOTE: Note = Note::C(Accidental::Sharp, 2); // TODO: verify
pub const CYMBAL_NOTE: Note = Note::C(Accidental::Sharp, 3);

pub struct DrumSynthSettings {}

impl Synth for DrumSynth {
    type Settings = DrumSynthSettings;

    fn make(address: u32, _settings: Self::Settings) -> Self
    where
        Self: Sized,
    {
        Self {
            address,
            kick_synth: SampleSynth::make(
                address,
                SampleSynthSettings {
                    sample: samples::kick::KICK,
                },
            ),
            snare_synth: SampleSynth::make(
                address,
                SampleSynthSettings {
                    sample: samples::snare::SNARE,
                },
            ),
            hihat_synth: SampleSynth::make(
                address,
                SampleSynthSettings {
                    sample: samples::hihat::HIHAT,
                },
            ),
        }
    }

    fn configure(&mut self, _settings: Self::Settings) {}

    fn play(&mut self, note: Note, velocity: U4F4) {
        match note.to_midi_code() {
            36 => self.kick_synth.play(note, velocity),  // C2
            38 => self.snare_synth.play(note, velocity), // D2
            42 => self.hihat_synth.play(note, velocity), // F#2
            code => {
                unimplemented!("Unimplemented note {code} requested of drum synth");
            }
        }
    }

    fn next(&mut self) -> I1F15 {
        self.kick_synth.next() + self.hihat_synth.next() + self.snare_synth.next()
    }

    fn run_command(&mut self, command: Command) {
        super::run_play_command(self, command);
    }

    fn address(&self) -> u32 {
        self.address
    }
}
