// // Generic synth that can be reconfigured at runtime using commands.

// use fixed::types::{I1F15, U4F4};
// use rytmos_engrave::staff::Note;

// use crate::commands::Command;

// use super::{
//     metronome::MetronomeSynth,
//     overtone::{OvertoneSynth, OvertoneSynthSettings},
//     sine::SineSynthSettings,
//     vibrato::{VibratoSynth, VibratoSynthSettings},
//     Synth,
// };

// /// Contains a simple overtone synth defined at construction and a metronome.
// /// Handles all commands
// pub struct OvertoneAndMetronomeSynth {
//     address: u32,
//     synth: OvertoneSynth<VibratoSynth, 4>,
//     metronome: MetronomeSynth,
// }

// impl OvertoneAndMetronomeSynth {
//     pub fn new(address: u32) -> Self {
//         let synths = [
//             VibratoSynth::new(
//                 address,
//                 VibratoSynthSettings {
//                     sine_settings: SineSynthSettings {
//                         attack_gain: U4F4::from_num(0.38),
//                         initial_phase: todo!(),
//                         decay_per_second: 0.5,
//                     },
//                     vibrato_frequency: 10.,
//                     vibrato_strength: 0.0001,
//                 },
//             ),
//             VibratoSynth::new(
//                 address,
//                 VibratoSynthSettings {
//                     sine_settings: SineSynthSettings {
//                         attack_gain: U4F4::from_num(0.4),
//                         initial_phase: todo!(),
//                         decay_per_second: 0.6,
//                     },
//                     vibrato_frequency: 10.,
//                     vibrato_strength: 0.0001,
//                 },
//             ),
//             VibratoSynth::new(
//                 address,
//                 VibratoSynthSettings {
//                     sine_settings: SineSynthSettings {
//                         attack_gain: U4F4::from_num(0.34),
//                         initial_phase: todo!(),
//                         decay_per_second: 0.5,
//                     },
//                     vibrato_frequency: 10.,
//                     vibrato_strength: 0.0001,
//                 },
//             ),
//             VibratoSynth::new(
//                 address,
//                 VibratoSynthSettings {
//                     sine_settings: SineSynthSettings {
//                         attack_gain: U4F4::from_num(0.02),
//                         initial_phase: todo!(),
//                         decay_per_second: 0.4,
//                     },
//                     vibrato_frequency: 10.,
//                     vibrato_strength: 0.0001,
//                 },
//             ),
//         ];

//         let synth = OvertoneSynth::new(address, OvertoneSynthSettings {}, synths);

//         Self {
//             address,
//             synth,
//             metronome: MetronomeSynth::new(address),
//         }
//     }
// }

// impl Synth for OvertoneAndMetronomeSynth {
//     type Settings = ();

//     // TODO: configure is useless as synths run in a different thread from menus.
//     // Settings can only be changed through commands
//     fn configure(&mut self, _settings: Self::Settings) {}

//     fn play(&mut self, note: Note, velocity: U4F4) {
//         self.synth.play(note, velocity);
//     }

//     fn next(&mut self) -> I1F15 {
//         let synth = self.synth.next();
//         let metronome = self.metronome.next();
//         synth.wrapping_add(metronome)
//     }

//     fn run_command(&mut self, command: Command) {
//         super::run_play_command(self, command);

//         todo!()

//         // match command {
//         //     Command::Play(_, _, _) => (),
//         //     Command::SetAttack(_, _) => todo!(),
//         //     Command::Tick(emphasis) => {
//         //         let note = if emphasis {
//         //             rytmos_engrave::a!(0)
//         //         } else {
//         //             rytmos_engrave::b!(0)
//         //         };
//         //         self.metronome.play(note, 1.0);
//         //     }
//         //     Command::SetTempo(_) => todo!(),
//         // }
//     }

//     fn address(&self) -> u32 {
//         self.address
//     }
// }
