use embedded_graphics::{
    draw_target::DrawTarget,
    geometry::Point,
    pixelcolor::BinaryColor,
    prelude::Size,
    primitives::{Line, Primitive, PrimitiveStyle, Rectangle, StyledDrawable},
    Drawable,
};
use heapless::{FnvIndexMap, Vec};
use log::{debug, error};

use crate::symbols::{
    BASS_CLEF, DOTTED_EIGHTH_REST, DOTTED_HALF_REST, DOTTED_QUARTER_REST, EIGHTH_REST, EIGHT_FLAG,
    EMPTY_NOTEHEAD, FILLED_NOTEHEAD, HALF_REST, QUARTER_REST, SIXTEENTH_FLAG, SIXTEENTH_REST,
    WHOLE_REST,
};

#[derive(Debug)]
pub enum EngraveError {
    TooManyMusicSymbolsForVecAllocation,
    MoreMusicSymbolsThanSpacedSymbolsAccountedFor,
    NotEnoughSpaceForSymbols,
    Impossible,
    BeatMapInsertError,
}

#[derive(Clone, Copy)]
pub enum StaffElement<'a> {
    Music(&'a [Music]),
    Barline,
    KeySignature(Key),
    Clef(Clef),
}

#[derive(Debug, Clone, Copy)]
pub enum Music {
    Note(Note, Duration),
    Rest(Duration),
    Tie,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum StemDirection {
    Up,
    Down,
    NotApplicable,
}

#[derive(Debug, Copy, Clone)]
struct MusicSymbol {
    y: i32,
    x: Option<i32>,
    stem_direction: StemDirection,
    stem_length: i32,
    kind: Duration,
    rest: bool,
    tied: bool,
}

#[derive(Debug)]
struct MusicSymbolDefinitions {
    symbols: Vec<MusicSymbol, 16>,
}

impl MusicSymbolDefinitions {
    const REST_OFFSET: i32 = Staff::LEDGER_MARGIN + Staff::LINE_SPACING - 2;
    const DEFAULT_STEM_LENGTH: i32 = 9;

    fn new(music: &[Music]) -> Result<Self, EngraveError> {
        let mut symbols = Vec::new();

        for &symbol in music {
            match symbol {
                Music::Note(note, duration) => {
                    let head = MusicSymbol {
                        y: note.y_offset(),
                        x: None,
                        stem_direction: note.default_stem_direction(),
                        stem_length: Self::DEFAULT_STEM_LENGTH,
                        kind: duration,
                        rest: false,
                        tied: false,
                    };
                    symbols
                        .push(head)
                        .map_err(|_| EngraveError::TooManyMusicSymbolsForVecAllocation)?;
                }
                Music::Rest(duration) => {
                    let head = MusicSymbol {
                        y: Self::REST_OFFSET,
                        x: None,
                        stem_direction: StemDirection::NotApplicable,
                        stem_length: Self::DEFAULT_STEM_LENGTH,
                        kind: duration,
                        rest: true,
                        tied: false,
                    };
                    symbols
                        .push(head)
                        .map_err(|_| EngraveError::TooManyMusicSymbolsForVecAllocation)?;
                }
                Music::Tie => {
                    symbols.last_mut().map(|s| s.tied = true);
                }
            }
        }

        Ok(Self { symbols })
    }
}

#[derive(Debug)]
struct SpacedMusicSymbol {
    symbol: MusicSymbol,
    space: i32,
}

#[derive(Debug)]
struct SpacedMusicSymbols {
    symbols: Vec<SpacedMusicSymbol, 16>,
}

impl SpacedMusicSymbols {
    pub const MINIMUM_SYMBOL_SPACE: i32 = 7;
    fn new(symbols: MusicSymbolDefinitions, width: i32) -> Result<Self, EngraveError> {
        let mut spaced_symbols = Vec::new();

        let n = symbols.symbols.len() as i32;
        let base_space = width / n;
        let extra_space = width % n;

        let spaced = if extra_space > 0 {
            n / extra_space
        } else {
            width
        };

        for (index, symbol) in symbols.symbols.into_iter().enumerate() {
            let space = if index as i32 % spaced == 0 {
                base_space + 1
            } else {
                base_space
            };

            if space < Self::MINIMUM_SYMBOL_SPACE {
                return Err(EngraveError::NotEnoughSpaceForSymbols);
            }

            let spaced_symbol = SpacedMusicSymbol { symbol, space };
            spaced_symbols
                .push(spaced_symbol)
                .map_err(|_| EngraveError::MoreMusicSymbolsThanSpacedSymbolsAccountedFor)?;
        }

        Ok(Self {
            symbols: spaced_symbols,
        })
    }
}

/// Collection of symbols that should be rendered as a single glyph,
/// e.g. a single note, a single rest, or a group of beamed notes.
#[derive(Debug)]
struct GlyphDefinition {
    symbols: Vec<SpacedMusicSymbol, 4>,
    beamed: bool,
}

#[derive(Debug)]
struct Glyphs {
    glyphs: Vec<GlyphDefinition, 16>,
}

impl Glyphs {
    fn new(symbols: SpacedMusicSymbols) -> Result<Self, EngraveError> {
        let mut beat_tracker = 0.;

        // assign a beat to each symbol
        let mut beat_map: FnvIndexMap<u32, Vec<SpacedMusicSymbol, 4>, 16> = FnvIndexMap::new();

        for symbol in symbols.symbols.into_iter() {
            let duration_value = symbol.symbol.kind.value();

            let entry = beat_map.entry(beat_tracker as u32);

            match entry {
                heapless::Entry::Occupied(mut o) => {
                    o.get_mut()
                        .push(symbol)
                        .map_err(|_| EngraveError::BeatMapInsertError)?;
                }
                heapless::Entry::Vacant(e) => {
                    let mut new_vec = Vec::new();
                    new_vec
                        .push(symbol)
                        .map_err(|_| EngraveError::BeatMapInsertError)?;
                    e.insert(new_vec)
                        .map_err(|_| EngraveError::BeatMapInsertError)?;
                }
            }

            beat_tracker += duration_value;
        }

        let mut glyphs: Vec<(u32, Vec<GlyphDefinition, 16>), 4> = Vec::new();

        for (idx, beat) in beat_map.into_iter() {
            let beat_glyphs = Self::beat_to_glyphs(beat)?;
            glyphs
                .push((idx, beat_glyphs))
                .map_err(|_| EngraveError::NotEnoughSpaceForSymbols)?;
        }

        glyphs.sort_unstable_by_key(|(idx, _)| *idx);

        let mut glyphs: Vec<GlyphDefinition, 16> =
            glyphs.into_iter().flat_map(|(_, v)| v).collect();

        for glyph in glyphs.iter_mut() {
            Self::fix_stems(glyph);
        }

        Ok(Self { glyphs })
    }

