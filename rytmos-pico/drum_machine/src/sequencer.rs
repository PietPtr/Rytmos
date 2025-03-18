use fixed::types::U4F4;
use heapless::Vec;
use rytmos_synth::{
    commands::{Command, CommandMessage},
    synth::drum,
};

#[derive(Debug, Default)]
pub struct SingleSampleSequence {
    pub subdivs: [bool; 16],
    pub velocity: U4F4,
}

#[derive(Debug, Default)]
pub struct Sequence {
    pub hat: SingleSampleSequence,
    pub kick: SingleSampleSequence,
    pub snare: SingleSampleSequence,
}

#[derive(Debug, Default)]
pub struct Sequencer {
    sequence: Sequence,
    subdivision_index: u8,
    state: SequencerState,
    pub time_signature: SequenceTimeSignature,
    pub cymbal_every_four_measures: bool,
}

#[derive(Debug, Default)]
enum SequencerState {
    #[default]
    Stopped,
    CountOffSlow,
    CountOffFast,
    Playing(u8),
}

impl Sequencer {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn change_sequence(&mut self, sequence: Sequence) {
        self.sequence = sequence
    }

    pub fn current_subdivision(&self) -> u8 {
        self.subdivision_index
    }

    pub fn stop(&mut self) {
        self.state = SequencerState::Stopped;
    }

    pub fn play(&mut self) {
        self.subdivision_index = 0;
        self.state = SequencerState::Playing(0);
    }

    pub fn play_with_countoff(&mut self) {
        self.state = SequencerState::CountOffSlow;
    }

    pub fn next_subdivision(&mut self) -> Vec<Command, 4> {
        let subdiv = self.subdivision_index as usize;
        let next_subdiv = self.subdivision_index + 1;

        self.subdivision_index = if next_subdiv > self.time_signature.amount_of_subdivisions() {
            0
        } else {
            next_subdiv
        };

        match self.state {
            SequencerState::Stopped => Vec::new(),
            SequencerState::CountOffSlow => {
                self.state = SequencerState::CountOffFast;
                let (one, two) = match self.time_signature {
                    SequenceTimeSignature::FourFour => (0, 8),
                    SequenceTimeSignature::TwelveEight => (0, 6),
                };

                if subdiv == one || subdiv == two {
                    let mut vec = Vec::new();
                    vec.push(Command {
                        address: 0,
                        message: CommandMessage::Play(drum::STICKS_NOTE, U4F4::ONE),
                    })
                    .unwrap();

                    vec
                } else {
                    Vec::new()
                }
            }
            SequencerState::CountOffFast => {
                self.state = SequencerState::Playing(0);

                let (one, two, three, four) = match self.time_signature {
                    SequenceTimeSignature::FourFour => (0, 4, 8, 12),
                    SequenceTimeSignature::TwelveEight => (0, 3, 6, 9),
                };

                if subdiv == one || subdiv == two || subdiv == three || subdiv == four {
                    let mut vec = Vec::new();
                    vec.push(Command {
                        address: 0,
                        message: CommandMessage::Play(drum::STICKS_NOTE, U4F4::ONE),
                    })
                    .unwrap();

                    vec
                } else {
                    Vec::new()
                }
            }
            SequencerState::Playing(measure) => {
                self.state = SequencerState::Playing((measure + 1) & 0b11);

                let has_hat = self
                    .sequence
                    .hat
                    .subdivs
                    .get(subdiv)
                    .copied()
                    .unwrap_or(false);
                let has_snare = self
                    .sequence
                    .snare
                    .subdivs
                    .get(subdiv)
                    .copied()
                    .unwrap_or(false);
                let has_kick = self
                    .sequence
                    .kick
                    .subdivs
                    .get(subdiv)
                    .copied()
                    .unwrap_or(false);

                let mut vec = Vec::new();

                if self.cymbal_every_four_measures && measure == 0 && subdiv == 0 {
                    vec.push(Command {
                        address: 0,
                        message: CommandMessage::Play(drum::CYMBAL_NOTE, U4F4::ONE),
                    })
                    .unwrap()
                }

                if has_hat {
                    vec.push(Command {
                        address: 0,
                        message: CommandMessage::Play(drum::HIHAT_NOTE, self.sequence.hat.velocity),
                    })
                    .unwrap();
                }

                if has_snare {
                    vec.push(Command {
                        address: 0,
                        message: CommandMessage::Play(
                            drum::SNARE_NOTE,
                            self.sequence.snare.velocity,
                        ),
                    })
                    .unwrap();
                }

                if has_kick {
                    vec.push(Command {
                        address: 0,
                        message: CommandMessage::Play(drum::KICK_NOTE, self.sequence.kick.velocity),
                    })
                    .unwrap();
                }

                vec
            }
        }
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
