#[macro_export]
macro_rules! a {
    ($octave:expr) => {
        rytmos_engrave::staff::Note::A(rytmos_engrave::staff::Accidental::Natural, $octave)
    };
}

#[macro_export]
macro_rules! aes {
    ($octave:expr) => {
        rytmos_engrave::staff::Note::A(rytmos_engrave::staff::Accidental::Flat, $octave)
    };
}

#[macro_export]
macro_rules! ais {
    ($octave:expr) => {
        rytmos_engrave::staff::Note::A(rytmos_engrave::staff::Accidental::Sharp, $octave)
    };
}

#[macro_export]
macro_rules! b {
    ($octave:expr) => {
        rytmos_engrave::staff::Note::B(rytmos_engrave::staff::Accidental::Natural, $octave)
    };
}

#[macro_export]
macro_rules! bes {
    ($octave:expr) => {
        rytmos_engrave::staff::Note::B(rytmos_engrave::staff::Accidental::Flat, $octave)
    };
}

#[macro_export]
macro_rules! c {
    ($octave:expr) => {
        rytmos_engrave::staff::Note::C(rytmos_engrave::staff::Accidental::Natural, $octave)
    };
}

#[macro_export]
macro_rules! cis {
    ($octave:expr) => {
        rytmos_engrave::staff::Note::C(rytmos_engrave::staff::Accidental::Sharp, $octave)
    };
}

#[macro_export]
macro_rules! d {
    ($octave:expr) => {
        rytmos_engrave::staff::Note::D(rytmos_engrave::staff::Accidental::Natural, $octave)
    };
}

#[macro_export]
macro_rules! dis {
    ($octave:expr) => {
        rytmos_engrave::staff::Note::D(rytmos_engrave::staff::Accidental::Sharp, $octave)
    };
}

#[macro_export]
macro_rules! es {
    ($octave:expr) => {
        rytmos_engrave::staff::Note::E(rytmos_engrave::staff::Accidental::Flat, $octave)
    };
}

#[macro_export]
macro_rules! e {
    ($octave:expr) => {
        rytmos_engrave::staff::Note::E(rytmos_engrave::staff::Accidental::Natural, $octave)
    };
}

#[macro_export]
macro_rules! f {
    ($octave:expr) => {
        rytmos_engrave::staff::Note::F(rytmos_engrave::staff::Accidental::Natural, $octave)
    };
}

#[macro_export]
macro_rules! fis {
    ($octave:expr) => {
        rytmos_engrave::staff::Note::F(rytmos_engrave::staff::Accidental::Sharp, $octave)
    };
}

#[macro_export]
macro_rules! g {
    ($octave:expr) => {
        rytmos_engrave::staff::Note::G(rytmos_engrave::staff::Accidental::Natural, $octave)
    };
}

#[macro_export]
macro_rules! gis {
    ($octave:expr) => {
        rytmos_engrave::staff::Note::G(rytmos_engrave::staff::Accidental::Sharp, $octave)
    };
}

#[macro_export]
macro_rules! ges {
    ($octave:expr) => {
        rytmos_engrave::staff::Note::G(rytmos_engrave::staff::Accidental::Flat, $octave)
    };
}