    fn beat_to_glyphs(
        beat: Vec<SpacedMusicSymbol, 4>,
    ) -> Result<Vec<GlyphDefinition, 16>, EngraveError> {
        let mut glyphs = Vec::new();

        let mut glyph_symbols: Vec<SpacedMusicSymbol, 4> = Vec::new();
        for symbol in beat.into_iter() {
            let glyph_is_notes =
                !glyph_symbols.is_empty() && !glyph_symbols.first().unwrap().symbol.rest;

            let symbol_is_note = !symbol.symbol.rest;

            if glyph_is_notes && symbol_is_note {
                // if we're doing notes and this is a note, add it to the symbols and continue
                glyph_symbols
                    .push(symbol)
                    .map_err(|_| EngraveError::NotEnoughSpaceForSymbols)?;
            } else if glyph_is_notes && !symbol_is_note {
                // if we're doing notes and this is a rest, add a glyph and clear glyph_symbols, then add the rest glyph
                // Manual drain implementation
                let mut moved_symbols = Vec::new();
                while !glyph_symbols.is_empty() {
                    moved_symbols
                        .push(glyph_symbols.remove(0))
                        .map_err(|_| EngraveError::Impossible)?;
                }
                // Push the previous glyph
                let beamed = moved_symbols.len() > 1;
                glyphs
                    .push(GlyphDefinition {
                        symbols: moved_symbols,
                        beamed,
                    })
                    .map_err(|_| EngraveError::NotEnoughSpaceForSymbols)?;
                // Push a rest glyph
                let mut rest_symbol = Vec::new();
                rest_symbol
                    .push(symbol)
                    .map_err(|_| EngraveError::NotEnoughSpaceForSymbols)?;
                glyphs
                    .push(GlyphDefinition {
                        symbols: rest_symbol,
                        beamed: false,
                    })
                    .map_err(|_| EngraveError::NotEnoughSpaceForSymbols)?;
            } else if glyph_symbols.is_empty() {
                if symbol.symbol.rest {
                    let mut rest_symbol = Vec::new();
                    rest_symbol
                        .push(symbol)
                        .map_err(|_| EngraveError::NotEnoughSpaceForSymbols)?;
                    glyphs
                        .push(GlyphDefinition {
                            symbols: rest_symbol,
                            beamed: false,
                        })
                        .map_err(|_| EngraveError::NotEnoughSpaceForSymbols)?;
                } else {
                    glyph_symbols
                        .push(symbol)
                        .map_err(|_| EngraveError::NotEnoughSpaceForSymbols)?;
                }
            } else {
                debug!(
                    "{} {} {} {} {:?}",
                    glyph_is_notes,
                    symbol_is_note,
                    !glyph_symbols.is_empty(),
                    !glyph_symbols.first().unwrap().symbol.rest,
                    &glyph_symbols
                );

                panic!("Forgot a case");
            }
        }

        if !glyph_symbols.is_empty() {
            let beamed = glyph_symbols.len() > 1;
            glyphs
                .push(GlyphDefinition {
                    symbols: glyph_symbols,
                    beamed,
                })
                .map_err(|_| EngraveError::NotEnoughSpaceForSymbols)?;
        }

        Ok(glyphs)
    }

