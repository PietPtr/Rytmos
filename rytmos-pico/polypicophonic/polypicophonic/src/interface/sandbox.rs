use fixed::types::U4F4;
use heapless::Vec;

use rytmos_synth::commands::{Command, CommandMessage};

use crate::{
    chords::{self, ChordConstruction, ChordQuality},
    clavier::{Clavier, KeyId},
    io::{self, IO},
};

use super::Interface;

pub struct SandboxInterface<FIFO, CLAVIER> {
    fifo: FIFO,
    clavier: Clavier<CLAVIER>,
}

impl<FIFO: io::Fifo, CLAVIER: io::ClavierPins> SandboxInterface<FIFO, CLAVIER> {
    pub fn new(io: IO<FIFO, CLAVIER>) -> Self {
        Self {
            fifo: io.fifo,
            clavier: Clavier::new(io.clavier),
        }
    }
}

impl<FIFO: io::Fifo, CLAVIER: io::ClavierPins> Interface for SandboxInterface<FIFO, CLAVIER> {
    fn start(mut self) -> ! {
        let mut octave = 4;
        let attack = U4F4::ONE;

        loop {
            self.clavier.update_debouncers();

            let events = self.clavier.note_events();

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

            if self.clavier.debouncer_is_high(KeyId::Fn1) {
                chords::add_chord(&mut messages, ChordQuality::Major, CONSTRUCTION);
                chords::root_to_bass_register(&mut messages);
            }

            if self.clavier.debouncer_is_high(KeyId::Fn3) {
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

                self.fifo.write(command_serialized);
            }

            if self.clavier.debouncer_is_high(KeyId::Fn0) {
                octave = 5
            } else if self.clavier.debouncer_is_high(KeyId::Fn2) {
                octave = 3
            } else {
                octave = 4
            }
        }
    }
}
