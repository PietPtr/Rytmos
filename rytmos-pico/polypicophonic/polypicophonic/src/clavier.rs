use fixed::types::U4F4;
use heapless::Vec;

use common::debouncer::Debouncer;
use rytmos_engrave::staff::Note;
use rytmos_engrave::{a, ais, b, c, cis, d, dis, e, f, fis, g, gis};

use crate::io::ClavierPins;

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum KeyId {
    NoteC,
    NoteCis,
    NoteD,
    NoteDis,
    NoteE,
    NoteF,
    NoteFis,
    NoteG,
    NoteGis,
    NoteA,
    NoteAis,
    NoteB,
    Fn0,
    Fn1,
    Fn2,
    Fn3,
}

impl TryFrom<usize> for KeyId {
    type Error = ();

    fn try_from(value: usize) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(KeyId::NoteC),
            1 => Ok(KeyId::NoteCis),
            2 => Ok(KeyId::NoteD),
            3 => Ok(KeyId::NoteDis),
            4 => Ok(KeyId::NoteE),
            5 => Ok(KeyId::NoteF),
            6 => Ok(KeyId::NoteFis),
            7 => Ok(KeyId::NoteG),
            8 => Ok(KeyId::NoteGis),
            9 => Ok(KeyId::NoteA),
            10 => Ok(KeyId::NoteAis),
            11 => Ok(KeyId::NoteB),
            12 => Ok(KeyId::Fn0),
            13 => Ok(KeyId::Fn1),
            14 => Ok(KeyId::Fn2),
            15 => Ok(KeyId::Fn3),
            _ => Err(()),
        }
    }
}

pub struct Clavier<CLAVIER> {
    pub keys: CLAVIER,
    pub debouncers: [Debouncer; 16],
    last_note_events: Vec<NoteEvent, 4>,
    last_notes_state: [bool; 12],
}

const DEBOUNCE_TIME: u32 = 10;

impl<CLAVIER: ClavierPins> Clavier<CLAVIER> {
    pub fn new(keys: CLAVIER) -> Self {
        Self {
            keys,
            debouncers: [Debouncer::new(DEBOUNCE_TIME); 16],
            last_notes_state: [false; 12],
            last_note_events: Vec::new(),
        }
    }

    /// Updates the debouncers and reads and returns their state
    pub fn update_debouncers(&mut self) {
        for (id, debouncer) in self.debouncers.iter_mut().enumerate() {
            debouncer.update(self.keys.read(id.try_into().unwrap()));
        }
    }

    pub fn debouncer_is_high(&self, key: KeyId) -> bool {
        let key = key as usize;
        let Some(debouncer) = self.debouncers.get(key) else {
            log::error!("No debouncer found for KeyID {}.", key);
            return false;
        };

        debouncer.is_high().unwrap_or(false)
    }

    pub fn note_events(&self) -> &[NoteEvent] {
        &self.last_note_events
    }