    // TODO: unwraps in this function...
    fn fix_stems(glyph: &mut GlyphDefinition) -> &mut GlyphDefinition {
        if glyph.symbols.len() == 1 {
            return glyph;
        }

        // Set the direction of the stems to the same value for each
        let mut stem_directions: Vec<StemDirection, 4> = Vec::new();
        for symbol in glyph.symbols.iter_mut() {
            stem_directions.push(symbol.symbol.stem_direction).unwrap();
        }

        let ups = stem_directions
            .iter()
            .filter(|&&d| d == StemDirection::Up)
            .count();

        let downs = stem_directions
            .iter()
            .filter(|&&d| d == StemDirection::Down)
            .count();

        let direction = if ups >= downs {
            StemDirection::Up
        } else {
            StemDirection::Down
        };

        for symbol in glyph.symbols.iter_mut() {
            symbol.symbol.stem_direction = direction;
        }

        // Set the stem lengths such that they line up
        let stem_offset = if direction == StemDirection::Up {
            -1
        } else {
            1
        };
        let stem_endpoints: Vec<i32, 4> = glyph
            .symbols
            .iter()
            .map(|s| s.symbol.y + stem_offset * MusicSymbolDefinitions::DEFAULT_STEM_LENGTH)
            .collect();

        let most_extreme_endpoint = if direction == StemDirection::Up {
            stem_endpoints.iter().min().unwrap()
        } else {
            stem_endpoints.iter().max().unwrap()
        };

        for symbol in glyph.symbols.iter_mut() {
            symbol.symbol.stem_length = stem_offset * (most_extreme_endpoint - symbol.symbol.y);
        }

        glyph
    }
}

impl Music {
    /// Draws notes, rests, ties, and other music defining notation.
    /// Assumes a beat consists of a quarter note.
    /// x Makes a collection of noteheads that should be rendered
    /// x Calculates the free space that nodeheads can maximally move to the left and right
    ///     x This is the range that the notehead can exist in
    /// x Calculates the Y drawing position of the notehead (relative to the given position)
    /// x Assert that all ranges are at least 7 pixels
    /// x Determines which notes should be tied using the following criteria:
    ///     x The notes must start in the same beat (assumes 4/4, and that vec start = bar start)
    ///     x The notes are eighth, dotted eigtht, or sixteenth duration
    ///     x The notes are consecutive
    ///     x Does NOT take tie-ing into account, any group of notes that follow these criteria are beamed
    /// x This results in a list of glyphs, each containing a vec of symbols
    ///     with their available space. If the Vec contains more than one note, it is beamed
    /// - Actually performs the drawing of the glyps, here finally the position argument is applied to everything:
    ///     - For each notehead, compute the necessary space:
    ///         - whole, half, and quarter notes don't need space for flags
    ///         - Beamed notes may need some space dependending on their position and orientation
    ///     - Move noteheads around to provide necessary space
    ///     - Notes without beams can be drawn trivially using their symbol (with stem in correct direction)
    ///     - Beamed notes need 4 drawing steps
    ///         - 1) Draw the noteheads
    ///         - Determine the direction of all stems (follow majority)
    ///         - Determine the y position of the beam (all beams are strictly horizontal)
    ///         - 2) Draw all stems starting at the notehead up (down) to the computed y
    ///         - 3) Draw a horizontal line for the beam
    ///         - 4) Draw additional horizontal lines for sixteenths
    ///     - Rests are drawn using their symbol at a specific hard coded y position
    /// TODO: ledger lines for high / low notes
    /// TODO: interactive rhythm definition is simulatable using button events and showing toggle switch state in console (or on screen)
    pub fn draw<D>(
        target: &mut D,
        position: Point,
        width: i32,
        music: &[Music],
    ) -> Result<u32, D::Error>
    where
        D: DrawTarget<Color = BinaryColor>,
    {
        // TODO: fix all the unwraps by drawing different things
        let symbols = MusicSymbolDefinitions::new(music)
            .and_then(|symbols| SpacedMusicSymbols::new(symbols, width))
            .and_then(Glyphs::new);

        if symbols.is_err() {
            error!("Error engraving symbols: {:?}", symbols);
        }

        let mut glyphs = symbols.unwrap();

        let mut x = 0;
        for glyph in glyphs.glyphs.iter_mut() {
            let glyph_start_x = x;

            for symbol in glyph.symbols.iter_mut() {
                Self::draw_spaced_music_symbol(target, position, x, symbol, glyph.beamed)?;
                symbol.symbol.x = Some(x);

                x += symbol.space;
            }

            if glyph.beamed {
                Self::draw_beams(target, position, glyph_start_x, glyph)?;
            }
        }

        // Draw ties and other things that might need x info
        let mut last_symbol: Option<&SpacedMusicSymbol> = None;
        for glyph in glyphs.glyphs.iter_mut() {
            for symbol in glyph.symbols.iter() {
                let tied = if let Some(symbol) = last_symbol {
                    symbol.symbol.tied
                } else {
                    false
                };

                if tied {
                    // Unwrap is safe because tied can only be true if last symbol is some
                    Self::draw_tie(target, position, last_symbol.unwrap(), symbol)?;
                }

                last_symbol = Some(symbol)
            }
        }

        Ok(0)
    }

