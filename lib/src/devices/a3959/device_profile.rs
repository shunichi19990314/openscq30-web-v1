use std::sync::{Arc, Mutex};

use nom::error::VerboseError;

use crate::{
    device_profile::{DeviceFeatures, DeviceProfile},
    devices::{
        a3959::packets::{
            A3959StateUpdatePacket, SetEqualizerMonoPreservedDrcPacket,
            SetSoundModeTypeThreePacket,
        },
        standard::{
            self,
            packets::inbound::{state_update_packet::StateUpdatePacket, InboundPacket},
            state::DeviceState,
            structures::{
                AmbientSoundModeCycle, Command, EqualizerConfiguration, HearId,
                MultiButtonConfiguration, SoundModes, SoundModesTypeTwo, SoundModesTypeThree,
                STATE_UPDATE,
            },
            packets::outbound::SetFlagPacket,
        },
    },
    soundcore_device::{
        device::{device_implementation::DeviceImplementation, soundcore_command::CommandResponse},
        device_model::DeviceModel,
    },
};

/// command bytes match the v2 `SET_*_COMMAND` constants
const SET_GAMING_MODE_COMMAND: Command = Command::new([0x08, 0xee, 0x00, 0x00, 0x00, 0x01, 0x87]);
const SET_SURROUND_SOUND_COMMAND: Command = Command::new([0x08, 0xee, 0x00, 0x00, 0x00, 0x02, 0x86]);
const SET_LOW_BATTERY_PROMPT_COMMAND: Command =
    Command::new([0x08, 0xee, 0x00, 0x00, 0x00, 0x10, 0x82]);

fn set_flag(
    state: DeviceState,
    enabled: bool,
    command: Command,
    apply: impl Fn(&mut DeviceState, bool),
) -> crate::Result<CommandResponse> {
    let packet = SetFlagPacket { command, enabled };
    let mut new_state = state;
    apply(&mut new_state, enabled);
    Ok(CommandResponse {
        packets: vec![packet.into()],
        new_state,
    })
}

pub(crate) const A3959_DEVICE_PROFILE: DeviceProfile = DeviceProfile {
    features: DeviceFeatures {
        available_sound_modes: None,
        has_hear_id: false,
        num_equalizer_channels: 1,
        num_equalizer_bands: 10,
        has_dynamic_range_compression: true,
        // TODO(phase 2): button configuration needs a per-button set packet ([0x04, 0x81]) and
        // triple press slots in the UI; the raw bytes are already parsed and preserved.
        has_button_configuration: false,
        has_wear_detection: false,
        has_touch_tone: false,
        has_auto_power_off: false,
        has_ambient_sound_mode_cycle: true,
        dynamic_range_compression_min_firmware_version: None,
        has_gaming_mode: true,
        has_surround_sound: true,
        has_dual_connections: true,
        has_low_battery_prompt: true,
    },
    compatible_models: &[DeviceModel::A3959],
    implementation: || Arc::new(A3959Implementation::default()),
};

#[derive(Default)]
pub(crate) struct A3959Implementation {
    /// dynamic range compression bytes received from the device, resent verbatim when the
    /// equalizer is set (the meaning of the block is not fully understood yet)
    drc: Arc<Mutex<[u8; 10]>>,
}

impl DeviceImplementation for A3959Implementation {
    fn packet_handlers(
        &self,
    ) -> std::collections::HashMap<
        Command,
        Box<dyn Fn(&[u8], DeviceState) -> DeviceState + Send + Sync>,
    > {
        let drc = self.drc.to_owned();
        let mut handlers = standard::implementation::packet_handlers();

        handlers.insert(
            STATE_UPDATE,
            Box::new(move |packet_bytes, state| {
                let packet = match A3959StateUpdatePacket::take::<VerboseError<_>>(packet_bytes) {
                    Ok((_, packet)) => packet,
                    Err(err) => {
                        tracing::error!("failed to parse packet: {err:?}");
                        return state;
                    }
                };
                *drc.lock().expect("drc mutex poisoned") = packet.drc_preserved;
                StateUpdatePacket::from(packet).into()
            }),
        );

        handlers
    }

