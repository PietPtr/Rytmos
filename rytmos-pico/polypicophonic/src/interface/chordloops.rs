use fixed::types::U4F4;
use heapless::Vec;
use rytmos_engrave::staff::Note;
use rytmos_synth::commands::{Command, CommandMessage};

use crate::{
    chords::{self, ChordConstruction, ChordQuality},
    clavier::KeyId,
};

use super::{Interface, PicoPianoHardware};

const CHORDS_PER_LOOP: usize = 4;
const CONSTRUCTION: ChordConstruction = ChordConstruction::InvertToWithinOctave;

pub struct ChordLoopInterface {
    hw: PicoPianoHardware,
    state: State,
    settings: ChordLoopSettings,
    chords: Vec<Chord, CHORDS_PER_LOOP>,
    loops: f32,
}

struct ChordLoopSettings {
    loops_per_chord: f32, // proxy for tempo
}

type Chord = Vec<Note, CHORDS_PER_LOOP>;

enum State {
    AwaitChords,
    Playing { current_chord: usize },
    Paused,
}

impl ChordLoopInterface {
    pub fn new(hw: PicoPianoHardware) -> Self {
        Self {
            hw,
            chords: Vec::new(),
            state: State::AwaitChords,
            settings: ChordLoopSettings {
                loops_per_chord: 100.,
            },
            loops: 0.0,
        }
    }

    /// While awaiting chords, it is only possible to input chords.
    /// Hold FN1 for major and FN3 for minor, then press the root note's key.
    fn await_chords(&mut self, messages: &mut Vec<CommandMessage, 4>) {
        if let Some(true) = self.hw.clavier.debouncer_is_high(KeyId::Fn1) {
            chords::add_chord(messages, ChordQuality::Major, CONSTRUCTION);
            chords::root_to_bass_register(messages);
        }

        if let Some(true) = self.hw.clavier.debouncer_is_high(KeyId::Fn3) {
            chords::add_chord(messages, ChordQuality::Minor, CONSTRUCTION);
            chords::root_to_bass_register(messages);
        }

        // same code as for chords in the sandbox interface,
        // but each time a chord is played add it to the state
        let result = {
            let notes = messages
                .iter()
                .filter_map(|message| match message {
                    CommandMessage::Play(note, _) => Some(*note),
                    _ => None,
                })
                .collect::<Chord>();

            self.chords.push(notes)
        };

        match result {
            Ok(_) => {}
            Err(_) => {
                // Vec full, time for the next state
                self.state = State::Playing { current_chord: 0 }
            }
        };
    }

    /// While playing, the following input is accepted:
    /// - Keys for playing a melody
    /// - Change tempo by holding FN1 and pressing C for tempo down, C# for tempo up
    /// - Change octave temporarily by holding FN0 (up) or FN2 (down)
    fn playing(&mut self, current_chord: usize, messages: &mut Vec<CommandMessage, 4>) {
        // TODO: drums?
        if self.loops > self.settings.loops_per_chord {
            let new_current_chord = current_chord + 1;
            self.state = State::Playing {
                current_chord: new_current_chord % self.chords.len(),
            };
            let chord = &self.chords[current_chord];

            chord.iter().for_each(|&note| {
                let _ = messages.push(CommandMessage::Play(note, U4F4::from_str("0.7").unwrap()));
            });
        }
    }
}

impl Interface for ChordLoopInterface {
    fn start(mut self) -> ! {
        let octave = 4;
        let attack = U4F4::ONE;

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

            match self.state {
                State::AwaitChords => self.await_chords(&mut messages),
                State::Playing { current_chord } => self.playing(current_chord, &mut messages),
                State::Paused => {
                    // same code as in sandbox for melody (still want to be able to play melodies even if the chords are paused)
                }
            }

            for message in messages {
                let command = Command {
                    address: 0x0,
                    message,
                };
                let command_serialized = command.serialize();

                self.hw.fifo.write(command_serialized);
            }

            self.loops += 1.0;
        }
    }
}
