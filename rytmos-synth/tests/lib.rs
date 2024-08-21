use std::sync::Once;

use log::info;
use plotters::prelude::*;
use rytmos_synth::synth::{SineSynth, SAMPLE_RATE};

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

    let mut synth = SineSynth::new(40.0, 0.9, 0.0, 0.1);
    let samples: Vec<i16> = (0..SAMPLES).map(|_| synth.next().unwrap()).collect();

    let mut synth = SineSynth::new(80.0, 1.0, 0.1, 0.2);
    let samples_2: Vec<i16> = (0..SAMPLES).map(|_| synth.next().unwrap()).collect();

    let mut synth = SineSynth::new(160.0, 0.53, 0.9, 0.01);
    let samples_3: Vec<i16> = (0..SAMPLES).map(|_| synth.next().unwrap()).collect();

    let mut synth = SineSynth::new(320.0, 0.26, 0.5, 0.01);
    let samples_4: Vec<i16> = (0..SAMPLES).map(|_| synth.next().unwrap()).collect();

    let components = vec![samples, samples_2, samples_3, samples_4];

    let vector_len = components[0].len();
    let signal = components.into_iter().fold(vec![0; vector_len], |acc, v| {
        acc.iter()
            .zip(v.iter())
            .map(|(&a, &b): (&i16, &i16)| a.saturating_add(b))
            .collect()
    });

    plot_samples(&signal[..22000]).unwrap();

    export_to_wav(signal, "signal.wav");
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
        .legend(|(x, y)| PathElement::new([(x, y), (x + 20, y)], &BLUE));

    chart
        .configure_series_labels()
        .border_style(&BLACK)
        .draw()?;

    Ok(())
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
