use fixed::types::U4F4;
use heapless::Vec;
use rytmos_synth::{
    commands::{Command, CommandMessage},
    synth::drum,
};

#[derive(Debug, Default)]
struct Sequence {
    hat: [bool; 16],
    kick: [bool; 16],
    snare: [bool; 16],
}

#[derive(Debug, Default)]
struct Sequencer {
    sequence: Sequence,
    subdivision_index: u8,
    time_signature: SequenceTimeSignature,
}

impl Sequencer {
    pub fn update(&mut self, sequence: Sequence) {
        self.sequence = sequence
    }

    pub fn next(&mut self) -> Vec<Command, 3> {
        let subdiv = self.subdivision_index as usize;
        let next_subdiv = self.subdivision_index + 1;

        self.subdivision_index = if next_subdiv > self.time_signature.amount_of_subdivisions() {
            0
        } else {
            next_subdiv
        };

        let has_hat = self.sequence.hat.get(subdiv).copied().unwrap_or(false);
        let has_snare = self.sequence.snare.get(subdiv).copied().unwrap_or(false);
        let has_kick = self.sequence.kick.get(subdiv).copied().unwrap_or(false);

        let mut vec = Vec::new();

        if has_hat {
            vec.push(Command {
                address: 0,
                message: CommandMessage::Play(drum::HIHAT_NOTE, U4F4::ONE),
            })
            .unwrap();
        }

        if has_snare {
            vec.push(Command {
                address: 0,
                message: CommandMessage::Play(drum::SNARE_NOTE, U4F4::ONE),
            })
            .unwrap();
        }

        if has_kick {
            vec.push(Command {
                address: 0,
                message: CommandMessage::Play(drum::KICK_NOTE, U4F4::ONE),
            })
            .unwrap();
        }

        vec
    }
}

#[derive(Debug, Default)]
pub enum SequenceTimeSignature {
    #[default]
    FourFour,
    TwelveEight,
}

impl SequenceTimeSignature {
    fn amount_of_subdivisions(&self) -> u8 {
        match self {
            SequenceTimeSignature::FourFour => 16,
            SequenceTimeSignature::TwelveEight => 12,
        }
    }
}
