#![no_std]
#![no_main]

#[link_section = ".boot2"]
#[no_mangle]
#[used]
pub static BOOT2_FIRMWARE: [u8; 256] = rp2040_boot2::BOOT_LOADER_W25Q080;

use core::cell::RefCell;
use core::u32;

use common::debouncer::Debouncer;
use cortex_m::{interrupt::Mutex, singleton};
use defmt::*;
use defmt_rtt as _;
use embedded_hal::digital::v2::InputPin;
use fixed::types::I1F15;
use fixed::types::U4F4;
use fixed::types::U8F8;
use fugit::Duration;
use fugit::HertzU32;
use panic_probe as _;
use pio_proc::pio_file;
use rp_pico::{
    entry,
    hal::{
        clocks::{Clock, ClockSource, ClocksManager, InitError},
        dma::{double_buffer, DMAExt},
        gpio::{self, FunctionPio0},
        multicore::{Multicore, Stack},
        pio::{Buffers, PIOBuilder, PIOExt, PinDir, ShiftDirection},
        pll::{common_configs::PLL_USB_48MHZ, setup_pll_blocking},
        sio::{Sio, SioFifo},
        timer::{Alarm, Alarm1},
        watchdog::Watchdog,
        xosc::setup_xosc_blocking,
        Timer,
    },
    pac,
};

use rytmos_engrave::{a, ais, b, c, cis, d, dis, e, f, fis, g, gis};
use rytmos_synth::commands::CommandMessage;
use rytmos_synth::synth::sine::SineSynth;
use rytmos_synth::synth::sine::SineSynthSettings;
use rytmos_synth::{
    commands::Command,
    synth::{
        sawtooth::{SawtoothSynth, SawtoothSynthSettings},
        Synth,
    },
};

use common::consts::*;
use common::plls;

static mut CORE1_STACK: Stack<4096> = Stack::new();

fn synth_core(sys_freq: u32) -> ! {
    let mut pac = unsafe { pac::Peripherals::steal() };
    let core = unsafe { pac::CorePeripherals::steal() };
    let mut sio = Sio::new(pac.SIO);
    let pins = gpio::Pins::new(
        pac.IO_BANK0,
        pac.PADS_BANK0,
        sio.gpio_bank0,
        &mut pac.RESETS,
    );
    let mut delay = cortex_m::delay::Delay::new(core.SYST, sys_freq);

    let i2s_sck_pin = pins.gpio8.into_function::<FunctionPio0>();
    let i2s_din_pin = pins.gpio9.into_function::<FunctionPio0>();
    let i2s_bck_pin = pins.gpio10.into_function::<FunctionPio0>();
    let i2s_lck_pin = pins.gpio11.into_function::<FunctionPio0>();

    let pio_i2s_mclk_output = pio_file!("src/i2s.pio", select_program("mclk_output")).program;
    let pio_i2s_send_master = pio_file!("src/i2s.pio", select_program("i2s_out_master")).program;

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

    info!("Start Synth core.");

    let sawtoonth_settings = SawtoothSynthSettings {
        decay: U8F8::from_num(0.9),
    };

    let mut synth = SawtoothSynth::new(0x0, sawtoonth_settings);
    let mut synth = SineSynth::new(
        0x1,
        SineSynthSettings {
            attack_gain: U4F4::from_num(1.0),
            initial_phase: I1F15::MIN,
            decay: 0.0,
        },
    );

    let mut sample = 0i16;

    let mut warned = false;

    // TODO: build a note scheduler that runs in this thread that can provide accurately timed notes.
    // Introduces new commands that address a scheduler and schedule other commands

    loop {
        sio.fifo
            .read()
            .and_then(Command::deserialize)
            .inspect(|&command| synth.run_command(command));

        if !warned && i2s_tx_transfer.is_done() {
            warn!("i2s transfer already done, probably late.");
            warned = true;
        }

        let (next_tx_buf, next_tx_transfer) = i2s_tx_transfer.wait();
        for (i, e) in next_tx_buf.iter_mut().enumerate() {
            if i % 2 == 0 {
                sample = synth.next().to_bits();
                *e = sample as u32 / 4;
            } else {
                *e = sample as u32 / 4;
            }
            *e <<= 16;
        }

        i2s_tx_transfer = next_tx_transfer.read_next(next_tx_buf);
    }
}

static FIFO: Mutex<RefCell<Option<SioFifo>>> = Mutex::new(RefCell::new(None));
static ALARM: Mutex<RefCell<Option<Alarm1>>> = Mutex::new(RefCell::new(None));

