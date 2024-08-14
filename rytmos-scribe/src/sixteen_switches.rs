use rytmos::staff::{Accidental, Duration as Dur};
use rytmos::staff::{Music, Note};

/// Generates Rytmos sheet music based on the states of sixteen tri state switches.
/// The switches states mean:
/// Up:         Attack
/// Neutral:    Do nothing
/// Down:       Mute
/// Modelling actions a musician would take on a stringed rhythm instrument.
pub struct Generate {}

/// Encodes the three states of a switch
#[derive(Debug, Clone, PartialEq, Eq, Copy)]
pub enum SwitchState {
    Atck,
    Noop,
    Mute,
}

/// A measure consists of 16 sixteenth notes and defines its rhythm with 16 switch states.
pub struct MeasureState {
    states: [SwitchState; 16],
}

impl MeasureState {
    pub fn new(states: [SwitchState; 16]) -> Self {
        Self { states }
    }
}

/// State of a string over the next u8 sixteenth notes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StringState {
    Ringing(u8),
    Silent(u8),
}

impl StringState {
    pub fn increment(&mut self) {
        match self {
            StringState::Ringing(ref mut i) => *i += 1,
            StringState::Silent(ref mut i) => *i += 1,
        }
    }

    pub fn decrement(&mut self) {
        match self {
            StringState::Ringing(ref mut i) => *i -= 1,
            StringState::Silent(ref mut i) => *i -= 1,
        }
    }

    fn time_value(&self) -> u8 {
        match self {
            StringState::Ringing(i) => *i,
            StringState::Silent(i) => *i,
        }
    }
}

/// Defines when a string is ringing and for how long. Converted from MeasureState,
/// and converted into Rytmos notation
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlayDefinition {
    sixteenths: Vec<StringState>,
}

impl PlayDefinition {
    pub fn new(sixteenths: Vec<StringState>) -> Result<Self, String> {
        let mut sum = 0;
        for state in sixteenths.iter() {
            sum += state.time_value();
        }

        if sum != 16 {
            return Err(format!(
                "PlayDefinition was not exactly 16 sixteenths: {sum}"
            ));
        }

        Ok(Self { sixteenths })
    }

    /// Turns a play definition rhythm into Rytmos music, using C as note.
    pub fn to_music(&self) -> Result<Vec<Music>, String> {
        let note = |dur| Music::Note(Note::C(Accidental::Natural, 3), dur);

        let mut music = vec![];

        let mut beat_pos = 0;
        for state in self.sixteenths.iter() {
            match state {
                StringState::Ringing(1) => music.push(note(Dur::Sixteenth)),
                StringState::Ringing(i) => {
                    let length_in_beats = *i / 4.;
                    let length_in_current_beat = (4 - beat_pos).min(*i);

                    // Catch some special cases first
                    match (beat_pos, length_in_beats, *i) {
                        (0, 8, 8) => music.push(note(Dur::Half)),
                        (0, 12, 12) => music.push(note(Dur::DottedHalf)),
                        (0, 16, 16) => music.push(note(Dur::Whole)),
                    }

                    match length_in_current_beat {
                        1 => music.push(note(Dur::Sixteenth)),
                        2 => music.push(note(Dur::Eighth)),
                        3 => music.push(note(Dur::DottedEighth)),
                        4 => music.push(note(Dur::Quarter)),
                        _ => {
                            return Err(format!(
                                "Cannot have >4 beats in current beat: {length_in_current_beat}"
                            ))
                        }
                    }
                    // TODO: continue
                }
                StringState::Silent(i) => todo!(),
            }
        }

        todo!()
    }
}
impl From<MeasureState> for PlayDefinition {
    fn from(measure: MeasureState) -> Self {
        let mut play = vec![];

        for state in measure.states.iter() {
            if play.is_empty() {
                let next = match state {
                    SwitchState::Atck => StringState::Ringing(1),
                    SwitchState::Noop => StringState::Silent(1),
                    SwitchState::Mute => StringState::Silent(1),
                };

                play.push(next);
            } else {
                let current_state = play.last_mut().unwrap();
                match state {
                    SwitchState::Atck => play.push(StringState::Ringing(1)),
                    SwitchState::Noop => current_state.increment(),
                    SwitchState::Mute => play.push(StringState::Silent(1)),
                }
            }
        }

        // Unwrap is safe and on each loop we either increment or add a new element of size 1 so
        // this new doesn't throw errors.
        PlayDefinition::new(play).unwrap()
    }
}
