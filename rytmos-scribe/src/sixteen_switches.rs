use embedded_graphics::pixelcolor::BinaryColor;
use embedded_graphics::prelude::*;
use embedded_graphics::primitives::{PrimitiveStyle, Rectangle, StyledDrawable};
use heapless::Vec;
use rytmos_engrave::staff::{Accidental, Duration as Dur};
use rytmos_engrave::staff::{Music, Note};

/// Generates Rytmos sheet music based on the states of sixteen tri state switches.
/// The switches states mean:
/// Up:         Attack
/// Neutral:    Do nothing
/// Down:       Mute
/// Modelling actions a musician would take on a stringed rhythm instrument.
pub struct Generate {}

#[derive(Debug)]
pub enum ScribeError {
    InvalidSixteenthIndex(usize),
    InvalidPlaydefLength(u8),
    VecFull,
    MoreThanFourBeatsInCurrentBeat(u8),
    MoreThanFourBeatsAfterFirstBeatAfterRender(u8),
    DurationMustBeLessThanFourNow(u8),
}

/// Encodes the three states of a switch
#[derive(Debug, Clone, PartialEq, Eq, Copy, Default)]
pub enum SwitchState {
    Atck,
    #[default]
    Noop,
    Mute,
}

/// A measure consists of 16 sixteenth notes and defines its rhythm with 16 switch states.
#[derive(Debug, Clone, PartialEq, Eq, Copy, Default)]
pub struct MeasureState {
    states: [SwitchState; 16],
}

impl MeasureState {
    pub fn new(states: [SwitchState; 16]) -> Self {
        Self { states }
    }

    pub fn set(&mut self, idx: usize, state: SwitchState) -> Result<(), ScribeError> {
        if idx > 15 {
            return Err(ScribeError::InvalidSixteenthIndex(idx));
        }
        self.states[idx] = state;

        Ok(())
    }

