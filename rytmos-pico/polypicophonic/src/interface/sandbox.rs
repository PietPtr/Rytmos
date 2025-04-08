use common::debouncer::Debouncer;
use embedded_hal::digital::v2::InputPin;
use fixed::types::U4F4;
use heapless::Vec;
use rytmos_engrave::{a, ais, b, c, cis, d, dis, e, f, fis, g, gis};
use rytmos_synth::commands::{Command, CommandMessage};

use crate::{
    chords::{self, ChordConstruction, ChordQuality},
    clavier::KeyId,
};

use super::{Interface, PicoPianoHardware};

pub struct SandboxInterface {
    hw: PicoPianoHardware,
}

impl SandboxInterface {
    pub fn new(hw: PicoPianoHardware) -> Self {
        Self { hw }
    }
}

impl Interface for SandboxInterface {
    fn start(mut self) -> ! {
        let mut octave = 4;
        let attack = U4F4::ONE;

        let mut button_states = [false; 12];

        loop {
            self.hw.clavier.update_debouncers();

            let events = self.hw.clavier.note_events();

            let mut messages = events
                .iter()
                .filter_map(|event| {
                    Some(CommandMessage::Play(
                        event.note(octave)?,
                        event.velocity(attack),
                    ))
                })
                .collect::<Vec<_, 4>>();

            // ---- chords

            const CONSTRUCTION: ChordConstruction = ChordConstruction::InvertToWithinOctave;

            if let Some(true) = self.hw.clavier.debouncer_is_high(KeyId::Fn1) {
                chords::add_chord(&mut messages, ChordQuality::Major, CONSTRUCTION);
                chords::root_to_bass_register(&mut messages);
            }

            if let Some(true) = self.hw.clavier.debouncer_is_high(KeyId::Fn3) {
                chords::add_chord(&mut messages, ChordQuality::Minor, CONSTRUCTION);
                chords::root_to_bass_register(&mut messages);
            }

            // ----

            for message in messages {
                let command = Command {
                    address: 0x0,
                    message,
                };
                let command_serialized = command.serialize();

                self.hw.fifo.write(command_serialized);
            }

            if let Some(true) = self.hw.clavier.debouncer_is_high(KeyId::Fn0) {
                octave = 5
            } else if let Some(true) = self.hw.clavier.debouncer_is_high(KeyId::Fn2) {
                octave = 3
            } else {
                octave = 4
            }
        }
    }
}