#[entry]
fn main() -> ! {
    let mut pac = pac::Peripherals::take().unwrap();
    let core = pac::CorePeripherals::take().unwrap();
    let mut watchdog = Watchdog::new(pac.WATCHDOG);
    let mut sio = Sio::new(pac.SIO);

    watchdog.enable_tick_generation((EXTERNAL_XTAL_FREQ_HZ.raw() / 1_000_000) as u8);

    let mut clocks = ClocksManager::new(pac.CLOCKS);

    common::setup_clocks!(pac, clocks);

    let mut _delay = cortex_m::delay::Delay::new(core.SYST, clocks.system_clock.freq().to_Hz());

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

    let mut timer = Timer::new(pac.TIMER, &mut pac.RESETS, &clocks);
    let mut alarm = timer.alarm_1().unwrap();

    cortex_m::interrupt::free(move |cs| {
        FIFO.borrow(cs).replace(Some(sio.fifo));

        alarm
            .schedule(Duration::<u32, 1, 1000000>::millis(1))
            .unwrap();
        alarm.enable_interrupt();

        ALARM.borrow(cs).replace(Some(alarm));
    });

    let c_pin = pins.gpio4.into_pull_up_input();
    let cis_pin = pins.gpio5.into_pull_up_input();
    let d_pin = pins.gpio6.into_pull_up_input();
    let dis_pin = pins.gpio7.into_pull_up_input();
    let e_pin = pins.gpio12.into_pull_up_input();
    let f_pin = pins.gpio13.into_pull_up_input();
    let fis_pin = pins.gpio14.into_pull_up_input();
    let g_pin = pins.gpio15.into_pull_up_input();
    let gis_pin = pins.gpio19.into_pull_up_input();
    let a_pin = pins.gpio18.into_pull_up_input();
    let ais_pin = pins.gpio17.into_pull_up_input();
    let b_pin = pins.gpio16.into_pull_up_input();

    let fn0_pin = pins.gpio0.into_pull_up_input();
    let fn1_pin = pins.gpio1.into_pull_up_input();
    let fn2_pin = pins.gpio2.into_pull_up_input();
    let fn3_pin = pins.gpio3.into_pull_up_input();

    const DEBOUNCE_TIME: u32 = 500;
    let mut fn0_debouncer = Debouncer::new(DEBOUNCE_TIME);
    let mut fn1_debouncer = Debouncer::new(DEBOUNCE_TIME);
    let mut fn2_debouncer = Debouncer::new(DEBOUNCE_TIME);
    let mut fn3_debouncer = Debouncer::new(DEBOUNCE_TIME);

    let mut c_debouncer = Debouncer::new(DEBOUNCE_TIME);
    let mut cis_debouncer = Debouncer::new(DEBOUNCE_TIME);
    let mut d_debouncer = Debouncer::new(DEBOUNCE_TIME);
    let mut dis_debouncer = Debouncer::new(DEBOUNCE_TIME);
    let mut e_debouncer = Debouncer::new(DEBOUNCE_TIME);
    let mut f_debouncer = Debouncer::new(DEBOUNCE_TIME);
    let mut fis_debouncer = Debouncer::new(DEBOUNCE_TIME);
    let mut g_debouncer = Debouncer::new(DEBOUNCE_TIME);
    let mut gis_debouncer = Debouncer::new(DEBOUNCE_TIME);
    let mut a_debouncer = Debouncer::new(DEBOUNCE_TIME);
    let mut ais_debouncer = Debouncer::new(DEBOUNCE_TIME);
    let mut b_debouncer = Debouncer::new(DEBOUNCE_TIME);

    info!("Start I/O thread.");

    let mut octave = 4;
    let attack_scaler: U4F4 = U4F4::from_num(1.2);
    let mut attack = U4F4::from_num(1.0);

    let mut button_states = [false; 12];

    loop {
        fn0_debouncer.update(fn0_pin.is_low().unwrap());
        fn1_debouncer.update(fn1_pin.is_low().unwrap());
        fn2_debouncer.update(fn2_pin.is_low().unwrap());
        fn3_debouncer.update(fn3_pin.is_low().unwrap());

        c_debouncer.update(c_pin.is_low().unwrap());
        cis_debouncer.update(cis_pin.is_low().unwrap());
        d_debouncer.update(d_pin.is_low().unwrap());
        dis_debouncer.update(dis_pin.is_low().unwrap());
        e_debouncer.update(e_pin.is_low().unwrap());
        f_debouncer.update(f_pin.is_low().unwrap());
        fis_debouncer.update(fis_pin.is_low().unwrap());
        g_debouncer.update(g_pin.is_low().unwrap());
        gis_debouncer.update(gis_pin.is_low().unwrap());
        a_debouncer.update(a_pin.is_low().unwrap());
        ais_debouncer.update(ais_pin.is_low().unwrap());
        b_debouncer.update(b_pin.is_low().unwrap());

        let new_button_states = [
            c_pin.is_low().unwrap(),
            cis_pin.is_low().unwrap(),
            d_pin.is_low().unwrap(),
            dis_pin.is_low().unwrap(),
            e_pin.is_low().unwrap(),
            f_pin.is_low().unwrap(),
            fis_pin.is_low().unwrap(),
            g_pin.is_low().unwrap(),
            gis_pin.is_low().unwrap(),
            a_pin.is_low().unwrap(),
            ais_pin.is_low().unwrap(),
            b_pin.is_low().unwrap(),
        ];

        let message = if new_button_states[NOTE_C] && !button_states[NOTE_C] {
            Some(CommandMessage::Play(c!(octave), U4F4::from_num(attack)))
        } else if new_button_states[NOTE_CIS] && !button_states[NOTE_CIS] {
            Some(CommandMessage::Play(cis!(octave), U4F4::from_num(attack)))
        } else if new_button_states[NOTE_D] && !button_states[NOTE_D] {
            Some(CommandMessage::Play(d!(octave), U4F4::from_num(attack)))
        } else if new_button_states[NOTE_DIS] && !button_states[NOTE_DIS] {
            Some(CommandMessage::Play(dis!(octave), U4F4::from_num(attack)))
        } else if new_button_states[NOTE_E] && !button_states[NOTE_E] {
            Some(CommandMessage::Play(e!(octave), U4F4::from_num(attack)))
        } else if new_button_states[NOTE_F] && !button_states[NOTE_F] {
            Some(CommandMessage::Play(f!(octave), U4F4::from_num(attack)))
        } else if new_button_states[NOTE_FIS] && !button_states[NOTE_FIS] {
            Some(CommandMessage::Play(fis!(octave), U4F4::from_num(attack)))
        } else if new_button_states[NOTE_G] && !button_states[NOTE_G] {
            Some(CommandMessage::Play(g!(octave), U4F4::from_num(attack)))
        } else if new_button_states[NOTE_GIS] && !button_states[NOTE_GIS] {
            Some(CommandMessage::Play(gis!(octave), U4F4::from_num(attack)))
        } else if new_button_states[NOTE_A] && !button_states[NOTE_A] {
            Some(CommandMessage::Play(a!(octave), U4F4::from_num(attack)))
        } else if new_button_states[NOTE_AIS] && !button_states[NOTE_AIS] {
            Some(CommandMessage::Play(ais!(octave), U4F4::from_num(attack)))
        } else if new_button_states[NOTE_B] && !button_states[NOTE_B] {
            Some(CommandMessage::Play(b!(octave), U4F4::from_num(attack)))
        } else if button_states.iter().take(12).any(|&b| b)
            && new_button_states.iter().take(12).all(|b| !b)
        {
            Some(CommandMessage::Play(c!(0), U4F4::from_num(0.0)))
        } else {
            None
        };

        if let Some(message) = message {
            let command = Command {
                address: 0x0,
                message,
            };
            let command_serialized = command.serialize();

            cortex_m::interrupt::free(|cs| {
                let mut fifo = FIFO.borrow(cs).take().unwrap();
                fifo.write(command_serialized);
                FIFO.borrow(cs).replace(Some(fifo));
            });
        }

        if let Ok(true) = fn0_debouncer.is_high() {
            octave = 4
        } else if let Ok(true) = fn2_debouncer.is_high() {
            octave = 2
        } else {
            octave = 3
        }

        if let Ok(true) = fn1_debouncer.is_high() {
            if cis_debouncer.stable_rising_edge() {
                let new_attack = attack.saturating_mul(attack_scaler);
                if new_attack == attack && new_attack != U4F4::MAX {
                    attack = U4F4::from_bits(attack.to_bits() + 1);
                } else {
                    attack = new_attack
                }
                info!("Attack: {}", attack.to_bits());
            }
            if c_debouncer.stable_rising_edge() {
                attack = attack
                    .saturating_div(attack_scaler)
                    .max(U4F4::from_bits(0b1));
                info!("Attack: {}", attack.to_bits());
            }
        }

        button_states = new_button_states;
    }
}

const NOTE_C: usize = 0;
const NOTE_CIS: usize = 1;
const NOTE_D: usize = 2;
const NOTE_DIS: usize = 3;
const NOTE_E: usize = 4;
const NOTE_F: usize = 5;
const NOTE_FIS: usize = 6;
const NOTE_G: usize = 7;
const NOTE_GIS: usize = 8;
const NOTE_A: usize = 9;
const NOTE_AIS: usize = 10;
const NOTE_B: usize = 11;
const FN_0: usize = 12;
const FN_1: usize = 13;
const FN_2: usize = 14;
const FN_3: usize = 15;

// TODO: use IO_IRQ_BANK0 to add interrupts on the pins with buttons to handle input so the main thread can be used for graphics.