    fn draw_spaced_music_symbol<D>(
        target: &mut D,
        position: Point,
        x: i32,
        symbol: &SpacedMusicSymbol,
        beamed: bool,
    ) -> Result<(), D::Error>
    where
        D: DrawTarget<Color = BinaryColor>,
    {
        if symbol.symbol.rest {
            let pos = position + Point::new(x, MusicSymbolDefinitions::REST_OFFSET);
            match symbol.symbol.kind {
                Duration::Eighth => crate::symbols::draw_symbol(target, pos, EIGHTH_REST)?,
                Duration::Whole => crate::symbols::draw_symbol(target, pos, WHOLE_REST)?,
                Duration::Half => crate::symbols::draw_symbol(target, pos, HALF_REST)?,
                Duration::DottedHalf => crate::symbols::draw_symbol(target, pos, DOTTED_HALF_REST)?,
                Duration::Quarter => crate::symbols::draw_symbol(target, pos, QUARTER_REST)?,
                Duration::DottedQuarter => {
                    crate::symbols::draw_symbol(target, pos, DOTTED_QUARTER_REST)?
                }
                Duration::DottedEighth => {
                    crate::symbols::draw_symbol(target, pos, DOTTED_EIGHTH_REST)?
                }
                Duration::Sixteenth => crate::symbols::draw_symbol(target, pos, SIXTEENTH_REST)?,
            };
        } else {
            let position = position + Point::new(x, symbol.symbol.y);
            let line_style = PrimitiveStyle::with_stroke(BinaryColor::On, 1);
            let bg_style = PrimitiveStyle::with_stroke(BinaryColor::Off, 1);

            // If necessary, draw ledger lines
            let top_ledger_y = Staff::LEDGER_MARGIN;
            let bottom_ledger_y = Staff::LEDGER_MARGIN + Staff::LINE_SPACING * 4; // or 6?

            if symbol.symbol.y < top_ledger_y {
                let amount_of_ledgers_necessary =
                    (top_ledger_y - symbol.symbol.y) / Staff::LINE_SPACING;

                let mut y = top_ledger_y - Staff::LINE_SPACING;

                for _ in 0..amount_of_ledgers_necessary {
                    let start = Point::new(position.x - 1, y);
                    let end = Point::new(position.x + 5, y);
                    Line::new(start, end).draw_styled(&line_style, target)?;

                    y -= Staff::LINE_SPACING;
                }
            } else if symbol.symbol.y > bottom_ledger_y {
                let amount_of_ledgers_necessary =
                    (symbol.symbol.y - bottom_ledger_y) / Staff::LINE_SPACING + 1;

                let mut y = bottom_ledger_y + Staff::LINE_SPACING;

                for _ in 0..amount_of_ledgers_necessary {
                    let start = Point::new(position.x - 1, y);
                    let end = Point::new(position.x + 6, y);
                    Line::new(start, end).draw_styled(&line_style, target)?;

                    y += Staff::LINE_SPACING;
                }
            }

            // Draw the head
            match symbol.symbol.kind {
                Duration::Whole | Duration::Half | Duration::DottedHalf => {
                    crate::symbols::draw_symbol(target, position, EMPTY_NOTEHEAD)?
                }
                _ => crate::symbols::draw_symbol(target, position, FILLED_NOTEHEAD)?,
            };

            if symbol.symbol.kind != Duration::Whole {
                // Draw stem
                let (start_pos, end_pos) = if symbol.symbol.stem_direction == StemDirection::Up {
                    (
                        position + Point::new(4, 0),
                        position + Point::new(4, -symbol.symbol.stem_length + 1),
                    )
                } else {
                    (
                        position + Point::new(1, 4),
                        position + Point::new(1, symbol.symbol.stem_length + 3),
                    )
                };

                Line::new(start_pos, end_pos)
                    .into_styled(line_style)
                    .draw(target)?;

                let x_offset = Point::new(1, 0);
                Line::new(start_pos + x_offset, end_pos + x_offset)
                    .into_styled(bg_style)
                    .draw(target)?;

                Line::new(start_pos - x_offset, end_pos - x_offset)
                    .into_styled(bg_style)
                    .draw(target)?;
            }

            // Draw dot, if applicable
            if matches!(
                symbol.symbol.kind,
                Duration::DottedEighth | Duration::DottedQuarter | Duration::DottedHalf
            ) {
                // Draw two dots, one overlaps with a ledger line
                Rectangle::new(
                    position + Point::new(6, 2),
                    Size {
                        width: 1,
                        height: 1,
                    },
                )
                .into_styled(line_style)
                .draw(target)?;

                Rectangle::new(
                    position + Point::new(6, 0),
                    Size {
                        width: 1,
                        height: 1,
                    },
                )
                .into_styled(line_style)
                .draw(target)?;
            }

            if !beamed {
                // Draw flag
                let flipped = symbol.symbol.stem_direction == StemDirection::Down;

                let offset = if !flipped {
                    Point {
                        x: 6,
                        y: -MusicSymbolDefinitions::DEFAULT_STEM_LENGTH + 1,
                    }
                } else {
                    Point {
                        x: 0,
                        y: MusicSymbolDefinitions::DEFAULT_STEM_LENGTH,
                    }
                };

                let flip_offset = Point {
                    x: if flipped { -1 } else { 0 },
                    y: if flipped { -1 } else { 0 },
                };

                let pos = position + offset + flip_offset;

                match symbol.symbol.kind {
                    Duration::Eighth => {
                        crate::symbols::draw_symbol_with_direction(
                            target, pos, EIGHT_FLAG, flipped,
                        )?;
                    }
                    Duration::DottedEighth => {
                        crate::symbols::draw_symbol_with_direction(
                            target,
                            pos,
                            DOTTED_EIGHTH_REST,
                            flipped,
                        )?;
                    }
                    Duration::Sixteenth => {
                        crate::symbols::draw_symbol_with_direction(
                            target,
                            pos + Point {
                                x: 0,
                                y: if flipped { -2 } else { 0 },
                            },
                            SIXTEENTH_FLAG,
                            flipped,
                        )?;
                    }
                    _ => {}
                }
            }
        }

        Ok(())
    }

