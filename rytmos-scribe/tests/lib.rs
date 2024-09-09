use heapless::Vec;
use rytmos_scribe::sixteen_switches::MeasureState;
use rytmos_scribe::sixteen_switches::RhythmDefinition;
use rytmos_scribe::sixteen_switches::StringState as Str;
use rytmos_scribe::sixteen_switches::SwitchState as S;

#[test]
fn test_play_definition_converion() {
    // TODO: test 2 mutes in a row
    #[rustfmt::skip]
    let test_states = [
        [S::Atck, S::Noop, S::Noop, S::Noop, S::Noop, S::Noop, S::Noop, S::Noop, S::Noop, S::Noop, S::Noop, S::Noop, S::Noop, S::Noop, S::Noop, S::Noop],
        [S::Atck, S::Noop, S::Noop, S::Noop, S::Mute, S::Noop, S::Atck, S::Noop, S::Atck, S::Noop, S::Mute, S::Noop, S::Atck, S::Atck, S::Atck, S::Atck],
    ];

    #[rustfmt::skip]
    let expects = [
        vec![Str::Ringing(16)],
        vec![Str::Ringing(4), Str::Silent(2), Str::Ringing(2), Str::Ringing(2), Str::Silent(2), Str::Ringing(1), Str::Ringing(1), Str::Ringing(1), Str::Ringing(1)]
    ];

    for (state, expect) in test_states.into_iter().zip(expects) {
        let playdef_actual: RhythmDefinition =
            RhythmDefinition::try_from(MeasureState::new(state)).unwrap();
        let playdef_expect: RhythmDefinition =
            RhythmDefinition::new(Vec::from_iter(expect.into_iter())).unwrap();
        assert_eq!(playdef_actual, playdef_expect);
    }
}

// TODO: test the rytmos conversion