    /// Reads all current note states, compares them with the last read,
    /// finds which events should be fired and returns those, and saves
    /// the read states.
    /// Does not debounce.
    pub(crate) fn update_note_events(&mut self) {
        let new_notes_states = [
            self.keys.read(KeyId::NoteC),
            self.keys.read(KeyId::NoteCis),
            self.keys.read(KeyId::NoteD),
            self.keys.read(KeyId::NoteDis),
            self.keys.read(KeyId::NoteE),
            self.keys.read(KeyId::NoteF),
            self.keys.read(KeyId::NoteFis),
            self.keys.read(KeyId::NoteG),
            self.keys.read(KeyId::NoteGis),
            self.keys.read(KeyId::NoteA),
            self.keys.read(KeyId::NoteAis),
            self.keys.read(KeyId::NoteB),
        ];

        macro_rules! key_down {
            ($note:ident) => {
                new_notes_states[KeyId::$note as usize]
                    && !self.last_notes_state[KeyId::$note as usize]
            };
        }

        macro_rules! key_up {
            ($note:ident) => {
                !new_notes_states[KeyId::$note as usize]
                    && self.last_notes_state[KeyId::$note as usize]
            };
        }

        // Of length four because the SIO fifo is length four

        let mut events: Vec<NoteEvent, 4> = Vec::new();

        if key_down!(NoteC) {
            let _ = events.push(NoteEvent::NoteDown(KeyId::NoteC));
        } else if key_up!(NoteC) {
            let _ = events.push(NoteEvent::NoteUp(KeyId::NoteC));
        }

        if key_down!(NoteCis) {
            let _ = events.push(NoteEvent::NoteDown(KeyId::NoteCis));
        } else if key_up!(NoteCis) {
            let _ = events.push(NoteEvent::NoteUp(KeyId::NoteCis));
        }

        if key_down!(NoteD) {
            let _ = events.push(NoteEvent::NoteDown(KeyId::NoteD));
        } else if key_up!(NoteD) {
            let _ = events.push(NoteEvent::NoteUp(KeyId::NoteD));
        }

        if key_down!(NoteDis) {
            let _ = events.push(NoteEvent::NoteDown(KeyId::NoteDis));
        } else if key_up!(NoteDis) {
            let _ = events.push(NoteEvent::NoteUp(KeyId::NoteDis));
        }

        if key_down!(NoteE) {
            let _ = events.push(NoteEvent::NoteDown(KeyId::NoteE));
        } else if key_up!(NoteE) {
            let _ = events.push(NoteEvent::NoteUp(KeyId::NoteE));
        }

        if key_down!(NoteF) {
            let _ = events.push(NoteEvent::NoteDown(KeyId::NoteF));
        } else if key_up!(NoteF) {
            let _ = events.push(NoteEvent::NoteUp(KeyId::NoteF));
        }

        if key_down!(NoteFis) {
            let _ = events.push(NoteEvent::NoteDown(KeyId::NoteFis));
        } else if key_up!(NoteFis) {
            let _ = events.push(NoteEvent::NoteUp(KeyId::NoteFis));
        }

        if key_down!(NoteG) {
            let _ = events.push(NoteEvent::NoteDown(KeyId::NoteG));
        } else if key_up!(NoteG) {
            let _ = events.push(NoteEvent::NoteUp(KeyId::NoteG));
        }

        if key_down!(NoteGis) {
            let _ = events.push(NoteEvent::NoteDown(KeyId::NoteGis));
        } else if key_up!(NoteGis) {
            let _ = events.push(NoteEvent::NoteUp(KeyId::NoteGis));
        }

        if key_down!(NoteA) {
            let _ = events.push(NoteEvent::NoteDown(KeyId::NoteA));
        } else if key_up!(NoteA) {
            let _ = events.push(NoteEvent::NoteUp(KeyId::NoteA));
        }

        if key_down!(NoteAis) {
            let _ = events.push(NoteEvent::NoteDown(KeyId::NoteAis));
        } else if key_up!(NoteAis) {
            let _ = events.push(NoteEvent::NoteUp(KeyId::NoteAis));
        }

        if key_down!(NoteB) {
            let _ = events.push(NoteEvent::NoteDown(KeyId::NoteB));
        } else if key_up!(NoteB) {
            let _ = events.push(NoteEvent::NoteUp(KeyId::NoteB));
        }

        self.last_notes_state = new_notes_states;
        self.last_note_events = events;
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum NoteEvent {
    NoteUp(KeyId),
    NoteDown(KeyId),
}

impl NoteEvent {
    pub fn velocity(&self, attack: U4F4) -> U4F4 {
        match self {
            NoteEvent::NoteUp(_) => U4F4::ZERO,
            NoteEvent::NoteDown(_) => attack,
        }
    }

    pub fn note(&self, octave: i32) -> Option<Note> {
        let key = self.key();

        match key {
            KeyId::NoteC => Some(c!(octave)),
            KeyId::NoteCis => Some(cis!(octave)),
            KeyId::NoteD => Some(d!(octave)),
            KeyId::NoteDis => Some(dis!(octave)),
            KeyId::NoteE => Some(e!(octave)),
            KeyId::NoteF => Some(f!(octave)),
            KeyId::NoteFis => Some(fis!(octave)),
            KeyId::NoteG => Some(g!(octave)),
            KeyId::NoteGis => Some(gis!(octave)),
            KeyId::NoteA => Some(a!(octave)),
            KeyId::NoteAis => Some(ais!(octave)),
            KeyId::NoteB => Some(b!(octave)),
            _ => None,
        }
    }

    pub fn key(&self) -> KeyId {
        match self {
            NoteEvent::NoteUp(key_id) => *key_id,
            NoteEvent::NoteDown(key_id) => *key_id,
        }
    }
}
