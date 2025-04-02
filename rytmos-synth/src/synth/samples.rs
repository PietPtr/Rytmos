use fixed::types::I1F15;

pub trait Sample {
    const DATA: &'static [u8];
}

pub fn sample<S: Sample>(index: usize) -> Option<I1F15> {
    sample_unsafe(S::DATA, index)
}

pub fn sample_unsafe(data: &[u8], index: usize) -> Option<I1F15> {
    let low_index = index * 2;
    let high_index = index * 2 + 1;

    if high_index >= data.len() {
        return None;
    }

    let low = data[low_index] as u16;
    let high = data[high_index] as u16;

    let sample = (low << 8) | high;

    Some(I1F15::from_bits(sample as i16))
}

pub struct Kick {}
impl Sample for Kick {
    const DATA: &'static [u8] = include_bytes!("samples/kick.bin");
}

pub struct Snare {}
impl Sample for Snare {
    const DATA: &'static [u8] = include_bytes!("samples/snare.bin");
}

pub struct Hihat {}
impl Sample for Hihat {
    const DATA: &'static [u8] = include_bytes!("samples/hihat.bin");
}

pub struct Untitled {}
impl Sample for Untitled {
    const DATA: &'static [u8] = include_bytes!("samples/untitled.bin");
}

pub struct Strong {}
impl Sample for Strong {
    const DATA: &'static [u8] = include_bytes!("samples/strong.bin");
}

pub struct Weak {}
impl Sample for Weak {
    const DATA: &'static [u8] = include_bytes!("samples/weak.bin");
}

pub struct Cymbal {}
impl Sample for Cymbal {
    const DATA: &'static [u8] = include_bytes!("samples/cymbal.bin");
}
