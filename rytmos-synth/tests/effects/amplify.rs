use fixed::types::{I1F15, U8F8};
use rytmos_synth::effect::{
    amplify::{Amplify, AmplifySettings},
    Effect,
};

#[test]
fn test_amplify() {
    let mut effect = Amplify::make(
        0,
        AmplifySettings {
            amplification: U8F8::from_num(20),
        },
    );

    assert_eq!(effect.next(I1F15::from_num(0.5)), I1F15::MAX);
    assert_eq!(effect.next(I1F15::from_num(-0.5)), I1F15::NEG_ONE);
    assert_eq!(
        effect.next(I1F15::from_num(0.001)),
        I1F15::from_num(0.02014)
    );
    assert_eq!(
        effect.next(I1F15::from_num(-0.001)),
        I1F15::from_num(-0.02014)
    );
}
