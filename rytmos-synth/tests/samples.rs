use std::fs::File;

use fixed::types::I1F15;
use rytmos_synth::synth::{
    samples::{sample, Kick},
    SAMPLE_RATE,
};

#[test]
fn wav_to_bin() {
    for i in 0..200 {
        let sample = sample::<Kick>(i);
        println!("{:?}", sample);
    }
}

/// Takes a .wav file and converts it to a file that the Sample trait supports including over include_bytes!()
#[test]
fn wav2bin() {
    use std::io::Write;

    let path = "/tmp/kick.wav".to_string();
    let mut file = File::create(path.clone().replace(".wav", ".bin")).unwrap();

    let mut reader = hound::WavReader::open(path).expect("Failed to open WAV file");

    let sample_len = reader.samples::<i16>().len();
    println!("sample_len={sample_len}");

    let samples = reader
        .samples::<i16>()
        .map(|r| r.unwrap())
        .map(|v| I1F15::wrapping_from_num(v as f32 / i16::MAX as f32))
        .collect::<Vec<_>>();

    let resampled = resample(reader.spec().sample_rate, SAMPLE_RATE as u32, &samples);
    println!("resampled={}", resampled.len());

    for value in resampled {
        let word = value.to_bits() as u16;
        let low = word as u8;
        let high = (word >> 8) as u8;
        file.write_all(&[high, low]).unwrap();
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

fn resample(input_rate: u32, output_rate: u32, data: &[I1F15]) -> Vec<I1F15> {
    println!(
        "Resampling {} samples from {} to {}",
        data.len(),
        input_rate,
        output_rate
    );

    let ratio = input_rate as f32 / output_rate as f32;
    let output_len = ((data.len() as f32) / ratio).ceil() as usize;

    let mut resampled = Vec::new();

    for i in 0..output_len {
        let input_pos = i as f32 * ratio;
        let idx = input_pos as usize;
        let frac = input_pos - idx as f32;

        let interpolated = if idx + 1 < data.len() {
            let left: f32 = data[idx].to_num();
            let right: f32 = data[idx + 1].to_num();
            I1F15::from_num(left + (right - left) * frac)
        } else {
            data[idx]
        };

        resampled.push(interpolated);
    }

    resampled
}
