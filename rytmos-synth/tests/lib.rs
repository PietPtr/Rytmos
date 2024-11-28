use std::sync::Once;

use fixed::{
    traits::Fixed,
    types::{extra::U15, I0F16, I14F2, I1F15, U14F2, U1F15, U4F4, U8F8},
    FixedU32,
};
use plotters::prelude::*;
use rand::Rng;
use rytmos_engrave::*;
use rytmos_synth::{
    commands::Command,
    synth::{
        lpf::LowPassFilter,
        metronome::MetronomeSynth,
        overtone::{OvertoneSynth, OvertoneSynthSettings},
        sawtooth::{SawtoothSynth, SawtoothSynthSettings},
        sine::{SineSynth, SineSynthSettings},
        vibrato::{VibratoSynth, VibratoSynthSettings},
        Synth, SAMPLE_RATE,
    },
};

static INIT: Once = Once::new();

fn init_logger() {
    INIT.call_once(|| {
        env_logger::init();
    });
}

// TODO: repair all the synths
#[test]
fn test_sine_synth() {
    init_logger();

    const SAMPLES: usize = 120100;

    let mut synth = SineSynth::new(
        0,
        SineSynthSettings {
            attack_gain: U4F4::from_num(0.9),
            initial_phase: I1F15::from_num(0.),
            decay_per_second: 0.1,
        },
    );

    let samples: Vec<i16> = (0..SAMPLES)
        .map(|i| {
            if i == 20 {
                synth.play(c!(0), U4F4::from_num(1.2))
            }
            if i == 40000 {
                synth.play(e!(4), U4F4::from_num(1.2))
            }
            if i == 80000 {
                synth.play(g!(4), U4F4::from_num(1.2))
            }
            synth.next().to_bits()
        })
        .collect();

    plot_samples(&samples[..2400]).unwrap();
    export_to_wav(samples, "signal.wav");
}

fn calculate_errors(true_values: &[i16], approx_values: &[i16]) -> (f64, f64) {
    let len = true_values.len();
    let mut mae = 0.0;
    let mut mse = 0.0;

    for (&true_val, &approx_val) in true_values.iter().zip(approx_values.iter()) {
        let error = (true_val as i32 - approx_val as i32) as f64;
        mae += error.abs() as f64;
        mse += error * error;
    }

    mae /= len as f64;
    mse /= len as f64;
    let rmse = mse.sqrt();

    (mae, rmse)
}

#[test]
fn test_sine_error() {
    // TODO: verify if the sine generator generates the correct frequency, and finish this error thing
    init_logger();

    const SAMPLES: usize = 6400;

    let mut synth = SineSynth::new(
        0,
        SineSynthSettings {
            attack_gain: U4F4::from_num(1.),
            initial_phase: I1F15::from_num(0.),
            decay_per_second: 0.1,
        },
    );

    let samples: Vec<i16> = (0..SAMPLES)
        .map(|i| {
            if i == 0 {
                synth.play(a!(4), U4F4::from_num(1.0))
            }
            synth.next().to_bits()
        })
        .collect();

    const SAMPLE_RATE: f64 = 24000.0;
    const C0_FREQUENCY: f64 = 439.45; // TODO: calculate what this is due to fixed point stuff
    const AMPLITUDE: i16 = (1. * i16::MAX as f64) as i16;

    let sine_wave: Vec<i16> = (0..SAMPLES)
        .map(|n| {
            let theta = 2.0 * std::f64::consts::PI * C0_FREQUENCY * (n as f64) / SAMPLE_RATE;
            (AMPLITUDE as f64 * theta.sin()).round() as i16
        })
        .collect();

    plot_two_samples(&samples, &sine_wave).unwrap();
    let (mae, mse) = calculate_errors(&sine_wave, &samples);

    let mae = mae / 65536.;
    let mse = mse / 65536.;

    println!("MAE={mae} MSE={mse}");
}

// #[test]
// fn test_vibrato_synth() {
//     init_logger();

//     let mut synth = VibratoSynth::new(VibratoSynthSettings {
//         sine_settings: SineSynthSettings {
//             attack_gain: 1.0,
//             initial_phase: 0.0,
//             decay_per_second: 1.0,
//         },
//         vibrato_frequency: 5.0,
//         vibrato_strength: 0.0001,
//     });

//     synth.play(a!(4), 1.0);

//     let samples: Vec<i16> = (0..44100).map(|_| synth.next()).collect();

//     plot_samples(&samples[..22000]).unwrap();
//     export_to_wav(samples, "signal.wav");
// }

