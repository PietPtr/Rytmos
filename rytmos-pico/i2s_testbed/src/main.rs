#![no_std]
#![no_main]

#[link_section = ".boot2"]
#[no_mangle]
#[used]
pub static BOOT2_FIRMWARE: [u8; 256] = rp2040_boot2::BOOT_LOADER_W25Q080;

use core::any::Any;
use core::u32;

use cortex_m::singleton;
use defmt::*;
use defmt_rtt as _;
use embedded_hal::digital::v2::OutputPin;
use fixed::types::I1F15;
use fixed::types::U4F4;
use fugit::HertzU32;
use heapless::Vec;
use panic_probe as _;
use rp_pico::hal::Timer;
use rp_pico::pac::Peripherals;
use rp_pico::{
    entry,
    hal::{
        clocks::{Clock, ClockSource, ClocksManager, InitError},
        dma::{double_buffer, DMAExt},
        gpio::{self, FunctionPio0},
        multicore::{Multicore, Stack},
        pio::{Buffers, PIOBuilder, PIOExt, PinDir, ShiftDirection},
        pll::{common_configs::PLL_USB_48MHZ, setup_pll_blocking},
        sio::Sio,
        watchdog::Watchdog,
        xosc::setup_xosc_blocking,
    },
    pac,
};

use rytmos_engrave::a;
use rytmos_synth::effect::exponential_decay::ExponentialDecay;
use rytmos_synth::effect::exponential_decay::ExponentialDecaySettings;
use rytmos_synth::effect::linear_decay::LinearDecay;
use rytmos_synth::effect::lpf::LowPassFilter;
use rytmos_synth::synth::composed::overtone::OvertoneSynth;
use rytmos_synth::synth::composed::overtone::OvertoneSynthSettings;
use rytmos_synth::synth::composed::synth_with_effects::SynthWithEffect;
use rytmos_synth::synth::composed::synth_with_effects::SynthWithEffectSettings;
use rytmos_synth::synth::metronome::MetronomeSynth;
use rytmos_synth::synth::nothing::NothingSynth;
use rytmos_synth::synth::sine::SineSynth;
use rytmos_synth::synth::sine::SineSynthSettings;
use rytmos_synth::synth::vibrato::VibratoSynth;
use rytmos_synth::synth::vibrato::VibratoSynthSettings;
use rytmos_synth::synth::{
    sawtooth::{SawtoothSynth, SawtoothSynthSettings},
    Synth,
};

use common::consts::*;
use common::plls;

static mut CORE1_STACK: Stack<4096> = Stack::new();

