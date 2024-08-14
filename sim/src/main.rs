use std::{thread, time::Duration};

use defmt_rtt as _;
use embedded_graphics::{
    pixelcolor::BinaryColor,
    prelude::*,
    primitives::{PrimitiveStyle, Rectangle},
};
use embedded_graphics_simulator::{
    sdl2::Keycode, BinaryColorTheme, OutputSettingsBuilder, SimulatorDisplay, SimulatorEvent,
    Window,
};
use env_logger::{Builder, Env};
use log::LevelFilter;
use rytmos::staff::{self, Accidental, Clef, Music, Note, Staff, StaffElement};

fn main() -> Result<(), core::convert::Infallible> {
    let music1 = vec![
        StaffElement::Clef(Clef::Bass),
        StaffElement::Music(&[
            Music::Note(
                Note::A(Accidental::Natural, 2),
                staff::Duration::DottedEighth,
            ),
            Music::Note(Note::B(Accidental::Natural, 2), staff::Duration::Sixteenth),
            Music::Rest(staff::Duration::Eighth),
            Music::Note(Note::F(Accidental::Sharp, 3), staff::Duration::Eighth),
            Music::Tie,
            Music::Note(Note::F(Accidental::Sharp, 3), staff::Duration::Eighth),
            Music::Note(Note::D(Accidental::Natural, 3), staff::Duration::Eighth),
            Music::Note(Note::G(Accidental::Natural, 3), staff::Duration::Sixteenth),
            Music::Note(Note::F(Accidental::Sharp, 3), staff::Duration::Sixteenth),
            Music::Note(Note::A(Accidental::Flat, 3), staff::Duration::Sixteenth),
            Music::Note(Note::E(Accidental::Natural, 3), staff::Duration::Sixteenth),
        ]),
    ];

    let all_rests = vec![
        StaffElement::Clef(Clef::Bass),
        StaffElement::Music(&[
            Music::Rest(staff::Duration::Whole),
            Music::Rest(staff::Duration::Half),
            Music::Rest(staff::Duration::Quarter),
            Music::Rest(staff::Duration::Eighth),
            Music::Rest(staff::Duration::Sixteenth),
        ]),
    ];

    let sixteenths = vec![
        StaffElement::Clef(Clef::Bass),
        StaffElement::Music(&[
            Music::Rest(staff::Duration::Sixteenth),
            Music::Note(Note::C(Accidental::Natural, 3), staff::Duration::Sixteenth),
            Music::Rest(staff::Duration::Sixteenth),
            Music::Note(Note::C(Accidental::Natural, 3), staff::Duration::Sixteenth),
            Music::Rest(staff::Duration::Sixteenth),
            Music::Note(Note::C(Accidental::Natural, 3), staff::Duration::Sixteenth),
            Music::Rest(staff::Duration::Sixteenth),
            Music::Note(Note::C(Accidental::Natural, 3), staff::Duration::Sixteenth),
            Music::Rest(staff::Duration::Sixteenth),
            Music::Note(Note::C(Accidental::Natural, 3), staff::Duration::Sixteenth),
            Music::Rest(staff::Duration::Sixteenth),
            Music::Note(Note::C(Accidental::Natural, 3), staff::Duration::Sixteenth),
            Music::Rest(staff::Duration::Sixteenth),
            Music::Note(Note::C(Accidental::Natural, 3), staff::Duration::Sixteenth),
            Music::Rest(staff::Duration::Sixteenth),
            Music::Note(Note::C(Accidental::Natural, 3), staff::Duration::Sixteenth),
        ]),
    ];

    let beamed_sixteenths_and_eighths = vec![
        StaffElement::Clef(Clef::Bass),
        StaffElement::Music(&[
            Music::Note(Note::A(Accidental::Natural, 2), staff::Duration::Sixteenth),
            Music::Note(Note::A(Accidental::Natural, 2), staff::Duration::Sixteenth),
            Music::Note(Note::A(Accidental::Natural, 2), staff::Duration::Sixteenth),
            Music::Note(Note::A(Accidental::Natural, 2), staff::Duration::Sixteenth),
            Music::Note(Note::A(Accidental::Natural, 3), staff::Duration::Sixteenth),
            Music::Note(Note::A(Accidental::Natural, 3), staff::Duration::Sixteenth),
            Music::Note(Note::A(Accidental::Natural, 3), staff::Duration::Sixteenth),
            Music::Note(Note::A(Accidental::Natural, 3), staff::Duration::Sixteenth),
            Music::Note(Note::B(Accidental::Natural, 2), staff::Duration::Sixteenth),
            Music::Note(Note::B(Accidental::Natural, 2), staff::Duration::Sixteenth),
            Music::Note(Note::B(Accidental::Natural, 2), staff::Duration::Sixteenth),
            Music::Note(Note::B(Accidental::Natural, 2), staff::Duration::Sixteenth),
            Music::Note(Note::B(Accidental::Natural, 3), staff::Duration::Sixteenth),
            Music::Note(Note::B(Accidental::Natural, 3), staff::Duration::Sixteenth),
            Music::Note(Note::B(Accidental::Natural, 3), staff::Duration::Sixteenth),
            Music::Note(Note::B(Accidental::Natural, 3), staff::Duration::Sixteenth),
        ]),
    ];

    let disconnected_beams = vec![
        StaffElement::Clef(Clef::Bass),
        StaffElement::Music(&[
            // 1 e + a
            Music::Note(Note::A(Accidental::Natural, 2), staff::Duration::Sixteenth),
            Music::Note(Note::A(Accidental::Natural, 2), staff::Duration::Sixteenth),
            Music::Note(Note::A(Accidental::Natural, 2), staff::Duration::Eighth),
            // 2 e + a
            Music::Note(Note::A(Accidental::Natural, 2), staff::Duration::Sixteenth),
            Music::Note(Note::A(Accidental::Natural, 2), staff::Duration::Eighth),
            Music::Note(Note::A(Accidental::Natural, 2), staff::Duration::Sixteenth),
            // 3 e + a
            Music::Note(Note::A(Accidental::Natural, 2), staff::Duration::Eighth),
            Music::Note(Note::A(Accidental::Natural, 2), staff::Duration::Sixteenth),
            Music::Note(Note::A(Accidental::Natural, 2), staff::Duration::Sixteenth),
            // 4 e + a
            Music::Note(Note::A(Accidental::Natural, 2), staff::Duration::Sixteenth),
            Music::Note(
                Note::A(Accidental::Natural, 2),
                staff::Duration::DottedEighth,
            ),
        ]),
    ];

    let ties = vec![
        StaffElement::Clef(Clef::Bass),
        StaffElement::Music(&[
            // 1 e + a
            Music::Note(Note::C(Accidental::Natural, 3), staff::Duration::Sixteenth),
            Music::Note(Note::B(Accidental::Natural, 2), staff::Duration::Sixteenth),
            Music::Note(Note::B(Accidental::Natural, 2), staff::Duration::Eighth),
            Music::Tie,
            // 2 e + a
            Music::Note(Note::B(Accidental::Natural, 2), staff::Duration::Sixteenth),
            Music::Note(Note::C(Accidental::Natural, 3), staff::Duration::Eighth),
            Music::Note(Note::E(Accidental::Natural, 3), staff::Duration::Sixteenth),
            Music::Tie,
            // 3 e + a
            Music::Note(Note::E(Accidental::Natural, 3), staff::Duration::Eighth),
            Music::Note(Note::D(Accidental::Natural, 3), staff::Duration::Sixteenth),
            Music::Note(Note::F(Accidental::Natural, 3), staff::Duration::Sixteenth),
            Music::Tie,
            // 4 e + a
            Music::Note(Note::F(Accidental::Natural, 3), staff::Duration::Sixteenth),
            Music::Note(
                Note::A(Accidental::Natural, 3),
                staff::Duration::DottedEighth,
            ),
        ]),
    ];

    let dotted_rests = vec![
        StaffElement::Clef(Clef::Bass),
        StaffElement::Music(&[
            Music::Rest(staff::Duration::DottedHalf),
            Music::Rest(staff::Duration::DottedQuarter),
            Music::Rest(staff::Duration::DottedEighth),
        ]),
    ];

    let flags = vec![
        StaffElement::Clef(Clef::Bass),
        StaffElement::Music(&[
            Music::Note(Note::A(Accidental::Natural, 2), staff::Duration::Eighth),
            Music::Rest(staff::Duration::Eighth),
            Music::Note(Note::A(Accidental::Natural, 3), staff::Duration::Eighth),
            Music::Rest(staff::Duration::Eighth),
            Music::Note(Note::A(Accidental::Natural, 2), staff::Duration::Sixteenth),
            Music::Rest(staff::Duration::Sixteenth),
            Music::Note(Note::A(Accidental::Natural, 3), staff::Duration::Sixteenth),
            Music::Rest(staff::Duration::Sixteenth),
            Music::Rest(staff::Duration::Quarter),
        ]),
    ];

    // TODO: all this is an octave transposed (fix now, then make configurable once we synth)
    let ledgers = vec![
        StaffElement::Clef(Clef::Bass),
        StaffElement::Music(&[
            Music::Note(Note::E(Accidental::Natural, 2), staff::Duration::Sixteenth),
            Music::Note(Note::D(Accidental::Natural, 2), staff::Duration::Sixteenth),
            Music::Note(Note::C(Accidental::Natural, 2), staff::Duration::Sixteenth),
            Music::Rest(staff::Duration::Sixteenth),
            Music::Note(Note::A(Accidental::Natural, 1), staff::Duration::Sixteenth),
            Music::Note(Note::G(Accidental::Natural, 1), staff::Duration::Sixteenth),
            Music::Note(Note::F(Accidental::Natural, 1), staff::Duration::Sixteenth),
            Music::Rest(staff::Duration::Sixteenth),
            Music::Note(Note::C(Accidental::Natural, 4), staff::Duration::Sixteenth),
            Music::Note(Note::D(Accidental::Natural, 4), staff::Duration::Sixteenth),
            Music::Note(Note::E(Accidental::Natural, 4), staff::Duration::Sixteenth),
            Music::Rest(staff::Duration::Sixteenth),
            Music::Note(Note::G(Accidental::Natural, 4), staff::Duration::Sixteenth),
            Music::Note(Note::A(Accidental::Natural, 4), staff::Duration::Sixteenth),
            Music::Rest(staff::Duration::Sixteenth),
            Music::Note(Note::C(Accidental::Natural, 5), staff::Duration::Sixteenth),
        ]),
    ];

    let treble = vec![
        StaffElement::Clef(Clef::Treble),
        StaffElement::Music(&[Music::Note(
            Note::A(Accidental::Natural, 1),
            staff::Duration::Eighth,
        )]),
    ];

    let examples = &[
        ledgers,
        treble,
        ties,
        disconnected_beams,
        beamed_sixteenths_and_eighths,
        music1,
        all_rests,
        sixteenths,
        dotted_rests,
        // all_quarters,
        flags,
    ];

    Builder::from_env(Env::default().default_filter_or(LevelFilter::Trace.to_string())).init();

    let display_size = Size::new(128, 64);
    let mut display = SimulatorDisplay::<BinaryColor>::new(display_size);

    let output_settings = OutputSettingsBuilder::new()
        .theme(BinaryColorTheme::OledWhite)
        .scale(8)
        .pixel_spacing(1)
        .build();
    let mut window = Window::new("Rytmos", &output_settings);

    // let staff = Staff::new(display_size.width - 8, Point::new(4, 4));
    let staff = Staff::new(display_size.width, Point::new(0, 0));

    let mut example_idx = 0;

    'main: loop {
        Rectangle::new(Point::zero(), display_size)
            .into_styled(PrimitiveStyle::with_fill(BinaryColor::Off))
            .draw(&mut display)?;

        staff.draw(&mut display, &examples[example_idx])?;

        window.update(&display);

        for event in window.events() {
            match event {
                SimulatorEvent::KeyUp {
                    keycode,
                    keymod: _,
                    repeat,
                } => match (keycode, repeat) {
                    (Keycode::Space, false) => {
                        example_idx = (example_idx + 1) % examples.len();
                    }
                    _ => (),
                },
                SimulatorEvent::Quit => break 'main,
                _ => (),
            }
        }

        thread::sleep(Duration::from_millis(25));
    }

    Ok(())
}
