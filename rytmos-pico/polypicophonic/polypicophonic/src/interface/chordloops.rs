use enum_iterator::Sequence;
use fixed::types::U4F4;
use heapless::Vec;

use rytmos_engrave::staff::Note;
use rytmos_synth::commands::{Command, CommandMessage};

use crate::{
    chords::{self, ChordConstruction, ChordQuality},
    clavier::{Clavier, KeyId, NoteEvent},
    io::{self, IO},
};

use super::Interface;

const CHORDS_PER_LOOP: usize = 4;
const CONSTRUCTION: ChordConstruction = ChordConstruction::InvertToWithinOctave;

pub struct ChordLoopInterface<FIFO, CLAVIER> {
    fifo: FIFO,
    clavier: Clavier<CLAVIER>,
    state: State,
    settings: ChordLoopSettings,
    chords: Vec<Chord, CHORDS_PER_LOOP>,
    loops: f32,
}

struct ChordLoopSettings {
    loops_per_chord: f32, // proxy for tempo
    chord_attack: U4F4,
    melody_attack: U4F4,
    octave: i32,
    current_drum_track: DrumTrack,
}

type Chord = Vec<Note, CHORDS_PER_LOOP>;

enum State {
    AwaitChords,
    Playing { current_chord: usize },
    Paused,
}

#[derive(Debug, Sequence, Default)]
enum DrumTrack {
    #[default]
    NoDrums,
    Metronome,
    BackbeatOnly,
    SparseBeat,
    SimpleEights,
}

impl<FIFO: io::Fifo, CLAVIER: io::ClavierPins> ChordLoopInterface<FIFO, CLAVIER> {
    pub fn new(io: IO<FIFO, CLAVIER>) -> Self {
        Self {
            fifo: io.fifo,
            clavier: Clavier::new(io.clavier),
            chords: Vec::new(),
            state: State::AwaitChords,
            settings: ChordLoopSettings {
                loops_per_chord: 1000.,
                chord_attack: U4F4::from_str("0.7").unwrap(),
                melody_attack: U4F4::ONE,
                octave: 4,
                current_drum_track: DrumTrack::NoDrums,
            },
            loops: 0.0,
        }
    }

    /// While awaiting chords, it is only possible to input chords.
    /// Hold FN1 for major and FN3 for minor, then press the root note's key.
    fn await_chords(&mut self, messages: &mut Vec<CommandMessage, 4>) {
        if self.clavier.debouncer_is_high(KeyId::Fn1) {
            chords::add_chord(messages, ChordQuality::Major, CONSTRUCTION);
            chords::root_to_bass_register(messages);
        }

        if self.clavier.debouncer_is_high(KeyId::Fn3) {
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

    /// - Change tempo by holding FN1 and pressing C for tempo down, D for tempo up
    /// - Change octave temporarily by holding FN0 (up) or FN2 (down)
    /// - Change melody volume by holding FN1 and pressing E for volume down, F for volume up
    /// - Change chord volume by holding FN1 and pressing G for volume down, A for volume up
    /// - Change drum tracks by holding FN1 and pressing C# for next, D# for previous drum track (e.g. metronomes, simple beats, sparse beats, backbeat)
    /// - Play / Pause the chords by holding FN1 and pressing B.
    fn apply_settings(&mut self) {
        const TEMPO_MODIFIER: f32 = 1.1;
        const ATTACK_DIFF: U4F4 = U4F4::unwrapped_from_str("0.25");

        let c = &self.clavier;
        if c.debouncer_is_high(KeyId::Fn1) {
            if c.debouncer_is_high(KeyId::NoteC) {
                self.settings.loops_per_chord *= TEMPO_MODIFIER;
            }
            if c.debouncer_is_high(KeyId::NoteD) {
                self.settings.loops_per_chord /= TEMPO_MODIFIER;
                self.settings.loops_per_chord = self.settings.loops_per_chord.max(1.0);
            }
            if c.debouncer_is_high(KeyId::NoteE) {
                self.settings.melody_attack -= ATTACK_DIFF;
            }
            if c.debouncer_is_high(KeyId::NoteF) {
                self.settings.melody_attack += ATTACK_DIFF;
            }
            if c.debouncer_is_high(KeyId::NoteG) {
                self.settings.chord_attack -= ATTACK_DIFF;
            }
            if c.debouncer_is_high(KeyId::NoteA) {
                self.settings.chord_attack += ATTACK_DIFF;
            }
            if c.debouncer_is_high(KeyId::NoteCis) {
                self.settings.current_drum_track =
                    self.settings.current_drum_track.next().unwrap_or_default();
            }
            if c.debouncer_is_high(KeyId::NoteDis) {
                self.settings.current_drum_track = self
                    .settings
                    .current_drum_track
                    .previous()
                    .unwrap_or(enum_iterator::last().unwrap());
            }

            if self
                .clavier
                .note_events()
                .contains(&NoteEvent::NoteUp(KeyId::NoteB))
            {
                match self.state {
                    State::Playing { current_chord: _ } => self.state = State::Paused,
                    State::Paused => self.state = State::Playing { current_chord: 0 },
                    _ => {}
                }
            }
        }
    }

    /// While playing, the following input is accepted:
    /// - Keys for playing a melody
    /// - Any keycombos for making changes to settings
    fn playing(&mut self, current_chord: usize, messages: &mut Vec<CommandMessage, 4>) {
        // TODO: generate drum messages drums, but the vec might get full
        // TODO: incorporate the sequencer maybe somehow?
        if self.loops > self.settings.loops_per_chord {
            let new_current_chord = current_chord + 1;
            self.state = State::Playing {
                current_chord: new_current_chord % self.chords.len(),
            };
            let chord = &self.chords[current_chord];

            chord.iter().for_each(|&note| {
                let _ = messages.push(CommandMessage::Play(note, self.settings.chord_attack));
            });
        }

        self.apply_settings();
    }

    fn paused(&mut self) {
        self.apply_settings();
    }
}

impl<FIFO: io::Fifo, CLAVIER: io::ClavierPins> Interface for ChordLoopInterface<FIFO, CLAVIER> {
    fn run(&mut self) {
        self.clavier.update_debouncers();
        self.clavier.update_note_events();

        if self.clavier.debouncer_is_high(KeyId::Fn0) {
            self.settings.octave = 5
        } else if self.clavier.debouncer_is_high(KeyId::Fn2) {
            self.settings.octave = 3
        } else {
            self.settings.octave = 4
        }

        // If the FN1 key is pressed, ignore notes since the note keys are used as settings
        let events = if !self.clavier.debouncer_is_high(KeyId::Fn1) {
            self.clavier.note_events()
        } else {
            &[]
        };

        let mut messages = events
            .iter()
            .filter_map(|event| {
                Some(CommandMessage::Play(
                    event.note(self.settings.octave)?,
                    event.velocity(self.settings.melody_attack),
                ))
            })
            .collect::<Vec<_, 4>>();

        match self.state {
            State::AwaitChords => self.await_chords(&mut messages),
            State::Playing { current_chord } => self.playing(current_chord, &mut messages),
            State::Paused => self.paused(),
        }

        for message in messages {
            let command = Command {
                address: 0x0,
                message,
            };
            let command_serialized = command.serialize();

            self.fifo.write(command_serialized);
        }

        self.loops += 1.0;
    }
}
