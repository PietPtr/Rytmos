use fixed::types::{U12F4, U4F4};
use rytmos_engrave::staff::{Accidental, Note};

/// Commands for synths that can be serialized in a u32 so the fit in a Pico's FIFO.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CommandMessage {
    /// Note and velocity
    Play(Note, U4F4),
    /// Frequency and velocity. Drops two MSB's of attack in favor of frequency accuracy
    Frequency(U12F4, U4F4),
    /// Define default attack?
    SetAttack(U4F4),
    /// Play the tick of a metronome, with emphasis or not
    Tick(bool),
    /// Set the tempo of the synth in _sixteenths_ per minute (whatever that means for a synth)
    SetTempo(u16),
    /// Reconfiguration, any synth receives all data bits and can interpret them however they like
    Reconfigure(u32),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Command {
    pub address: u32, // Stored in a u32, supports only 4 bits addressing.
    pub message: CommandMessage,
}

#[cfg(feature = "defmt")]
impl defmt::Format for Command {
    fn format(&self, fmt: defmt::Formatter) {
        defmt::write!(fmt, "Command {{ address: {}, message: ", self.address);

        match &self.message {
            CommandMessage::Play(note, velocity) => {
                defmt::write!(
                    fmt,
                    "Play(Note: {:?}, Velocity: {:?})",
                    note,
                    velocity.to_num::<f32>()
                );
            }
            CommandMessage::Frequency(freq, velocity) => {
                defmt::write!(
                    fmt,
                    "Frequency(Frequency: {:?}, Velocity: {:?})",
                    freq.to_num::<f32>(),
                    velocity.to_num::<f32>()
                )
            }
            CommandMessage::SetAttack(attack) => {
                defmt::write!(fmt, "SetAttack({:?})", attack.to_num::<f32>());
            }
            CommandMessage::Tick(emphasis) => {
                defmt::write!(fmt, "Tick({})", emphasis);
            }
            CommandMessage::SetTempo(tempo) => {
                defmt::write!(fmt, "SetTempo({})", tempo);
            }
            CommandMessage::Reconfigure(data) => {
                defmt::write!(fmt, "Reconfigure({:032b})", data)
            }
        }

        defmt::write!(fmt, " }}");
    }
}

impl Command {
    pub fn serialize(&self) -> u32 {
        let address = self.address & 0b1111;
        match self.message {
            CommandMessage::Play(note, velocity) => {
                // 6 bits
                let command_id = 0b000000;

                // note: 3 bits
                let (note_bits, acc, octave) = match note {
                    Note::A(acc, octave) => (0, acc, octave),
                    Note::B(acc, octave) => (1, acc, octave),
                    Note::C(acc, octave) => (2, acc, octave),
                    Note::D(acc, octave) => (3, acc, octave),
                    Note::E(acc, octave) => (4, acc, octave),
                    Note::F(acc, octave) => (5, acc, octave),
                    Note::G(acc, octave) => (6, acc, octave),
                };

                // 3 bits
                let acc_bits: u32 = match acc {
                    Accidental::DoubleFlat => 0,
                    Accidental::Flat => 1,
                    Accidental::Natural => 2,
                    Accidental::Sharp => 3,
                    Accidental::DoubleSharp => 4,
                };

                // 4 bits
                let octave_bits = (octave & 0b1111) as u32;

                // velocity: 8 bits

                (velocity.to_bits() as u32)
                    | (note_bits << 8)
                    | (acc_bits << 11)
                    | (octave_bits << 14)
                    | (command_id << 22)
                    | (address << 28)
            }
            CommandMessage::SetAttack(attack) => {
                let command_id = 0b000001;
                (attack.to_bits() as u32) | (command_id << 22) | (address << 28)
            }
            CommandMessage::Tick(emphasis) => {
                let command_id = 0b000010;
                let emphasis = emphasis as u32;

                emphasis | (command_id << 22) | (address << 28)
            }
            CommandMessage::SetTempo(spm) => {
                let command_id = 0b000011;
                let spm = spm as u32;
                spm | (command_id << 22) | (address << 28)
            }
            CommandMessage::Frequency(freq, attack) => {
                let command_id = 0b000100;
                let attack = (attack.to_bits() & 0b11_1111) as u32;
                let frequency = (freq.to_bits()) as u32;

                frequency | (attack << 16) | (command_id << 22) | (address << 28)
            }
            CommandMessage::Reconfigure(data) => {
                let command_id = 0b000101;
                let data = data & 0b11_1111_1111_1111_1111_1111;
                data | (command_id << 22) | (address << 28)
            }
        }
    }

    pub fn deserialize(value: u32) -> Option<Self> {
        let command_id = (value >> 22) & 0b111111;
        let address = (value >> 28) & 0b1111;

        let message = match command_id {
            0 => {
                let velocity = value & 0xFF;
                let note_bits = (value >> 8) & 0b111;
                let acc_bits = (value >> 11) & 0b111;
                let octave_bits = (value >> 14) & 0b1111;
                let reserved = (value >> 18) & 0b1111;

                if reserved != 0 {
                    return None;
                }

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

                Some(CommandMessage::Play(note, U4F4::from_bits(velocity as u8)))
            }
            1 => {
                let attack = U4F4::from_bits(value as u8);
                let reserved = value & 0b00000000_00111111_11111111_00000000;
                if reserved == 0 {
                    Some(CommandMessage::SetAttack(attack))
                } else {
                    None
                }
            }
            2 => {
                let emphasis = (value & 1) == 1;
                let reserved = value & 0b00000000_00111111_11111111_11111110;
                if reserved == 0 {
                    Some(CommandMessage::Tick(emphasis))
                } else {
                    None
                }
            }
            3 => {
                let spm = (value & 0xffff) as u16;
                let reserved = value & 0b00000000_00111111_00000000_00000000;
                if reserved == 0 {
                    Some(CommandMessage::SetTempo(spm))
                } else {
                    None
                }
            }
            4 => {
                let freq = (value & 0xffff) as u16;
                let attack = ((value >> 16) & 0b111111) as u8;
                Some(CommandMessage::Frequency(
                    U12F4::from_bits(freq),
                    U4F4::from_bits(attack),
                ))
            }
            5 => {
                let data = value & 0b11_1111_1111_1111_1111_1111;
                Some(CommandMessage::Reconfigure(data))
            }
            _ => None,
        };

        message.map(|m| Self {
            address,
            message: m,
        })
    }
}
