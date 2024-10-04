use rytmos_engrave::staff::{Accidental, Note};

/// Commands for synths that can be serialized in a u32 so the fit in a Pico's FIFO.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Command {
    /// Note and velocity (encoded as a (velocity / 256) * scale)
    Play(Note, u8, u8),
    SetAttack(u8, u8), // TODO: make something more ergonomic for this u8, u8 fixed point type
}

impl Command {
    pub fn serialize(&self) -> u32 {
        match *self {
            Command::Play(note, velocity, scale) => {
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

                (velocity as u32)
                    | ((scale as u32) << 8)
                    | (note_bits << 16)
                    | (acc_bits << 19)
                    | (octave_bits << 22)
                    | (command_id << 26)
            }
            Command::SetAttack(attack, scale) => {
                let command_id = 0b00001;
                (attack as u32) | ((scale as u32) << 8) | (command_id << 26)
            }
            _ => unimplemented!(),
        }
    }

    pub fn deserialize(value: u32) -> Option<Self> {
        let command_id = value >> 26 & 0b111111;

        match command_id {
            0 => {
                let velocity = value & 0xFF;
                let scale = (value >> 8) & 0xFF;
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

                Some(Self::Play(note, velocity as u8, scale as u8))
            }
            1 => {
                let attack = value as u8;
                let scale = (value >> 8) as u8;
                Some(Self::SetAttack(attack, scale))
            }
            _ => None,
        }
    }
}
