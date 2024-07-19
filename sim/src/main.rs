use std::{thread, time::Duration};

use defmt::*;
use defmt_rtt as _;
use embedded_graphics::{
    pixelcolor::BinaryColor,
    prelude::*,
    primitives::{PrimitiveStyle, Rectangle},
};
use embedded_graphics_simulator::{
    BinaryColorTheme, OutputSettingsBuilder, SimulatorDisplay, SimulatorEvent, Window,
};
use env_logger::{Builder, Env};
use log::LevelFilter;
use rytmos::staff::{self, Accidental, Clef, Music, Note, Staff, StaffElement};

fn main() -> Result<(), core::convert::Infallible> {
    let music1 = &[
        StaffElement::Clef(Clef::Bass),
        StaffElement::Music(&[
            Music::Note(
                Note::A(Accidental::Natural, 1),
                staff::Duration::DottedEighth,
            ),
            Music::Note(Note::B(Accidental::Natural, 1), staff::Duration::Sixteenth),
            Music::Rest(staff::Duration::Eighth),
            Music::Note(Note::F(Accidental::Sharp, 2), staff::Duration::Eighth),
            Music::Tie,
            Music::Note(Note::C(Accidental::Sharp, 2), staff::Duration::Eighth),
            Music::Note(Note::D(Accidental::Natural, 2), staff::Duration::Eighth),
            Music::Note(Note::G(Accidental::Natural, 2), staff::Duration::Sixteenth),
            Music::Note(Note::F(Accidental::Sharp, 2), staff::Duration::Sixteenth),
            Music::Note(Note::A(Accidental::Flat, 2), staff::Duration::Sixteenth),
            Music::Note(Note::E(Accidental::Natural, 2), staff::Duration::Sixteenth),
        ]),
    ];

    let all_rests = &[
        StaffElement::Clef(Clef::Bass),
        StaffElement::Music(&[
            Music::Rest(staff::Duration::Whole),
            Music::Rest(staff::Duration::Half),
            Music::Rest(staff::Duration::Quarter),
            Music::Rest(staff::Duration::Eighth),
            Music::Rest(staff::Duration::Sixteenth),
        ]),
    ];

    let sixteenths = &[
        StaffElement::Clef(Clef::Bass),
        StaffElement::Music(&[
            Music::Rest(staff::Duration::Sixteenth),
            Music::Note(Note::C(Accidental::Natural, 2), staff::Duration::Sixteenth),
            Music::Rest(staff::Duration::Sixteenth),
            Music::Note(Note::C(Accidental::Natural, 2), staff::Duration::Sixteenth),
            Music::Rest(staff::Duration::Sixteenth),
            Music::Note(Note::C(Accidental::Natural, 2), staff::Duration::Sixteenth),
            Music::Rest(staff::Duration::Sixteenth),
            Music::Note(Note::C(Accidental::Natural, 2), staff::Duration::Sixteenth),
            Music::Rest(staff::Duration::Sixteenth),
            Music::Note(Note::C(Accidental::Natural, 2), staff::Duration::Sixteenth),
            Music::Rest(staff::Duration::Sixteenth),
            Music::Note(Note::C(Accidental::Natural, 2), staff::Duration::Sixteenth),
            Music::Rest(staff::Duration::Sixteenth),
            Music::Note(Note::C(Accidental::Natural, 2), staff::Duration::Sixteenth),
            Music::Rest(staff::Duration::Sixteenth),
            Music::Note(Note::C(Accidental::Natural, 2), staff::Duration::Sixteenth),
        ]),
    ];

    let dotted_rests = &[
        StaffElement::Clef(Clef::Bass),
        StaffElement::Music(&[
            Music::Rest(staff::Duration::DottedHalf),
            Music::Rest(staff::Duration::DottedQuarter),
            Music::Rest(staff::Duration::DottedEighth),
        ]),
    ];

    // TODO: multimeasure support in staff drawing? or responsibility of user?
    let all_quarters = &[
        StaffElement::Clef(Clef::Bass),
        StaffElement::Music(&[
            Music::Note(Note::D(Accidental::Natural, 2), staff::Duration::Quarter),
            Music::Note(Note::F(Accidental::Natural, 2), staff::Duration::Quarter),
            Music::Note(Note::A(Accidental::Natural, 2), staff::Duration::Quarter),
            Music::Note(Note::C(Accidental::Natural, 2), staff::Duration::Quarter),
        ]),
        StaffElement::Barline,
        StaffElement::Music(&[
            Music::Note(Note::G(Accidental::Natural, 2), staff::Duration::Quarter),
            Music::Note(Note::B(Accidental::Natural, 2), staff::Duration::Quarter),
            Music::Note(Note::D(Accidental::Natural, 2), staff::Duration::Quarter),
            Music::Note(Note::F(Accidental::Natural, 2), staff::Duration::Quarter),
        ]),
        StaffElement::Barline,
        StaffElement::Music(&[
            Music::Note(Note::C(Accidental::Natural, 2), staff::Duration::Quarter),
            Music::Note(Note::E(Accidental::Natural, 2), staff::Duration::Quarter),
            Music::Note(Note::G(Accidental::Natural, 2), staff::Duration::Quarter),
            Music::Note(Note::B(Accidental::Natural, 2), staff::Duration::Quarter),
        ]),
    ];

    let flags = &[
        StaffElement::Clef(Clef::Bass),
        StaffElement::Music(&[
            Music::Note(Note::A(Accidental::Natural, 1), staff::Duration::Eighth),
            Music::Rest(staff::Duration::Eighth),
            Music::Note(Note::A(Accidental::Natural, 2), staff::Duration::Eighth),
            Music::Rest(staff::Duration::Eighth),
            Music::Note(Note::A(Accidental::Natural, 1), staff::Duration::Sixteenth),
            Music::Rest(staff::Duration::Sixteenth),
            Music::Note(Note::A(Accidental::Natural, 2), staff::Duration::Sixteenth),
            Music::Rest(staff::Duration::Sixteenth),
            Music::Rest(staff::Duration::Quarter),
        ]),
    ];

    Builder::from_env(Env::default().default_filter_or(LevelFilter::Trace.to_string())).init();

    info!("Starting sim");
    let display_size = Size::new(128, 64);
    let mut display = SimulatorDisplay::<BinaryColor>::new(display_size);

    let line_style = PrimitiveStyle::with_stroke(BinaryColor::On, 1);

    let output_settings = OutputSettingsBuilder::new()
        .theme(BinaryColorTheme::OledWhite)
        .scale(8)
        .pixel_spacing(1)
        .build();
    let mut window = Window::new("Rytmos", &output_settings);

    // let staff = Staff::new(display_size.width - 8, Point::new(4, 4));
    let staff = Staff::new(display_size.width, Point::new(0, 0));

    let mut i = 500;
    loop {
        Rectangle::new(Point::zero(), display_size)
            .into_styled(PrimitiveStyle::with_fill(BinaryColor::Off))
            .draw(&mut display)?;
        // Circle::new(
        //     Point::new(i % display_size.width as i32, i / display_size.width as i32),
        //     20,
        // )
        // .into_styled(line_style)
        // .draw(&mut display)?;

        // for i in 0..5 {
        //     Line::new(
        //         Point::new(4, 12 + 4 * i),
        //         Point::new(display_size.width as i32 - 7, 12 + 4 * i),
        //     )
        //     .into_styled(line_style)
        //     .draw(&mut display)?;
        // }

        staff.draw(&mut display, flags)?;

        // let raw_image = ImageRaw::<BinaryColor>::new(symbols::WHOLE_NOTE, 6);
        // Image::new(&raw_image, Point::zero()).draw(&mut display)?;

        // let raw_image = ImageRaw::<BinaryColor>::new(symbols::HALF_NOTE, 6);
        // Image::new(&raw_image, Point::new(7, 0)).draw(&mut display)?;

        // let raw_image = ImageRaw::<BinaryColor>::new(symbols::QUARTER_NOTE, 6);
        // Image::new(&raw_image, Point::new(11, 3 + i % 25)).draw(&mut display)?;

        // let raw_image = ImageRaw::<BinaryColor>::new(symbols::QUARTER_REST, 5);
        // Image::new(&raw_image, Point::new(21, 16)).draw(&mut display)?;

        // let raw_image = ImageRaw::<BinaryColor>::new(symbols::EIGHTH_NOTE, 6);
        // Image::new(&raw_image, Point::new(28, 4)).draw(&mut display)?;

        // let raw_image = ImageRaw::<BinaryColor>::new(symbols::SIXTEENTH_NOTE, 7);
        // Image::new(&raw_image, Point::new(35, 8)).draw(&mut display)?;

        // let raw_image = ImageRaw::<BinaryColor>::new(symbols::WHOLE_REST, 6);
        // Image::new(&raw_image, Point::new(0, 9)).draw(&mut display)?;

        // let raw_image = ImageRaw::<BinaryColor>::new(symbols::HALF_REST, 6);
        // Image::new(&raw_image, Point::new(35, 8)).draw(&mut display)?;

        // let raw_image = ImageRaw::<BinaryColor>::new(symbols::EIGHTH_REST, 6);
        // Image::new(&raw_image, Point::new(42, 16)).draw(&mut display)?;

        // let raw_image = ImageRaw::<BinaryColor>::new(symbols::SIXTEENTH_REST, 7);
        // Image::new(&raw_image, Point::new(51, 16)).draw(&mut display)?;

        i += 1;

        window.update(&display);

        if window.events().any(|e| e == SimulatorEvent::Quit) {
            break;
        }
        thread::sleep(Duration::from_millis(250));
    }

    Ok(())
}