    pub fn draw<D>(&self, target: &mut D, position: Point) -> Result<(), D::Error>
    where
        D: DrawTarget<Color = BinaryColor>,
    {
        let style = PrimitiveStyle::with_stroke(BinaryColor::On, 1);

        for (i, state) in self.states.iter().enumerate() {
            let y = match state {
                SwitchState::Atck => 0,
                SwitchState::Noop => 1,
                SwitchState::Mute => 2,
            };

            Rectangle::new(
                position + Point { x: i as i32, y: y },
                Size {
                    width: 1,
                    height: 1,
                },
            )
            .draw_styled(&style, target)?;
        }

        Ok(())
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
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct PlayDefinition {
    pub sixteenths: Vec<StringState, 16>,
}

impl PlayDefinition {
    pub fn new(sixteenths: Vec<StringState, 16>) -> Result<Self, ScribeError> {
        let mut sum = 0;
        for state in sixteenths.iter() {
            sum += state.time_value();
        }

        if sum != 16 {
            return Err(ScribeError::InvalidPlaydefLength(sum));
        }

        Ok(Self { sixteenths })
    }

    /// Turns a play definition rhythm into Rytmos music, using C as note.
    /// Uses the fact that a playdefinition is always exactly 1 measure (checked in new)
    pub fn to_music(&self) -> Result<Vec<Music, 16>, ScribeError> {
        let note = |dur| Music::Note(Note::C(Accidental::Natural, 3), dur);
        let rest = |dur| Music::Rest(dur);

        let mut music = Vec::new();

        let mut beat_pos = 0;
        for state in self.sixteenths.iter() {
            match state {
                StringState::Ringing(i) => {
                    let duration = *i;
                    let length_in_current_beat = (4 - beat_pos).min(*i);

                    // Catch some special cases first
                    match (beat_pos, duration) {
                        (0, 6 | 8 | 12 | 16) => {
                            let duration_enum = match duration {
                                6 => Dur::DottedQuarter,
                                8 => Dur::Half,
                                12 => Dur::DottedHalf,
                                16 => Dur::Whole,
                                _ => unreachable!(),
                            };
                            music
                                .push(note(duration_enum))
                                .map_err(|_| ScribeError::VecFull)?;
                            beat_pos = (beat_pos + duration) % 4;
                            continue;
                        }
                        _ => (),
                    }

                    match length_in_current_beat {
                        1 => music
                            .push(note(Dur::Sixteenth))
                            .map_err(|_| ScribeError::VecFull)?,
                        2 => music
                            .push(note(Dur::Eighth))
                            .map_err(|_| ScribeError::VecFull)?,
                        3 => music
                            .push(note(Dur::DottedEighth))
                            .map_err(|_| ScribeError::VecFull)?,
                        4 => music
                            .push(note(Dur::Quarter))
                            .map_err(|_| ScribeError::VecFull)?,
                        _ => {
                            return Err(ScribeError::MoreThanFourBeatsInCurrentBeat(
                                length_in_current_beat,
                            ));
                        }
                    }

                    let mut after_first_beat_duration = duration - length_in_current_beat;

                    if after_first_beat_duration > 0 {
                        music.push(Music::Tie).map_err(|_| ScribeError::VecFull)?;
                    }

                    for _ in 0..(after_first_beat_duration / 4) {
                        music
                            .push(note(Dur::Quarter))
                            .map_err(|_| ScribeError::VecFull)?;
                        after_first_beat_duration -= 4;
                        if after_first_beat_duration > 0 {
                            music.push(Music::Tie).map_err(|_| ScribeError::VecFull)?;
                        }
                    }

                    if after_first_beat_duration >= 4 {
                        return Err(ScribeError::MoreThanFourBeatsAfterFirstBeatAfterRender(
                            after_first_beat_duration,
                        ));
                    }

                    match after_first_beat_duration {
                        0 => (),
                        1 => music
                            .push(note(Dur::Sixteenth))
                            .map_err(|_| ScribeError::VecFull)?,
                        2 => music
                            .push(note(Dur::Eighth))
                            .map_err(|_| ScribeError::VecFull)?,
                        3 => music
                            .push(note(Dur::DottedEighth))
                            .map_err(|_| ScribeError::VecFull)?,
                        _ => {
                            return Err(ScribeError::DurationMustBeLessThanFourNow(
                                after_first_beat_duration,
                            ));
                        }
                    }

                    beat_pos = (beat_pos + duration) % 4;
                }
                StringState::Silent(i) => {
                    let duration = *i;
                    let length_in_current_beat = (4 - beat_pos).min(*i);

                    // Catch some special cases first
                    match (beat_pos, duration) {
                        (0, 6 | 8 | 12 | 16) => {
                            let duration_enum = match duration {
                                6 => Dur::DottedQuarter,
                                8 => Dur::Half,
                                12 => Dur::DottedHalf,
                                16 => Dur::Whole,
                                _ => unreachable!(),
                            };
                            music
                                .push(rest(duration_enum))
                                .map_err(|_| ScribeError::VecFull)?;
                            beat_pos = (beat_pos + duration) % 4;
                            continue;
                        }
                        _ => (),
                    }

                    match length_in_current_beat {
                        1 => music
                            .push(rest(Dur::Sixteenth))
                            .map_err(|_| ScribeError::VecFull)?,
                        2 => music
                            .push(rest(Dur::Eighth))
                            .map_err(|_| ScribeError::VecFull)?,
                        3 => music
                            .push(rest(Dur::DottedEighth))
                            .map_err(|_| ScribeError::VecFull)?,
                        4 => music
                            .push(rest(Dur::Quarter))
                            .map_err(|_| ScribeError::VecFull)?,
                        _ => {
                            return Err(ScribeError::MoreThanFourBeatsInCurrentBeat(
                                length_in_current_beat,
                            ));
                        }
                    }

                    let mut after_first_beat_duration = duration - length_in_current_beat;

                    for _ in 0..(after_first_beat_duration / 4) {
                        music
                            .push(rest(Dur::Quarter))
                            .map_err(|_| ScribeError::VecFull)?;
                        after_first_beat_duration -= 4;
                    }

                    if after_first_beat_duration >= 4 {
                        return Err(ScribeError::DurationMustBeLessThanFourNow(
                            after_first_beat_duration,
                        ));
                    }

                    match after_first_beat_duration {
                        0 => (),
                        1 => music
                            .push(rest(Dur::Sixteenth))
                            .map_err(|_| ScribeError::VecFull)?,
                        2 => music
                            .push(rest(Dur::Eighth))
                            .map_err(|_| ScribeError::VecFull)?,
                        3 => music
                            .push(rest(Dur::DottedEighth))
                            .map_err(|_| ScribeError::VecFull)?,
                        _ => {
                            return Err(ScribeError::DurationMustBeLessThanFourNow(
                                after_first_beat_duration,
                            ))
                        }
                    }

                    beat_pos = (beat_pos + duration) % 4;
                }
            }
        }

        Ok(music)
    }
}
impl TryFrom<MeasureState> for PlayDefinition {
    type Error = ScribeError;
    fn try_from(measure: MeasureState) -> Result<Self, Self::Error> {
        let mut play = Vec::new();

        for state in measure.states.iter() {
            if play.is_empty() {
                let next = match state {
                    SwitchState::Atck => StringState::Ringing(1),
                    SwitchState::Noop => StringState::Silent(1),
                    SwitchState::Mute => StringState::Silent(1),
                };

                play.push(next).map_err(|_| ScribeError::VecFull)?;
            } else {
                let current_state = play.last_mut().unwrap();
                match state {
                    SwitchState::Atck => play
                        .push(StringState::Ringing(1))
                        .map_err(|_| ScribeError::VecFull)?,
                    SwitchState::Noop => current_state.increment(),
                    SwitchState::Mute => play
                        .push(StringState::Silent(1))
                        .map_err(|_| ScribeError::VecFull)?,
                }
            }
        }

        // Unwrap is safe and on each loop we either increment or add a new element of size 1 so
        // this new doesn't throw errors.
        PlayDefinition::new(play)
    }
}
