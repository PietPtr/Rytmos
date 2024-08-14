use rytmos_scribe::sixteen_switches::MeasureState;
use rytmos_scribe::sixteen_switches::PlayDefinition;
use rytmos_scribe::sixteen_switches::StringState as Str;
use rytmos_scribe::sixteen_switches::SwitchState as S;
use test_case::case;

// #[test_case()]
#[test]
fn test_play_definition_converion() {
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
        let playdef_actual: PlayDefinition = PlayDefinition::from(MeasureState::new(state));
        let playdef_expect: PlayDefinition = PlayDefinition::new(expect);
        assert_eq!(playdef_actual, playdef_expect);
    }
}

// TODO: test the rytmos conversion
