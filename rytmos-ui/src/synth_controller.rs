use heapless::Vec;
use rytmos_engrave::{c, staff::Music};
use rytmos_synth::commands::Command;

#[derive(Debug, Default, Clone, Copy)]
pub struct SynthControllerSettings {
    pub play_pattern: bool,
    pub measures_silence: u8,
    pub metronome: Option<u8>,
}

#[derive(Debug, Default, Clone, Copy)]
pub struct SynthControllerSettingsUpdate {
    pub play_pattern: Option<bool>,
    pub measures_silence: Option<u8>,
    pub metronome: Option<Option<u8>>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SynthControllerState {
    Playing,
    Stopped,
}

pub struct SynthController {
    settings: SynthControllerSettings,
    music: Vec<Music, 16>,
    time: u64,
    state: SynthControllerState,
}

/// Given a certain music definition, translates that music to commands for a synth.
impl SynthController {
    pub fn new(settings: SynthControllerSettings) -> Self {
        Self {
            settings,
            music: Vec::new(),
            time: 0,
            state: SynthControllerState::Stopped,
        }
    }

    pub fn set_music(&mut self, music: Vec<Music, 16>) {
        self.music = music;
    }

    /// Updates the internal settings of the synth controller and returns commands for the synth
    /// to reflect those settings.
    pub fn update_settings(&mut self, settings: SynthControllerSettingsUpdate) -> Vec<Command, 4> {
        let new_settings = SynthControllerSettings {
            play_pattern: settings.play_pattern.unwrap_or(self.settings.play_pattern),
            measures_silence: settings
                .measures_silence
                .unwrap_or(self.settings.measures_silence),
            metronome: settings.metronome.unwrap_or(self.settings.metronome),
        };

        self.settings = new_settings;

        Vec::new()
    }

    pub fn play_or_stop_toggle(&mut self) {
        self.state = match self.state {
            SynthControllerState::Playing => SynthControllerState::Stopped,
            SynthControllerState::Stopped => SynthControllerState::Playing,
        };

        if self.state == SynthControllerState::Stopped {
            self.time = 0;
        }
    }

    pub fn state(&self) -> SynthControllerState {
        self.state
    }

    pub fn next_command(&mut self) -> Vec<Command, 4> {
        let commands = self.command_for_time(self.time);

        self.time += 1;

        commands
    }

    pub fn command_for_time(&self, t: u64) -> Vec<Command, 4> {
        // TODO: play metronome tick?
        if !self.settings.play_pattern {
            return Vec::new();
        }

        // Current time indexed in sixteenths, looping over the measure we're playing
        let t16 = (t % ((self.settings.measures_silence as u64 + 1) * 128)) as f64 / 4.;
        let mut count16 = 0;

        let mut commands = Vec::new();
        let mut last_was_tie = false;

        for &music in self.music.iter() {
            match music {
                Music::Note(note, dur) => {
                    if t16 == count16 as f64 && !last_was_tie {
                        commands.push(Command::Play(note, 255, 1)).unwrap();
                        break;
                    }
                    count16 += (dur.value() * 4.) as u64;
                    last_was_tie = false;
                }
                Music::Rest(dur) => {
                    if t16 == count16 as f64 && !last_was_tie {
                        commands.push(Command::Play(c!(0), 0, 0)).unwrap();
                        break;
                    }
                    count16 += (dur.value() as u64) * 4;
                    last_was_tie = false;
                }
                Music::Tie => last_was_tie = true,
            }
        }

        commands
    }
}