    fn draw_beams<D>(
        target: &mut D,
        position: Point,
        x: i32,
        glyph: &GlyphDefinition,
    ) -> Result<(), D::Error>
    where
        D: DrawTarget<Color = BinaryColor>,
    {
        if glyph.symbols.is_empty() {
            // TODO: draw a failure
            error!("Empty glyph.");
            return Ok(());
        }

        let style = PrimitiveStyle::with_stroke(BinaryColor::On, 1);
        let off_style = PrimitiveStyle::with_stroke(BinaryColor::Off, 1);

        let first_symbol = glyph.symbols.first().unwrap().symbol;
        let last_symbol = glyph.symbols.last().unwrap().symbol;

        let flipped = first_symbol.stem_direction == StemDirection::Down;

        // always draw top beam
        let top_beam_start = position
            + if flipped {
                Point::new(x + 1, first_symbol.y + first_symbol.stem_length + 2)
            } else {
                Point::new(x + 4, first_symbol.y - first_symbol.stem_length + 1)
            };

        let beam_length = first_symbol
            .x
            .and_then(|x_first| last_symbol.x.map(|x_last| x_last - x_first));

        let beam_length = if let Some(length) = beam_length {
            length
        } else {
            error!(
                "Unknown beam length: x first = {:?}, x last = {:?}",
                first_symbol.x, last_symbol.x
            );
            return Ok(());
        };

        Rectangle::new(
            top_beam_start,
            Size {
                width: beam_length as u32,
                height: 2,
            },
        )
        .draw_styled(&style, target)?;

        // Draw black lines to obscure barlines
        for pair in glyph.symbols.windows(2) {
            if let [first, second] = pair {
                let x_first = first.symbol.x.unwrap_or_default();
                let x_second = second.symbol.x.unwrap_or_default();
                let beam_length = x_second - x_first - 1;

                Rectangle::new(
                    Point {
                        x: position.x + x_first + if flipped { 2 } else { 5 },
                        y: top_beam_start.y + if flipped { -1 } else { 2 },
                    },
                    Size {
                        width: beam_length as u32,
                        height: 1,
                    },
                )
                .draw_styled(&off_style, target)?;
            } else {
                panic!("Unexpected window function return")
            }
        }

        // for each consecutive note pair
        // if sixteenths - eight: draw block facing right
        // if eight - sixteenth: draw block facing left
        // if sixteenth - sixteenht: connect
        for pair in glyph.symbols.windows(2) {
            if let [first, second] = pair {
                let x_first = first.symbol.x.unwrap_or_default();
                let x_second = second.symbol.x.unwrap_or_default();
                let beam_length = x_second - x_first - 1;

                let disconnected_beam_length = 3u32;

                // For 2 cases the previous algorithm produces an artifact, prevent that
                let mut is_ess_or_sse = false;
                if glyph.symbols.len() == 3 {
                    if glyph.symbols.iter().all(|s| !s.symbol.rest) {
                        let durations: Vec<Duration, 4> =
                            glyph.symbols.iter().map(|s| s.symbol.kind).collect();

                        if durations[0] == Duration::Eighth
                            && durations[1] == Duration::Sixteenth
                            && durations[2] == Duration::Sixteenth
                        {
                            is_ess_or_sse = true;
                        }

                        if durations[0] == Duration::Sixteenth
                            && durations[1] == Duration::Sixteenth
                            && durations[2] == Duration::Eighth
                        {
                            is_ess_or_sse = true;
                        }
                    }
                }

                match (first.symbol.kind, second.symbol.kind) {
                    (Duration::Eighth, Duration::Eighth) => (),
                    (Duration::Eighth, Duration::Sixteenth) => {
                        if !is_ess_or_sse {
                            Rectangle::new(
                                Point {
                                    x: position.x + x_first + beam_length
                                        - disconnected_beam_length as i32
                                        + if flipped { 2 } else { 5 },
                                    y: top_beam_start.y + if flipped { -2 } else { 3 },
                                },
                                Size {
                                    width: disconnected_beam_length,
                                    height: 1,
                                },
                            )
                            .draw_styled(&style, target)?;
                        }
                    }
                    (Duration::DottedEighth, Duration::Sixteenth) => {
                        Rectangle::new(
                            Point {
                                x: position.x + x_first + beam_length
                                    - disconnected_beam_length as i32
                                    + if flipped { 2 } else { 5 },
                                y: top_beam_start.y + if flipped { -2 } else { 3 },
                            },
                            Size {
                                width: disconnected_beam_length,
                                height: 1,
                            },
                        )
                        .draw_styled(&style, target)?;
                    }
                    (Duration::Sixteenth, Duration::Eighth) => {
                        if !is_ess_or_sse {
                            Rectangle::new(
                                Point {
                                    x: position.x + x_first + if flipped { 2 } else { 5 },
                                    y: top_beam_start.y + if flipped { -2 } else { 3 },
                                },
                                Size {
                                    width: disconnected_beam_length,
                                    height: 1,
                                },
                            )
                            .draw_styled(&style, target)?;
                        }
                    }
                    (Duration::Sixteenth, Duration::DottedEighth) => {
                        Rectangle::new(
                            Point {
                                x: position.x + x_first + if flipped { 2 } else { 5 },
                                y: top_beam_start.y + if flipped { -2 } else { 3 },
                            },
                            Size {
                                width: disconnected_beam_length,
                                height: 1,
                            },
                        )
                        .draw_styled(&style, target)?;
                    }
                    (Duration::Sixteenth, Duration::Sixteenth) => {
                        Rectangle::new(
                            Point {
                                x: position.x + x_first + if flipped { 2 } else { 5 },
                                y: top_beam_start.y + if flipped { -2 } else { 3 },
                            },
                            Size {
                                width: beam_length as u32,
                                height: 1,
                            },
                        )
                        .draw_styled(&style, target)?;
                    }
                    _ => panic!("Unknown beaming"),
                }

                Rectangle::new(
                    Point {
                        x: position.x + x_first + if flipped { 2 } else { 5 },
                        y: top_beam_start.y + if flipped { -3 } else { 4 },
                    },
                    Size {
                        width: beam_length as u32,
                        height: 1,
                    },
                )
                .draw_styled(&off_style, target)?;
            } else {
                panic!("Unexpected window function return")
            }
        }

        Ok(())
    }

