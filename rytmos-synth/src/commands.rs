use fixed::types::U8F8;
use rytmos_engrave::staff::{Accidental, Note};

/// Commands for synths that can be serialized in a u32 so the fit in a Pico's FIFO.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Command {
    /// Note and velocity
    Play(Note, U8F8),
    /// Define default attack?
    SetAttack(U8F8),
    /// Play the tick of a metronome, with emphasis or not
    Tick(bool),
    /// Set the tempo of the synth in _sixteenths_ per minute (whatever that means for a synth)
    SetTempo(u16),
}

impl Command {
    pub fn serialize(&self) -> u32 {
        match *self {
            Command::Play(note, velocity) => {
                let command_id = 0b000000;
                let (note_bits, acc, octave) = match note {
                    Note::A(acc, octave) => (0, acc, octave),
                    Note::B(acc, octave) => (1, acc, octave),
                    Note::C(acc, octave) => (2, acc, octave),
                    Note::D(acc, octave) => (3, acc, octave),
                    Note::E(acc, octave) => (4, acc, octave),
                    Note::F(acc, octave) => (5, acc, octave),
                    Note::G(acc, octave) => (6, acc, octave),
                };

                let acc_bits: u32 = match acc {
                    Accidental::DoubleFlat => 0,
                    Accidental::Flat => 1,
                    Accidental::Natural => 2,
                    Accidental::Sharp => 3,
                    Accidental::DoubleSharp => 4,
                };

                let octave_bits = (octave & 0b1111) as u32;

                (velocity.to_bits() as u32)
                    | (note_bits << 16)
                    | (acc_bits << 19)
                    | (octave_bits << 22)
                    | (command_id << 26)
            }
            Command::SetAttack(attack) => {
                let command_id = 0b00001;
                (attack.to_bits() as u32) | (command_id << 26)
            }
            Command::Tick(emphasis) => {
                let command_id = 0b00010;
                let emphasis = emphasis as u32;

                emphasis | (command_id << 26)
            }
            Command::SetTempo(spm) => {
                let command_id = 0b00011;
                let spm = spm as u32;
                spm | command_id << 26
            }
        }
    }

    pub fn deserialize(value: u32) -> Option<Self> {
        let command_id = value >> 26 & 0b111111;

        match command_id {
            0 => {
                let velocity = value & 0xFFFF;
                let note_bits = (value >> 16) & 0x7;
                let acc_bits = (value >> 19) & 0x7;
                let octave_bits = (value >> 22) & 0x3FF;

                let octave = octave_bits as i32;

                let acc = match acc_bits {
                    0 => Accidental::DoubleFlat,
                    1 => Accidental::Flat,
                    2 => Accidental::Natural,
                    3 => Accidental::Sharp,
                    4 => Accidental::DoubleSharp,
                    _ => return None,
                };

                let note = match note_bits {
                    0 => Note::A(acc, octave),
                    1 => Note::B(acc, octave),
                    2 => Note::C(acc, octave),
                    3 => Note::D(acc, octave),
                    4 => Note::E(acc, octave),
                    5 => Note::F(acc, octave),
                    6 => Note::G(acc, octave),
                    _ => return None,
                };

                Some(Self::Play(note, U8F8::from_bits(velocity as u16)))
            }
            1 => {
                let attack = U8F8::from_bits(value as u16);
                let reserved = value & 0b00000011_11111111_00000000_00000000;
                if reserved == 0 {
                    Some(Self::SetAttack(attack))
                } else {
                    None
                }
            }
            2 => {
                let emphasis = (value & 1) == 1;
                let reserved = value & 0b00000011_11111111_11111111_11111110;
                if reserved == 0 {
                    Some(Self::Tick(emphasis))
                } else {
                    None
                }
            }
            3 => {
                let spm = (value & 0xffff) as u16;
                let reserved = value & 0b00000011_11111111_00000000_00000000;
                if reserved == 0 {
                    Some(Self::SetTempo(spm))
                } else {
                    None
                }
            }
            _ => None,
        }
    }
}

// TODO: scope module / crate for testing synths on the go?
