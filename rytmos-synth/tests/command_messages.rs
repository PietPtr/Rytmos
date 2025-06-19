use rand::Rng;
use rytmos_synth::commands::Command;

#[test]
fn test_command_serdes() {
    let mut rng = rand::thread_rng();

    let mut valid_commands = 0;

    let mut passed = true;

    for i in 0..10000000 {
        let mut value: u32 = rng.gen();
        let command_id = rng.gen_range(0..8) & 0b111111;

        value &= 0b11110000_00111111_11111111_11111111;
        value |= command_id << 22;

        if let Some(cmd) = Command::deserialize(value) {
            valid_commands += 1;
            let serialized = cmd.serialize();
            if value != serialized {
                println!(
                    "Failed serdes test #{i}: {:#?} VS {:#?} => \n{:032b} =/=\n{:032b}",
                    cmd,
                    Command::deserialize(serialized),
                    value,
                    serialized,
                );
                passed = false;
            }
        }
    }

    assert!(passed);

    println!("Serialized {} valid commands.", valid_commands);
    assert!(valid_commands > 0);
}