    fn initialize(&self, packet: &[u8]) -> crate::Result<DeviceState> {
        let packet = A3959StateUpdatePacket::take::<VerboseError<_>>(packet)
            .map(|(_, packet)| packet)
            .map_err(|err| crate::Error::ParseError {
                message: format!("{err:?}"),
            })?;
        *self.drc.lock().expect("drc mutex poisoned") = packet.drc_preserved;
        Ok(StateUpdatePacket::from(packet).into())
    }

    fn set_sound_modes(
        &self,
        _state: DeviceState,
        _sound_modes: SoundModes,
    ) -> crate::Result<CommandResponse> {
        Err(crate::Error::FeatureNotSupported {
            feature_name: "sound modes",
        })
    }

    fn set_sound_modes_type_two(
        &self,
        _state: DeviceState,
        _sound_modes: SoundModesTypeTwo,
    ) -> crate::Result<CommandResponse> {
        Err(crate::Error::FeatureNotSupported {
            feature_name: "sound modes type two",
        })
    }

    fn set_sound_modes_type_three(
        &self,
        state: DeviceState,
        sound_modes: SoundModesTypeThree,
    ) -> crate::Result<CommandResponse> {
        let packet = SetSoundModeTypeThreePacket { sound_modes };
        Ok(CommandResponse {
            packets: vec![packet.into()],
            new_state: DeviceState {
                sound_modes_type_three: Some(sound_modes),
                ..state
            },
        })
    }

    fn set_ambient_sound_mode_cycle(
        &self,
        state: DeviceState,
        cycle: AmbientSoundModeCycle,
    ) -> crate::Result<CommandResponse> {
        standard::implementation::set_ambient_sound_mode_cycle(state, cycle)
    }

    fn set_gaming_mode(
        &self,
        state: DeviceState,
        enabled: bool,
    ) -> crate::Result<CommandResponse> {
        set_flag(state, enabled, SET_GAMING_MODE_COMMAND, |state, v| {
            state.gaming_mode = Some(v);
        })
    }

    fn set_surround_sound(
        &self,
        state: DeviceState,
        enabled: bool,
    ) -> crate::Result<CommandResponse> {
        set_flag(state, enabled, SET_SURROUND_SOUND_COMMAND, |state, v| {
            state.surround_sound = Some(v);
        })
    }

    fn set_low_battery_prompt(
        &self,
        state: DeviceState,
        enabled: bool,
    ) -> crate::Result<CommandResponse> {
        set_flag(state, enabled, SET_LOW_BATTERY_PROMPT_COMMAND, |state, v| {
            state.low_battery_prompt = Some(v);
        })
    }

    fn set_equalizer_configuration(
        &self,
        state: DeviceState,
        equalizer_configuration: EqualizerConfiguration,
    ) -> crate::Result<CommandResponse> {
        let preserved_drc = *self.drc.lock().expect("drc mutex poisoned");
        let packet = SetEqualizerMonoPreservedDrcPacket {
            configuration: &equalizer_configuration,
            preserved_drc: &preserved_drc,
        };
        Ok(CommandResponse {
            packets: vec![packet.into()],
            new_state: DeviceState {
                equalizer_configuration,
                ..state
            },
        })
    }

    fn set_hear_id(
        &self,
        _state: DeviceState,
        _hear_id: HearId,
    ) -> crate::Result<CommandResponse> {
        Err(crate::Error::FeatureNotSupported {
            feature_name: "hear id",
        })
    }

    fn set_multi_button_configuration(
        &self,
        _state: DeviceState,
        _button_configuration: MultiButtonConfiguration,
    ) -> crate::Result<CommandResponse> {
        Err(crate::Error::FeatureNotSupported {
            feature_name: "custom button actions",
        })
    }
}
