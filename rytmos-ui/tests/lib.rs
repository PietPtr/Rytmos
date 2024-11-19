use std::sync::Once;

use fixed::types::U4F4;
use rytmos_engrave::{
    a, cis, dis,
    staff::{Duration, Music},
};
use rytmos_synth::commands::CommandMessage;
use rytmos_ui::synth_controller::{SynthController, SynthControllerSettings};

static INIT: Once = Once::new();

fn init_logger() {
    INIT.call_once(|| {
        env_logger::init();
    });
}

#[test]
fn test_command_for_time() {
    init_logger();

    let settings = SynthControllerSettings {
        play_pattern: true,
        measures_silence: 0,
        metronome: false,
    };

    let music_sequence = vec![
        Music::Note(a!(3), Duration::Quarter),
        Music::Rest(Duration::Quarter),
        Music::Note(cis!(4), Duration::Eighth),
        Music::Tie,
        Music::Note(cis!(4), Duration::Eighth),
        Music::Note(dis!(4), Duration::Eighth),
    ];

    let mut player = SynthController::new(settings);
    player.set_music(heapless::Vec::from_iter(music_sequence));

    // Test different time values
    let _test_cases = vec![
        (0, Some(CommandMessage::Play(a!(3), U4F4::from_num(1.)))),
        (1, None),
        (2, None),
        (3, None),
        (4, None),
        (5, None),
        (6, None),
        (7, None),
        (32, Some(CommandMessage::Play(cis!(4), U4F4::from_num(1.)))),
        (33, None),
        (40, None),
        (45, None),
        (48, Some(CommandMessage::Play(dis!(4), U4F4::from_num(1.)))),
    ];

    println!("----");
    for t in 0..129 {
        let result = player.next_command();
        if let Some(v) = result.first() {
            println!("{t}: {v:?}");
        }
    }

    // println!("----");
    // for (t, expected_command) in test_cases {
    //     let result = player.next_command();
    //     println!("{t}: {:?}", result);
    //     match expected_command {
    //         Some(expected_cmd) => {
    //             assert_eq!(result.len(), 1);
    //             assert_eq!(result[0], expected_cmd);
    //         }
    //         None => {
    //             assert!(result.is_empty());
    //         }
    //     }
    // }
}