// #[test]
// fn test_lpf() {
//     init_logger();

//     // Run a very distorted sine synth
//     let mut synth = SineSynth::new(SineSynthSettings {
//         attack_gain: 5.0,
//         initial_phase: 0.,
//         decay_per_second: 0.2,
//     });

//     // But filter it aggressively
//     let mut lpf = LowPassFilter::new(250.);

//     synth.play(a!(0), 1.0);

//     let samples: Vec<i16> = (0..44100).map(|_| lpf.next(synth.next())).collect();

//     plot_samples(&samples[..22000]).unwrap();
//     export_to_wav(samples, "signal.wav");
// }

#[test]
fn test_metronome() {
    init_logger();

    let mut synth = MetronomeSynth::new(0);

    let mut samples = vec![];

    // TODO: broken?
    for _ in 0..4 {
        synth.play(a!(0), U4F4::from_num(1.0));
        let mut samples_new: Vec<_> = (0..10000).map(|_| synth.next().to_bits()).collect();
        samples.append(&mut samples_new);
    }

    plot_samples(&samples[..40000]).unwrap();
    export_to_wav(samples, "signal.wav");
}

// #[test]
// fn test_overtone_synth() {
//     init_logger();

//     let synths = [
//         SineSynth::new(SineSynthSettings {
//             attack_gain: 0.38,
//             initial_phase: 0.13,
//             decay_per_second: 0.5,
//         }),
//         SineSynth::new(SineSynthSettings {
//             attack_gain: 0.4,
//             initial_phase: 0.77,
//             decay_per_second: 0.6,
//         }),
//         SineSynth::new(SineSynthSettings {
//             attack_gain: 0.34,
//             initial_phase: 0.21,
//             decay_per_second: 0.5,
//         }),
//         SineSynth::new(SineSynthSettings {
//             attack_gain: 0.02,
//             initial_phase: 0.29,
//             decay_per_second: 0.4,
//         }),
//         SineSynth::new(SineSynthSettings {
//             attack_gain: 0.01,
//             initial_phase: 0.11,
//             decay_per_second: 0.3,
//         }),
//         SineSynth::new(SineSynthSettings {
//             attack_gain: 0.01,
//             initial_phase: 0.59,
//             decay_per_second: 0.2,
//         }),
//     ];

//     let mut synth = OvertoneSynth::new(OvertoneSynthSettings {}, synths);

//     let samples: Vec<i16> = (0..88100)
//         .map(|i| {
//             if i == 250 {
//                 synth.play(e!(1), 1.2)
//             }
//             if i == 34000 {
//                 synth.play(e!(1), 0.);
//             }
//             synth.next()
//         })
//         .collect();

//     plot_samples(&samples[..22000]).unwrap();

//     export_to_wav(samples, "signal.wav");
// }

#[test]
fn test_sawtooth_synth() {
    init_logger();

    let mut synth = SawtoothSynth::new(
        0x0,
        SawtoothSynthSettings {
            decay: U8F8::from_num(0.90),
        },
    );

    synth.play(a!(4), U4F4::from_num(1.01));

    const LEN: usize = 44100;

    let samples: Vec<i16> = (0..LEN)
        .map(|_| synth.next())
        .map(|i| i.to_bits())
        .collect();

    plot_samples(&samples[..LEN]).unwrap();
    export_to_wav(samples, "signal.wav");
}

fn plot_samples(samples: &[i16]) -> Result<(), Box<dyn std::error::Error>> {
    let root = BitMapBackend::new("graph.png", (800, 600)).into_drawing_area();
    root.fill(&WHITE)?;

    let y_min = *samples.iter().min().unwrap() as i32;
    let y_max = *samples.iter().max().unwrap() as i32;

    let mut chart = ChartBuilder::on(&root)
        .x_label_area_size(30)
        .y_label_area_size(40)
        .build_cartesian_2d(0..(samples.len() as i32), y_min..y_max)?;

    chart.configure_mesh().draw()?;

    chart
        .draw_series(LineSeries::new(
            samples
                .iter()
                .enumerate()
                .map(|(x, y)| (x as i32, *y as i32)),
            &BLUE,
        ))?
        .label("Samples")
        .legend(|(x, y)| PathElement::new([(x, y), (x + 20, y)], BLUE));

    chart.configure_series_labels().border_style(BLACK).draw()?;

    Ok(())
}

