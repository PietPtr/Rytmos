use fixed::types::{I1F15, U4F4};
use log::warn;
use rytmos_engrave::staff::{Accidental, Note};

use crate::commands::Command;

use super::{
    sample::{SampleSynth, SampleSynthSettings},
    samples::{self},
    Synth,
};

pub struct DrumSynth {
    address: u32,
    kick_synth: SampleSynth<samples::Kick>,
    snare_synth: SampleSynth<samples::Snare>,
    hihat_synth: SampleSynth<samples::Hihat>,
    strong_synth: SampleSynth<samples::Strong>,
    weak_synth: SampleSynth<samples::Weak>,
    cymbal_synth: SampleSynth<samples::Cymbal>,
}

pub const KICK_NOTE: Note = Note::C(Accidental::Natural, 2);
pub const SNARE_NOTE: Note = Note::D(Accidental::Natural, 2);
pub const HIHAT_NOTE: Note = Note::F(Accidental::Sharp, 2);
pub const WEAK_NOTE: Note = Note::G(Accidental::Sharp, 1);
pub const STRONG_NOTE: Note = Note::A(Accidental::Sharp, 1);
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
            kick_synth: SampleSynth::make(address, SampleSynthSettings {}),
            snare_synth: SampleSynth::make(address, SampleSynthSettings {}),
            hihat_synth: SampleSynth::make(address, SampleSynthSettings {}),
            strong_synth: SampleSynth::make(address, SampleSynthSettings {}),
            weak_synth: SampleSynth::make(address, SampleSynthSettings {}),
            cymbal_synth: SampleSynth::make(address, SampleSynthSettings {}),
        }
    }

    fn configure(&mut self, _settings: Self::Settings) {}

    fn play(&mut self, note: Note, velocity: U4F4) {
        match note {
            KICK_NOTE => self.kick_synth.play(note, velocity),
            SNARE_NOTE => self.snare_synth.play(note, velocity),
            HIHAT_NOTE => self.hihat_synth.play(note, velocity),
            STRONG_NOTE => self.strong_synth.play(note, velocity),
            WEAK_NOTE => self.weak_synth.play(note, velocity),
            CYMBAL_NOTE => self.cymbal_synth.play(note, velocity),
            note => {
                warn!(
                    "Unimplemented note {} requested of drum synth",
                    note.to_midi_code()
                );
                self.kick_synth.play(note, velocity); // fallback
            }
        }
    }

    fn next(&mut self) -> I1F15 {
        self.kick_synth
            .next()
            .saturating_add(self.hihat_synth.next())
            .saturating_add(self.snare_synth.next())
            .saturating_add(self.strong_synth.next())
            .saturating_add(self.weak_synth.next())
            .saturating_add(self.cymbal_synth.next())
    }

    fn run_command(&mut self, command: Command) {
        super::run_play_command(self, command);
    }

    fn address(&self) -> u32 {
        self.address
    }

    fn freq(&mut self, freq: fixed::types::U12F4) {}

    fn attack(&mut self, attack: U4F4) {
        self.hihat_synth.attack(attack);
        self.snare_synth.attack(attack);
        self.kick_synth.attack(attack);
        self.strong_synth.attack(attack);
        self.weak_synth.attack(attack);
        self.cymbal_synth.attack(attack);
    }
}