    fn draw_tie<D>(
        target: &mut D,
        position: Point,
        first: &SpacedMusicSymbol,
        second: &SpacedMusicSymbol,
    ) -> Result<(), D::Error>
    where
        D: DrawTarget<Color = BinaryColor>,
    {
        let style = PrimitiveStyle::with_stroke(BinaryColor::On, 1);
        let off_style = PrimitiveStyle::with_fill(BinaryColor::Off);

        let first_x = first.symbol.x.unwrap_or(0);
        let second_x = second.symbol.x.unwrap_or(0);
        let tie_length = second_x - first_x;
        let flipped = first.symbol.stem_direction == StemDirection::Down;

        Rectangle::new(
            Point {
                x: position.x + first.symbol.x.unwrap_or(0) - 1 + if flipped { 3 } else { 3 },
                y: position.y + first.symbol.y - 2 + if flipped { -3 } else { 7 },
            },
            Size {
                width: tie_length as u32 + 2,
                height: 4,
            },
        )
        .draw_styled(&off_style, target)?;

        // left point of the tie
        Rectangle::new(
            Point {
                x: position.x + first.symbol.x.unwrap_or(0) + if flipped { 4 } else { 4 },
                y: position.y + first.symbol.y + if flipped { -2 } else { 6 },
            },
            Size {
                width: 1,
                height: 1,
            },
        )
        .draw_styled(&style, target)?;

        // right point of the tie
        Rectangle::new(
            Point {
                x: position.x + second.symbol.x.unwrap_or(0) + if flipped { 1 } else { 1 },
                y: position.y + first.symbol.y + if flipped { -2 } else { 6 },
            },
            Size {
                width: 1,
                height: 1,
            },
        )
        .draw_styled(&style, target)?;

        // straight part of the tie
        Rectangle::new(
            Point {
                x: position.x + first.symbol.x.unwrap_or(0) + 1 + if flipped { 4 } else { 4 },
                y: position.y + first.symbol.y + if flipped { -3 } else { 7 },
            },
            Size {
                width: tie_length as u32 - 4,
                height: 1,
            },
        )
        .draw_styled(&style, target)?;

        Ok(())
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Duration {
    Whole,
    Half,
    DottedHalf,
    Quarter,
    DottedQuarter,
    Eighth,
    DottedEighth,
    Sixteenth,
}

impl Duration {
    pub fn value(self) -> f64 {
        match self {
            Duration::Whole => 4.0,
            Duration::Half => 2.0,
            Duration::DottedHalf => 3.0,
            Duration::Quarter => 1.0,
            Duration::DottedQuarter => 1.5,
            Duration::Eighth => 0.5,
            Duration::DottedEighth => 0.75,
            Duration::Sixteenth => 0.25,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum Accidental {
    Natural,
    Sharp,
    Flat,
    DoubleSharp,
    DoubleFlat,
}

// TODO: move to a rytmos-common crate?
#[derive(Clone, Copy, Debug)]
pub enum Note {
    A(Accidental, i32),
    B(Accidental, i32),
    C(Accidental, i32),
    D(Accidental, i32),
    E(Accidental, i32),
    F(Accidental, i32),
    G(Accidental, i32),
}

impl Note {
    const C0_OFFSET: i32 = Staff::LEDGER_MARGIN + Staff::LINE_SPACING * (5 + 7) + 2;
    const STEM_DIRECTION_SWITCH_HEIGHT: i32 = Self::C0_OFFSET - 2 - 3 * 14; // At D3, flip

    fn y_offset(self) -> i32 {
        match self {
            Note::A(_, octave) => Self::C0_OFFSET - 10 - octave * 14,
            Note::B(_, octave) => Self::C0_OFFSET - 12 - octave * 14,
            Note::C(_, octave) => Self::C0_OFFSET - octave * 14,
            Note::D(_, octave) => Self::C0_OFFSET - 2 - octave * 14,
            Note::E(_, octave) => Self::C0_OFFSET - 4 - octave * 14,
            Note::F(_, octave) => Self::C0_OFFSET - 6 - octave * 14,
            Note::G(_, octave) => Self::C0_OFFSET - 8 - octave * 14,
        }
    }

    fn default_stem_direction(&self) -> StemDirection {
        if self.y_offset() > Self::STEM_DIRECTION_SWITCH_HEIGHT {
            StemDirection::Up
        } else {
            StemDirection::Down
        }
    }

    pub fn map_octave(&mut self, f: impl Fn(i32) -> i32) -> Self {
        match self {
            Note::A(acc, octave) => Note::A(*acc, f(*octave)),
            Note::B(acc, octave) => Note::B(*acc, f(*octave)),
            Note::C(acc, octave) => Note::C(*acc, f(*octave)),
            Note::D(acc, octave) => Note::D(*acc, f(*octave)),
            Note::E(acc, octave) => Note::E(*acc, f(*octave)),
            Note::F(acc, octave) => Note::F(*acc, f(*octave)),
            Note::G(acc, octave) => Note::G(*acc, f(*octave)),
        }
    }

    pub fn frequency(&self) -> f32 {
        let a4_frequency = 440.0;
        let semitone_ratio = libm::powf(2., 1. / 12.);

        let (base_note_semitones, octave) = match self {
            Note::A(_, octave) => (0, *octave),
            Note::B(_, octave) => (2, *octave),
            Note::C(_, octave) => (-9, *octave),
            Note::D(_, octave) => (-7, *octave),
            Note::E(_, octave) => (-5, *octave),
            Note::F(_, octave) => (-4, *octave),
            Note::G(_, octave) => (-2, *octave),
        };

        let accidental_offset = match self {
            Note::A(acc, _)
            | Note::B(acc, _)
            | Note::C(acc, _)
            | Note::D(acc, _)
            | Note::E(acc, _)
            | Note::F(acc, _)
            | Note::G(acc, _) => match acc {
                Accidental::DoubleFlat => -2,
                Accidental::Flat => -1,
                Accidental::Natural => 0,
                Accidental::Sharp => 1,
                Accidental::DoubleSharp => 2,
            },
        };

        let total_semitones = base_note_semitones + accidental_offset + (octave - 4) * 12;
        a4_frequency * libm::powf(semitone_ratio, total_semitones as f32)
    }
}

#[derive(Clone, Copy)]
pub enum Key {
    CMajor,
    FMajor,
    BbMajor,
    EbMajor,
    AbMajor,
    DbMajor,
    GbMajor,
    FsMajor,
    BMajor,
    EMajor,
    AMajor,
    DMajor,
    GMajor,
}

#[derive(Clone, Copy)]
pub enum Clef {
    Bass,
    Treble,
}

impl Clef {
    pub fn draw<D>(self, target: &mut D, position: Point) -> Result<u32, D::Error>
    where
        D: DrawTarget<Color = BinaryColor>,
    {
        match self {
            Clef::Bass => {
                crate::symbols::draw_symbol(target, position, BASS_CLEF)?;
                Ok(13)
            }
            Clef::Treble => Ok(0),
        }
    }
}

#[derive(Clone, Copy)]
pub struct Staff {
    width: i32,
    position: Point,
}

impl Staff {
    const LINE_SPACING: i32 = 4;
    const LEDGER_MARGIN: i32 = Self::LINE_SPACING * 5;
    const BASS_CLEF_OFFSET: Point = Point {
        x: 0,
        y: Self::LEDGER_MARGIN,
    };

    pub fn new(width: u32, position: Point) -> Self {
        Self {
            width: width as i32,
            position,
        }
    }

    pub fn draw<D>(&self, target: &mut D, elements: &[StaffElement]) -> Result<(), D::Error>
    where
        D: DrawTarget<Color = BinaryColor>,
    {
        let line_style = PrimitiveStyle::with_stroke(BinaryColor::On, 1);

        for i in 0..5 {
            Line::new(
                Point::new(
                    self.position.x,
                    Self::LEDGER_MARGIN + self.position.y + Self::LINE_SPACING * i,
                ),
                Point::new(
                    self.position.x + self.width,
                    Self::LEDGER_MARGIN + self.position.y + Self::LINE_SPACING * i,
                ),
            )
            .into_styled(line_style)
            .draw(target)?;
        }

        let mut working_position = self.position;

        for element in elements {
            let width_used = match element {
                StaffElement::Barline => Self::draw_barline(target, working_position)?,
                StaffElement::KeySignature(_) => todo!(),
                StaffElement::Clef(clef) => {
                    clef.draw(target, working_position + Self::BASS_CLEF_OFFSET)?
                }
                StaffElement::Music(music) => Music::draw(
                    target,
                    working_position,
                    self.width - working_position.x,
                    music,
                )?,
            };

            working_position.x += width_used as i32;
        }

        Ok(())
    }

    fn draw_barline<D>(target: &mut D, working_position: Point) -> Result<u32, D::Error>
    where
        D: DrawTarget<Color = BinaryColor>,
    {
        Rectangle::new(
            Point::new(working_position.x + 1, 12),
            Size {
                width: 1,
                height: 17,
            },
        )
        .draw_styled(&PrimitiveStyle::with_stroke(BinaryColor::On, 1), target)?;

        Ok(3)
    }
}