fn plot_two_samples(samples1: &[i16], samples2: &[i16]) -> Result<(), Box<dyn std::error::Error>> {
    let root = BitMapBackend::new("graph.png", (800, 600)).into_drawing_area();
    root.fill(&WHITE)?;

    let y_min = *samples1.iter().chain(samples2.iter()).min().unwrap() as i32;
    let y_max = *samples1.iter().chain(samples2.iter()).max().unwrap() as i32;

    let mut chart = ChartBuilder::on(&root)
        .x_label_area_size(30)
        .y_label_area_size(40)
        .build_cartesian_2d(0..(samples1.len().max(samples2.len()) as i32), y_min..y_max)?;

    chart.configure_mesh().draw()?;

    chart
        .draw_series(LineSeries::new(
            samples1
                .iter()
                .enumerate()
                .map(|(x, y)| (x as i32, *y as i32)),
            &BLUE,
        ))?
        .label("Samples 1")
        .legend(|(x, y)| PathElement::new([(x, y), (x + 20, y)], BLUE));

    chart
        .draw_series(LineSeries::new(
            samples2
                .iter()
                .enumerate()
                .map(|(x, y)| (x as i32, *y as i32)),
            &RED,
        ))?
        .label("Samples 2")
        .legend(|(x, y)| PathElement::new([(x, y), (x + 20, y)], RED));

    chart.configure_series_labels().border_style(BLACK).draw()?;

    Ok(())
}

#[test]
fn test_command_serdes() {
    let mut rng = rand::thread_rng();

    let mut valid_commands = 0;

    for i in 0..10000000 {
        let mut value: u32 = rng.gen();
        let command_id = rng.gen_range(0..8) & 0b111111;

        value &= 0b11110000_00111111_11111111_11111111;
        value |= command_id << 22;

        if let Some(cmd) = Command::deserialize(value) {
            valid_commands += 1;
            let serialized = cmd.serialize();
            assert_eq!(
                value, serialized,
                "Failed serdes test #{i}: {:#?} => \n{:032b} =/=\n{:032b}",
                cmd, value, serialized
            );
        }
    }

    println!("Serialized {} valid commands.", valid_commands);
    assert!(valid_commands > 0);
}

fn export_to_wav(samples: Vec<i16>, file_path: &str) {
    let spec = hound::WavSpec {
        channels: 1,
        sample_rate: SAMPLE_RATE as u32,
        bits_per_sample: 16,
        sample_format: hound::SampleFormat::Int,
    };

    let mut writer = hound::WavWriter::create(file_path, spec).unwrap();

    for sample in samples {
        writer.write_sample(sample).unwrap();
    }

    writer.finalize().unwrap();
}

#[test]
fn try_fixed_crate() {
    let test = I1F15::checked_from_num(0.4);
    dbg!(test);
    let test: i16 = test.unwrap().to_bits();
    dbg!(test);
    dbg!(I1F15::MAX, I1F15::MIN);
}

