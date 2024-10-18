use std::sync::Once;

use plotters::prelude::*;
use rand::Rng;
use rytmos_engrave::*;
use rytmos_synth::{
    commands::Command,
    synth::{
        lpf::LowPassFilter,
        metronome::{MetronomeSettings, MetronomeSynth},
        overtone::{OvertoneSynth, OvertoneSynthSettings},
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

#[test]
fn test_sine_synth() {
    init_logger();

    const SAMPLES: usize = 44100;

    let mut synth = SineSynth::new(SineSynthSettings {
        attack_gain: 0.9,
        initial_phase: 0.1,
        decay_per_second: 0.1,
    });

    let samples: Vec<i16> = (0..SAMPLES)
        .map(|i| {
            if i == 250 {
                synth.play(a!(1), 1.2)
            }
            synth.next()
        })
        .collect();

    plot_samples(&samples[..22000]).unwrap();
    export_to_wav(samples, "signal.wav");
}

#[test]
fn test_vibrato_synth() {
    init_logger();

    let mut synth = VibratoSynth::new(VibratoSynthSettings {
        sine_settings: SineSynthSettings {
            attack_gain: 1.0,
            initial_phase: 0.0,
            decay_per_second: 1.0,
        },
        vibrato_frequency: 5.0,
        vibrato_strength: 0.0001,
    });

    synth.play(a!(4), 1.0);

    let samples: Vec<i16> = (0..44100).map(|_| synth.next()).collect();

    plot_samples(&samples[..22000]).unwrap();
    export_to_wav(samples, "signal.wav");
}

#[test]
fn test_lpf() {
    init_logger();

    // Run a very distorted sine synth
    let mut synth = SineSynth::new(SineSynthSettings {
        attack_gain: 5.0,
        initial_phase: 0.,
        decay_per_second: 0.2,
    });

    // But filter it aggressively
    let mut lpf = LowPassFilter::new(250.);

    synth.play(a!(0), 1.0);

    let samples: Vec<i16> = (0..44100).map(|_| lpf.next(synth.next())).collect();

    plot_samples(&samples[..22000]).unwrap();
    export_to_wav(samples, "signal.wav");
}

#[test]
fn test_metronome() {
    init_logger();

    let mut synth = MetronomeSynth::new(MetronomeSettings {
        bpm: 120,
        accent_one: true,
    });

    synth.play(a!(0), 1.0);

    let samples: Vec<i16> = (0..44100).map(|_| synth.next()).collect();

    plot_samples(&samples[..44000]).unwrap();
    export_to_wav(samples, "signal.wav");
}

#[test]
fn test_overtone_synth() {
    init_logger();

    let synths = [
        SineSynth::new(SineSynthSettings {
            attack_gain: 0.38,
            initial_phase: 0.13,
            decay_per_second: 0.5,
        }),
        SineSynth::new(SineSynthSettings {
            attack_gain: 0.4,
            initial_phase: 0.77,
            decay_per_second: 0.6,
        }),
        SineSynth::new(SineSynthSettings {
            attack_gain: 0.34,
            initial_phase: 0.21,
            decay_per_second: 0.5,
        }),
        SineSynth::new(SineSynthSettings {
            attack_gain: 0.02,
            initial_phase: 0.29,
            decay_per_second: 0.4,
        }),
        SineSynth::new(SineSynthSettings {
            attack_gain: 0.01,
            initial_phase: 0.11,
            decay_per_second: 0.3,
        }),
        SineSynth::new(SineSynthSettings {
            attack_gain: 0.01,
            initial_phase: 0.59,
            decay_per_second: 0.2,
        }),
    ];

    let mut synth = OvertoneSynth::new(OvertoneSynthSettings {}, synths);

    let samples: Vec<i16> = (0..88100)
        .map(|i| {
            if i == 250 {
                synth.play(e!(1), 1.2)
            }
            if i == 34000 {
                synth.play(e!(1), 0.);
            }
            synth.next()
        })
        .collect();

    plot_samples(&samples[..22000]).unwrap();

    export_to_wav(samples, "signal.wav");
}

fn plot_samples(samples: &[i16]) -> Result<(), Box<dyn std::error::Error>> {
    let root = BitMapBackend::new("graph.png", (1600, 1200)).into_drawing_area();
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

#[test]
fn test_command_serdes() {
    let mut rng = rand::thread_rng();

    for _ in 0..10000000 {
        let mut value: u32 = rng.gen();
        let command_id = rng.gen_range(0..8) & 0b111111;

        value &= 0b00000011_11111111_11111111_11111111;
        value |= command_id << 26;

        if let Some(cmd) = Command::deserialize(value) {
            let serialized = cmd.serialize();
            assert_eq!(
                value, serialized,
                "Failed serdes test: {:#?} => \n{:032b} =/=\n{:032b}",
                cmd, value, serialized
            );
        }
    }
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
