// All lengths in mm
pub const FRET_POSITIONS: &[f64] = &[48.436, 94.154, 137.306, 178.036, 216.480];
pub const SCALE_LENGTH: f64 = 250.;

pub const E_STRING: StringProperties = StringProperties {
    length: 1000.,
    weight: 32.,
};

pub const A_STRING: StringProperties = StringProperties {
    length: 1000.,
    weight: 24.,
};

pub const D_STRING: StringProperties = StringProperties {
    length: 1000.,
    weight: 16.,
};

pub const G_STRING: StringProperties = StringProperties {
    length: 1000.,
    weight: 9.,
};

#[derive(Debug, Clone, Copy)]
pub struct StringProperties {
    pub length: f64, // mm
    pub weight: f64, // grams
}

impl StringProperties {
    /// Computes linear density in grams per mm
    pub fn linear_density(&self) -> f64 {
        self.weight / self.length
    }
}

#[derive(Debug)]
pub struct VibratingString {
    pub scale_length: f64,           // mm
    pub fret_distance_from_nut: f64, // mm
    pub tension: Option<f64>,        // Newtons
    pub string: StringProperties,
}

impl VibratingString {
    pub fn frequency(&self) -> f64 {
        let length = self.scale_length - self.fret_distance_from_nut;
        let tension_in_newtons = self.tension.unwrap_or(1.) * 1e6;
        1. / (2. * length) * (tension_in_newtons / self.string.linear_density()).sqrt()
    }
}

#[test]
fn test_freqs() {
    for (&string, string_name) in [E_STRING, A_STRING, D_STRING, G_STRING]
        .iter()
        .zip(["E", "A", "D", "G"].iter())
    {
        for (fret_number, &pos) in FRET_POSITIONS.iter().enumerate() {
            let string = VibratingString {
                scale_length: SCALE_LENGTH,
                fret_distance_from_nut: pos,
                tension: Some(100.),
                string,
            };

            println!(
                "{string_name}-string, fret #{fret_number} => {} Hz",
                string.frequency()
            )
        }
    }
}