#[test]
fn print_frequency_bit_consts() {
    let freqs: [U14F2; 128] = [
        U14F2::from_num(8.175799),
        U14F2::from_num(8.661957),
        U14F2::from_num(9.177024),
        U14F2::from_num(9.722718),
        U14F2::from_num(10.300861),
        U14F2::from_num(10.913382),
        U14F2::from_num(11.562326),
        U14F2::from_num(12.249857),
        U14F2::from_num(12.978272),
        U14F2::from_num(13.750000),
        U14F2::from_num(14.567618),
        U14F2::from_num(15.433853),
        U14F2::from_num(16.351598),
        U14F2::from_num(17.323914),
        U14F2::from_num(18.354048),
        U14F2::from_num(19.445436),
        U14F2::from_num(20.601722),
        U14F2::from_num(21.826764),
        U14F2::from_num(23.124651),
        U14F2::from_num(24.499715),
        U14F2::from_num(25.956544),
        U14F2::from_num(27.500000),
        U14F2::from_num(29.135235),
        U14F2::from_num(30.867706),
        U14F2::from_num(32.703196),
        U14F2::from_num(34.647829),
        U14F2::from_num(36.708096),
        U14F2::from_num(38.890873),
        U14F2::from_num(41.203445),
        U14F2::from_num(43.653529),
        U14F2::from_num(46.249303),
        U14F2::from_num(48.999429),
        U14F2::from_num(51.913087),
        U14F2::from_num(55.000000),
        U14F2::from_num(58.270470),
        U14F2::from_num(61.735413),
        U14F2::from_num(65.406391),
        U14F2::from_num(69.295658),
        U14F2::from_num(73.416192),
        U14F2::from_num(77.781746),
        U14F2::from_num(82.406889),
        U14F2::from_num(87.307058),
        U14F2::from_num(92.498606),
        U14F2::from_num(97.998859),
        U14F2::from_num(103.826174),
        U14F2::from_num(110.000000),
        U14F2::from_num(116.540940),
        U14F2::from_num(123.470825),
        U14F2::from_num(130.812783),
        U14F2::from_num(138.591315),
        U14F2::from_num(146.832384),
        U14F2::from_num(155.563492),
        U14F2::from_num(164.813778),
        U14F2::from_num(174.614116),
        U14F2::from_num(184.997211),
        U14F2::from_num(195.997718),
        U14F2::from_num(207.652349),
        U14F2::from_num(220.000000),
        U14F2::from_num(233.081881),
        U14F2::from_num(246.941651),
        U14F2::from_num(261.625565),
        U14F2::from_num(277.182631),
        U14F2::from_num(293.664768),
        U14F2::from_num(311.126984),
        U14F2::from_num(329.627557),
        U14F2::from_num(349.228231),
        U14F2::from_num(369.994423),
        U14F2::from_num(391.995436),
        U14F2::from_num(415.304698),
        U14F2::from_num(440.000000),
        U14F2::from_num(466.163762),
        U14F2::from_num(493.883301),
        U14F2::from_num(523.251131),
        U14F2::from_num(554.365262),
        U14F2::from_num(587.329536),
        U14F2::from_num(622.253967),
        U14F2::from_num(659.255114),
        U14F2::from_num(698.456463),
        U14F2::from_num(739.988845),
        U14F2::from_num(783.990872),
        U14F2::from_num(830.609395),
        U14F2::from_num(880.000000),
        U14F2::from_num(932.327523),
        U14F2::from_num(987.766603),
        U14F2::from_num(1046.502261),
        U14F2::from_num(1108.730524),
        U14F2::from_num(1174.659072),
        U14F2::from_num(1244.507935),
        U14F2::from_num(1318.510228),
        U14F2::from_num(1396.912926),
        U14F2::from_num(1479.977691),
        U14F2::from_num(1567.981744),
        U14F2::from_num(1661.218790),
        U14F2::from_num(1760.000000),
        U14F2::from_num(1864.655046),
        U14F2::from_num(1975.533205),
        U14F2::from_num(2093.004522),
        U14F2::from_num(2217.461048),
        U14F2::from_num(2349.318143),
        U14F2::from_num(2489.015870),
        U14F2::from_num(2637.020455),
        U14F2::from_num(2793.825851),
        U14F2::from_num(2959.955382),
        U14F2::from_num(3135.963488),
        U14F2::from_num(3322.437581),
        U14F2::from_num(3520.000000),
        U14F2::from_num(3729.310092),
        U14F2::from_num(3951.066410),
        U14F2::from_num(4186.009045),
        U14F2::from_num(4434.922096),
        U14F2::from_num(4698.636287),
        U14F2::from_num(4978.031740),
        U14F2::from_num(5274.040911),
        U14F2::from_num(5587.651703),
        U14F2::from_num(5919.910763),
        U14F2::from_num(6271.926976),
        U14F2::from_num(6644.875161),
        U14F2::from_num(7040.000000),
        U14F2::from_num(7458.620184),
        U14F2::from_num(7902.132820),
        U14F2::from_num(8372.018090),
        U14F2::from_num(8869.844191),
        U14F2::from_num(9397.272573),
        U14F2::from_num(9956.063479),
        U14F2::from_num(10548.081821),
        U14F2::from_num(11175.303406),
        U14F2::from_num(11839.821527),
        U14F2::from_num(12543.853951),
    ];

    // Bit constants for frequency values
    for f in freqs {
        println!("U14F2::from_bits({:#018b}),", f.to_bits());
    }

    // Bit constants for increments at the given sample rate.
    let sample_rate = 24000;
    for f in freqs {
        let inc = if f < 10000 {
            I1F15::from_num(FixedU32::<U15>::from(f) / FixedU32::<U15>::from_num(sample_rate))
        } else {
            I1F15::from_num(0)
        };

        println!("I1F15::from_bits({:#018b}),", inc.to_bits());
    }
}

#[test]
fn convert_i16_table_to_i1f15() {
    dbg!(I1F15::MIN, I1F15::MAX, I1F15::from_bits(1));
    // for sample in rytmos_synth::synth::samples::weak::WEAK_WAV {
    //     let converted = I1F15::from_num(sample as f32 / i16::MAX as f32);
    //     println!("I1F15::from_bits({:#018b}),", converted.to_bits());
    // }
}
