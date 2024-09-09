use embedded_graphics::{
    pixelcolor::BinaryColor,
    prelude::*,
    primitives::{Line, PrimitiveStyle, Rectangle, StyledDrawable},
};
use rytmos_engrave::symbols::{
    BEAT_A, BEAT_AND, BEAT_E, BEAT_FOUR, BEAT_ONE, BEAT_THREE, BEAT_TWO,
};
use rytmos_scribe::sixteen_switches::{RhythmDefinition, StringState};

// We track playing at a resolution of 128th notes
const PLAYED_BUFFER_SIZE: usize = 128;

/// Shows the beats and subdivisions, a line with how the current rhythm is defined, and what the user just played.
/// Always draws at 128 pixels width.
pub struct PlayAnalysis {
    current_rhythm: RhythmDefinition,
    correct_rhythm: [bool; PLAYED_BUFFER_SIZE],
    played_buffer: [bool; PLAYED_BUFFER_SIZE],
    play_ptr: usize,
}

impl PlayAnalysis {
    pub fn new(current_rhythm: RhythmDefinition) -> Self {
        let mut s = Self {
            current_rhythm: RhythmDefinition::default(),
            correct_rhythm: [false; PLAYED_BUFFER_SIZE],
            played_buffer: [false; PLAYED_BUFFER_SIZE],
            play_ptr: 0,
        };

        s.set_rhythm(current_rhythm);

        s
    }

    pub fn step(&mut self, ringing: bool) {
        self.played_buffer[self.play_ptr] = ringing;

        self.play_ptr = (self.play_ptr + 1) % PLAYED_BUFFER_SIZE
    }

    pub fn step_size_ms(bpm: u32) -> u32 {
        libm::roundf((60_000.0 / bpm as f32) / 32.0) as u32
    }

    pub fn set_rhythm(&mut self, rhythm: RhythmDefinition) {
        self.current_rhythm = rhythm;

        let mut instant: usize = 0;
        for string_action in self.current_rhythm.sixteenths.iter() {
            match string_action {
                StringState::Ringing(time) => {
                    let rings_until = instant + *time as usize * 8 - 1;
                    while instant < rings_until {
                        self.correct_rhythm[instant] = true;
                        instant += 1;
                    }
                    instant += 1;
                }
                StringState::Silent(time) => {
                    let silent_until = instant + *time as usize * 8 - 1;
                    while instant < silent_until {
                        self.correct_rhythm[instant] = false;
                        instant += 1;
                    }
                    instant += 1;
                }
            }
        }
    }

    pub fn draw<D>(&self, target: &mut D, position: Point) -> Result<(), D::Error>
    where
        D: DrawTarget<Color = BinaryColor>,
    {
        let mut range = (1..130).step_by(8);
        let mut x = || Point::new(range.next().unwrap(), 0);

        #[rustfmt::skip]
        let beats = [
            BEAT_ONE,   BEAT_E, BEAT_AND, BEAT_A,
            BEAT_TWO,   BEAT_E, BEAT_AND, BEAT_E,
            BEAT_THREE, BEAT_E, BEAT_AND, BEAT_A,
            BEAT_FOUR,  BEAT_E, BEAT_AND, BEAT_A,
        ];

        let style = PrimitiveStyle::with_stroke(BinaryColor::On, 1);
        let off_style = PrimitiveStyle::with_stroke(BinaryColor::Off, 1);

        for (sixteenth, &beat) in beats.iter().enumerate() {
            rytmos_engrave::symbols::draw_symbol(target, position + x(), beat)?;
            Rectangle::new(
                position
                    + Point {
                        x: (sixteenth * 8) as i32,
                        y: 6,
                    },
                Size {
                    width: 1,
                    height: 1,
                },
            )
            .draw_styled(&style, target)?;
        }

        let mut sixteenth = 0;
        for string_action in self.current_rhythm.sixteenths.iter() {
            match string_action {
                StringState::Ringing(time) => {
                    Line::new(
                        position
                            + Point {
                                x: (sixteenth * 8) as i32,
                                y: 7,
                            },
                        position
                            + Point {
                                x: ((sixteenth as i32 + *time as i32) * 8) - 2,
                                y: 7,
                            },
                    )
                    .draw_styled(&style, target)?;
                    sixteenth += time;
                }
                StringState::Silent(time) => {
                    sixteenth += time;
                }
            }
        }

        for (div, (&played, &shouldve_played)) in self
            .played_buffer
            .iter()
            .zip(self.correct_rhythm.iter())
            .enumerate()
        {
            if played && shouldve_played {
                Rectangle::new(
                    position
                        + Point {
                            x: div as i32,
                            y: 8,
                        },
                    Size {
                        width: 1,
                        height: 1,
                    },
                )
                .draw_styled(&style, target)?;
            }

            if played && !shouldve_played {
                // TODO: currently the flickering is incorrect, might be nice to remake this thing as a kind of waveform ?
                if self.play_ptr % 2 == 0 {
                    Rectangle::new(
                        position
                            + Point {
                                x: div as i32,
                                y: 8,
                            },
                        Size {
                            width: 1,
                            height: 1,
                        },
                    )
                    .draw_styled(&style, target)?;
                }
            }
        }

        let now_x = (self.play_ptr) as i32;
        Line::new(
            position + Point { x: now_x, y: 5 },
            position + Point { x: now_x, y: 9 },
        )
        .draw_styled(&style, target)?;
        Line::new(
            position + Point { x: now_x + 1, y: 5 },
            position + Point { x: now_x + 1, y: 9 },
        )
        .draw_styled(&off_style, target)?;

        Ok(())
    }
}
