use fugit::HertzU32;

pub const EXTERNAL_XTAL_FREQ_HZ: HertzU32 = HertzU32::from_raw(12_000_000u32);
pub const RP2040_CLOCK_HZ: HertzU32 = HertzU32::from_raw(307_200_000u32);

pub const SAMPLE_RATE: HertzU32 = HertzU32::from_raw(24_000u32);
pub const PIO_INSTRUCTIONS_PER_SAMPLE: u32 = 2;
pub const NUM_CHANNELS: u32 = 2;
pub const SAMPLE_RESOLUTION: u32 = 16; // in bits per sample

pub const I2S_PIO_CLOCK_HZ: HertzU32 = HertzU32::from_raw(
    SAMPLE_RATE.raw() * NUM_CHANNELS * SAMPLE_RESOLUTION * PIO_INSTRUCTIONS_PER_SAMPLE,
);
pub const I2S_PIO_CLOCKDIV_INT: u16 = (RP2040_CLOCK_HZ.raw() / I2S_PIO_CLOCK_HZ.raw()) as u16;
pub const I2S_PIO_CLOCKDIV_FRAC: u8 =
    (((RP2040_CLOCK_HZ.raw() % I2S_PIO_CLOCK_HZ.raw()) * 256) / I2S_PIO_CLOCK_HZ.raw()) as u8;

pub const MCLK_HZ: HertzU32 = HertzU32::from_raw(8 * I2S_PIO_CLOCK_HZ.raw());
pub const MCLK_CLOCKDIV_INT: u16 = (RP2040_CLOCK_HZ.raw() / MCLK_HZ.raw()) as u16;
pub const MCLK_CLOCKDIV_FRAC: u8 =
    (((RP2040_CLOCK_HZ.raw() % MCLK_HZ.raw()) * 256) / MCLK_HZ.raw()) as u8;

pub const BUFFER_SIZE: usize = 16;