fn synth_core(sys_freq: u32) -> ! {
    let mut pac = unsafe { pac::Peripherals::steal() };
    let core = unsafe { pac::CorePeripherals::steal() };
    let sio = Sio::new(pac.SIO);
    let pins = gpio::Pins::new(
        pac.IO_BANK0,
        pac.PADS_BANK0,
        sio.gpio_bank0,
        &mut pac.RESETS,
    );
    let mut delay = cortex_m::delay::Delay::new(core.SYST, sys_freq);
    let clocks = ClocksManager::new(pac.CLOCKS);
    let timer = Timer::new(pac.TIMER, &mut pac.RESETS, &clocks);

    let i2s_sck_pin = pins.gpio12.into_function::<FunctionPio0>();
    let i2s_din_pin = pins.gpio13.into_function::<FunctionPio0>();
    let i2s_bck_pin = pins.gpio14.into_function::<FunctionPio0>();
    let i2s_lck_pin = pins.gpio15.into_function::<FunctionPio0>();

    let (pio_i2s_mclk_output, pio_i2s_send_master) = common::pio::i2s_programs();

    let (mut pio, sm0, sm1, _, _) = pac.PIO0.split(&mut pac.RESETS);
    let pio_i2s_mclk_output = pio.install(&pio_i2s_mclk_output).unwrap();
    let pio_i2s_send_master = pio.install(&pio_i2s_send_master).unwrap();

    let (mut sm0, _rx0, _tx0) = PIOBuilder::from_program(pio_i2s_mclk_output)
        .set_pins(i2s_sck_pin.id().num, 1)
        .clock_divisor_fixed_point(MCLK_CLOCKDIV_INT, MCLK_CLOCKDIV_FRAC)
        .build(sm0);

    let (mut sm1, _rx1, tx1) = PIOBuilder::from_program(pio_i2s_send_master)
        .out_pins(i2s_din_pin.id().num, 1)
        .side_set_pin_base(i2s_bck_pin.id().num)
        .clock_divisor_fixed_point(I2S_PIO_CLOCKDIV_INT, I2S_PIO_CLOCKDIV_FRAC)
        .out_shift_direction(ShiftDirection::Left)
        .autopull(true)
        .pull_threshold(16u8)
        .buffers(Buffers::OnlyTx)
        .build(sm1);

    sm0.set_pindirs([(i2s_sck_pin.id().num, PinDir::Output)]);
    sm0.start();
    sm1.set_pindirs([
        (i2s_din_pin.id().num, PinDir::Output),
        (i2s_lck_pin.id().num, PinDir::Output),
        (i2s_bck_pin.id().num, PinDir::Output),
    ]);
    sm1.start();

    let dma_channels = pac.DMA.split(&mut pac.RESETS);
    let i2s_tx_buf1 = singleton!(: [u32; BUFFER_SIZE*2] = [0; BUFFER_SIZE*2]).unwrap();
    let i2s_tx_buf2 = singleton!(: [u32; BUFFER_SIZE*2] = [0; BUFFER_SIZE*2]).unwrap();
    let i2s_dma_config =
        double_buffer::Config::new((dma_channels.ch0, dma_channels.ch1), i2s_tx_buf1, tx1);
    let i2s_tx_transfer = i2s_dma_config.start();
    let mut i2s_tx_transfer = i2s_tx_transfer.read_next(i2s_tx_buf2);

    delay.delay_ms(100);

    // Set up systick timer correctly so we can benchmark

    #[cfg(feature = "benchmark")]
    {
        pac.PPB.syst_csr.write(|w| unsafe { w.bits(0x5) });
        pac.PPB.syst_rvr.write(|w| unsafe { w.bits(0xffffff) });
    }

    info!("Start Synth core.");

    // --- Benchmarks ---
    // Each of these synths is benchmarked on a real pico running at ~307MHz (SYS_PLL_CONFIG_307P2MHZ from plls.rs in common)
    // Driving i2s at a 24kHz sample rate. This means that for each buffer of 16 samples we have ~666us to
    // compute all the samples (this is what the percentages show). Everything compiled in release mode.
    // A single A4 note is played immediately and 512 samples are generated

    // TIME: average=60us (9.0%) min=39us (5.85%) max=82us (12.31%)
    // CYCLES: average=18679 min=12257 max=25058
    let vibrato = VibratoSynth::make(
        0,
        VibratoSynthSettings {
            sine_settings: SineSynthSettings {
                extra_attack_gain: U4F4::from_num(1),
                initial_phase: I1F15::from_num(0),
                do_lerp: true,
            },
            vibrato_velocity: U4F4::from_num(1),
            vibrato_synth_divider: 7,
            vibrato_strength: 5,
        },
    );

    // TIME: average=5us (0.75%) min=5us (0.75%) max=7us (1.05%)
    // CYCLES: average=1806 (112) min=1732 max=1883
    let sawtooth = SawtoothSynth::make(0, ());

    // TIME: average=44us (6.6%) min=27us (4.05%) max=61us (9.15%)
    // CYCLES: average=13672 (854) min=8197 max=18561
    let sine = SineSynth::make(0, SineSynthSettings::default());

    // TIME: average=41us (6.15%) min=24us (3.6%) max=58us (8.7%)
    // CYCLES: average=12868 (804) min=7433 max=17809
    let sine_no_lerp = SineSynth::make(
        0,
        SineSynthSettings {
            do_lerp: false,
            ..SineSynthSettings::default()
        },
    );

    // TIME: average=45us (6.75%) min=28us (4.2%) max=62us (9.3%)
    // CYCLES: average=14011 min=8523 max=18955
    let exponential_decay = SynthWithEffect::<SineSynth, ExponentialDecay>::make(
        0,
        SynthWithEffectSettings::<SineSynth, ExponentialDecay>::default(),
    );

    // TIME: average=45us (6.75%) min=27us (4.05%) max=62us (9.3%)
    // CYCLES: average=14056 min=8520 max=19019
    let linear_decay = SynthWithEffect::<SineSynth, LinearDecay>::make(
        0,
        SynthWithEffectSettings::<SineSynth, LinearDecay>::default(),
    );

    // TIME: average=45us (6.75%) min=27us (4.05%) max=62us (9.3%)
    // CYCLES: average=13952 (872) min=8392 max=18924
    let sine_lpf = SynthWithEffect::<SineSynth, LowPassFilter>::make(
        0,
        SynthWithEffectSettings::<SineSynth, LowPassFilter>::default(),
    );

    // TIME: average=210us (31.53%) min=187us (28.07%) max=230us (34.53%)
    // CYCLES: average=64741 (4046) min=57185 max=70598
    let overtone: OvertoneSynth<SynthWithEffect<SineSynth, LinearDecay>, 4> = OvertoneSynth::make(
        0,
        OvertoneSynthSettings {
            synths: [
                SynthWithEffectSettings::default(),
                SynthWithEffectSettings::default(),
                SynthWithEffectSettings::default(),
                SynthWithEffectSettings::default(),
            ],
        },
    );

    // TIME: average=0us (0.0%) min=0us (0.0%) max=1us (0.15%)
    // CYCLES: average=221 (13) min=219 max=223
    let nothing = NothingSynth::make(0, ());

    // TIME: average=3us (0.45%) min=3us (0.45%) max=7us (1.05%)
    // CYCLES: average=1143 (71) min=909 max=2114
    let metronome = MetronomeSynth::make(0, ());

    let mut synth = sawtooth;

    synth.play(a!(4), U4F4::from_num(1.));

    let mut sample = 0i16;

    let mut warned = false;

    const TIME_AVAILABLE_US: u64 = (BUFFER_SIZE as u64 * 1000000) / SAMPLE_RATE.raw() as u64;

    #[cfg(feature = "benchmark")]
    let mut times: Vec<(u64, u64), 512> = Vec::new();

    let mut i = 0;

    loop {
        if !warned && i2s_tx_transfer.is_done() {
            warn!("i2s transfer already done, probably late.");
            warned = true;
        }

        let (next_tx_buf, next_tx_transfer) = i2s_tx_transfer.wait();

        let start = timer.get_counter();
        let systick_start = pac.PPB.syst_cvr.read().current().bits();

        for (i, e) in next_tx_buf.iter_mut().enumerate() {
            if i % 2 == 0 {
                sample = synth.next().to_bits();
                *e = sample as u32 / 16;
            } else {
                *e = sample as u32 / 16;
            }
            *e <<= 16;
        }

        i2s_tx_transfer = next_tx_transfer.read_next(next_tx_buf);

        #[cfg(feature = "benchmark")]
        {
            let systick_end = pac.PPB.syst_cvr.read().current().bits();
            let end = timer.get_counter();

            let time_taken_us = (end - start).to_micros();
            let time_taken_cycles = (systick_start - systick_end) as u64;

            // reset systick counter
            pac.PPB
                .syst_cvr
                .write(|w| unsafe { w.current().bits(0xffffff) });

            trace!(
                "16 samples took {}ns / {}ns ({}%) ({} clock cycles)",
                time_taken_us,
                TIME_AVAILABLE_US,
                (time_taken_us as f64 / TIME_AVAILABLE_US as f64) * 100.,
                time_taken_cycles
            );

            if times.is_full() {
                fn as_capacity(time: u64) -> f64 {
                    ((time as f64 / TIME_AVAILABLE_US as f64) * 10000.) as u64 as f64 / 100.
                }

                let times_us = times.iter().map(|t| t.0);
                let times_cycles = times.iter().map(|t| t.1);

                let average_us = times_us.clone().sum::<u64>() / times.len() as u64;
                let min_us = times_us.clone().min().unwrap();
                let max_us = times_us.clone().max().unwrap();

                let average_cycles = times_cycles.clone().sum::<u64>() / times.len() as u64;
                let min_cycles = times_cycles.clone().min().unwrap();
                let max_cycles = times_cycles.clone().max().unwrap();

                info!(
                "\nRan {} buffers\ncapacity={}us\nTIME: average={}us ({}%) min={}us ({}%) max={}us ({}%)\nCYCLES: average={} ({}) min={} max={}",
                times.len(),
                TIME_AVAILABLE_US,
                average_us,
                as_capacity(average_us),
                min_us,
                as_capacity(min_us),
                max_us,
                as_capacity(max_us),
                average_cycles,
                average_cycles / BUFFER_SIZE as u64,
                min_cycles,
                max_cycles
            );
                break;
            } else {
                // Skip the first few samples so the system can get up to speed.
                if i > 50 {
                    times.push((time_taken_us, time_taken_cycles)).unwrap();
                }
            }
        }
        i += 1
    }

    loop {}
}

#[entry]
fn main() -> ! {
    let mut pac = pac::Peripherals::take().unwrap();
    let core = pac::CorePeripherals::take().unwrap();
    let mut watchdog = Watchdog::new(pac.WATCHDOG);
    let mut sio = Sio::new(pac.SIO);

    watchdog.enable_tick_generation((EXTERNAL_XTAL_FREQ_HZ.raw() / 1_000_000) as u8);

    let mut clocks = ClocksManager::new(pac.CLOCKS);

    common::setup_clocks!(pac, clocks);

    let pins = gpio::Pins::new(
        pac.IO_BANK0,
        pac.PADS_BANK0,
        sio.gpio_bank0,
        &mut pac.RESETS,
    );

    // Setup the other core
    let sys_freq = clocks.system_clock.freq().to_Hz();
    let mut mc = Multicore::new(&mut pac.PSM, &mut pac.PPB, &mut sio.fifo);
    let cores = mc.cores();
    let core1 = &mut cores[1];
    let _test = core1.spawn(unsafe { &mut CORE1_STACK.mem }, move || {
        synth_core(sys_freq)
    });

    info!("Start I/O thread.");

    loop {}
}
