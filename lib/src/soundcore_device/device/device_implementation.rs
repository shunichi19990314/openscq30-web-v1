use std::collections::HashMap;

use crate::devices::standard::{
    state::DeviceState,
    structures::{
        AmbientSoundModeCycle, Command, EqualizerConfiguration, HearId, MultiButtonConfiguration,
        SoundModes, SoundModesTypeTwo, SoundModesTypeThree,
    },
};

use super::soundcore_command::CommandResponse;

pub trait DeviceImplementation {
    fn packet_handlers(
        &self,
    ) -> HashMap<Command, Box<dyn Fn(&[u8], DeviceState) -> DeviceState + Send + Sync>>;

    fn initialize(&self, packet: &[u8]) -> crate::Result<DeviceState>;

    fn set_sound_modes(
        &self,
        state: DeviceState,
        sound_modes: SoundModes,
    ) -> crate::Result<CommandResponse>;

    fn set_sound_modes_type_two(
        &self,
        state: DeviceState,
        sound_modes: SoundModesTypeTwo,
    ) -> crate::Result<CommandResponse>;

    fn set_sound_modes_type_three(
        &self,
        _state: DeviceState,
        _sound_modes: SoundModesTypeThree,
    ) -> crate::Result<CommandResponse> {
        Err(crate::Error::FeatureNotSupported {
            feature_name: "sound modes type three",
        })
    }

    fn set_gaming_mode(
        &self,
        _state: DeviceState,
        _enabled: bool,
    ) -> crate::Result<CommandResponse> {
        Err(crate::Error::FeatureNotSupported {
            feature_name: "gaming mode",
        })
    }

    fn set_surround_sound(
        &self,
        _state: DeviceState,
        _enabled: bool,
    ) -> crate::Result<CommandResponse> {
        Err(crate::Error::FeatureNotSupported {
            feature_name: "surround sound",
        })
    }

    fn set_low_battery_prompt(
        &self,
        _state: DeviceState,
        _enabled: bool,
    ) -> crate::Result<CommandResponse> {
        Err(crate::Error::FeatureNotSupported {
            feature_name: "low battery prompt",
        })
    }

    fn set_ambient_sound_mode_cycle(
        &self,
        state: DeviceState,
        cycle: AmbientSoundModeCycle,
    ) -> crate::Result<CommandResponse>;

    fn set_equalizer_configuration(
        &self,
        state: DeviceState,
        equalizer_configuration: EqualizerConfiguration,
    ) -> crate::Result<CommandResponse>;

    fn set_hear_id(&self, state: DeviceState, hear_id: HearId) -> crate::Result<CommandResponse>;

    fn set_multi_button_configuration(
        &self,
        state: DeviceState,
        button_configuration: MultiButtonConfiguration,
    ) -> crate::Result<CommandResponse>;
}
