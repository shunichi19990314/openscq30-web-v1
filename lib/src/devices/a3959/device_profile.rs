use std::sync::{Arc, Mutex};

use nom::error::VerboseError;

use crate::{
    device_profile::{DeviceFeatures, DeviceProfile},
    devices::{
        a3959::packets::{
            A3959StateUpdatePacket, SetButtonActionPacket, SetEqualizerMonoPreservedDrcPacket,
            SetSoundModeTypeThreePacket,
        },
        standard::{
            self,
            packets::inbound::{state_update_packet::StateUpdatePacket, InboundPacket},
            state::DeviceState,
            structures::{
                AmbientSoundModeCycle, ButtonAction, ButtonConfiguration, Command,
                EqualizerConfiguration, HearId, MultiButtonConfiguration, SoundModes,
                SoundModesTypeTwo, SoundModesTypeThree, STATE_UPDATE,
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

#[cfg(test)]
mod tests {
    use crate::devices::a3959::packets::REAL_DEVICE_STATE_UPDATE;
    use crate::devices::standard::packets::outbound::OutboundPacketBytesExt;
    use crate::devices::standard::structures::{ButtonAction, ButtonConfiguration};

    use super::*;

    #[test]
    fn it_sends_per_button_packets_and_preserves_the_disconnected_nibble() {
        let implementation = A3959Implementation::default();
        let state = implementation
            .initialize(REAL_DEVICE_STATE_UPDATE)
            .expect("should initialize");

        let mut wanted = state.button_configuration.clone().expect("buttons");
        // left double press: 0x63 (connected=NextSong, disconnected=PlayPause) -> PlayPause
        wanted.left_double_click = ButtonConfiguration {
            action: ButtonAction::PlayPause,
            is_enabled: true,
        };
        // unchanged slots must not produce packets
        let response = implementation
            .set_multi_button_configuration(state, wanted)
            .expect("should set");
        assert_eq!(response.packets.len(), 1);
        let bytes = SetButtonActionPacket {
            order_index: 2,
            button_id: 0,
            // disconnected nibble 6 preserved, connected nibble becomes 6 (PlayPause)
            action_byte: 0x66,
        }
        .bytes();
        assert_eq!(bytes, response.packets[0].bytes());
    }

    #[test]
    fn it_disables_a_button_with_0xff() {
        let implementation = A3959Implementation::default();
        let state = implementation
            .initialize(REAL_DEVICE_STATE_UPDATE)
            .expect("should initialize");
        let mut wanted = state.button_configuration.clone().expect("buttons");
        wanted.left_long_press = ButtonConfiguration {
            action: ButtonAction::AmbientSoundMode,
            is_enabled: false,
        };
        let response = implementation
            .set_multi_button_configuration(state, wanted)
            .expect("should set");
        assert_eq!(response.packets.len(), 1);
        let bytes = SetButtonActionPacket {
            order_index: 6,
            button_id: 1,
            action_byte: 0xFF,
        }
        .bytes();
        assert_eq!(bytes, response.packets[0].bytes());
    }
}

pub(crate) const A3959_DEVICE_PROFILE: DeviceProfile = DeviceProfile {
    features: DeviceFeatures {
        available_sound_modes: None,
        has_hear_id: false,
        num_equalizer_channels: 1,
        num_equalizer_bands: 10,
        has_dynamic_range_compression: true,
        // per-button set packets ([0x04, 0x81]); the v1 UI exposes the six
        // single/double/long press slots, triple press stays on the device
        has_button_configuration: true,
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
    /// raw button action bytes (8 slots), used to preserve the disconnected-state nibble
    /// and the triple press slots when setting one of the six UI-editable slots
    buttons_raw: Arc<Mutex<[u8; 8]>>,
}

impl DeviceImplementation for A3959Implementation {
    fn packet_handlers(
        &self,
    ) -> std::collections::HashMap<
        Command,
        Box<dyn Fn(&[u8], DeviceState) -> DeviceState + Send + Sync>,
    > {
        let drc = self.drc.to_owned();
        let buttons_raw = self.buttons_raw.to_owned();
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
                *buttons_raw.lock().expect("buttons mutex poisoned") = packet.buttons_raw;
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
        *self.buttons_raw.lock().expect("buttons mutex poisoned") = packet.buttons_raw;
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
        // the web UI re-synchronizes the equalizer shortly after connecting; resending
        // an identical configuration is unnecessary and suspected of making some
        // firmwares drop the connection, so skip the write in that case
        if state.equalizer_configuration == equalizer_configuration {
            return Ok(CommandResponse {
                packets: vec![],
                new_state: state,
            });
        }
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
        state: DeviceState,
        button_configuration: MultiButtonConfiguration,
    ) -> crate::Result<CommandResponse> {
        // (order index, button id, slot accessor) for the six UI-editable slots
        let slots: [(u8, u8, fn(&MultiButtonConfiguration) -> ButtonConfiguration); 6] = [
            (0, 2, |c| c.left_single_click),
            (1, 2, |c| c.right_single_click),
            (2, 0, |c| c.left_double_click),
            (3, 0, |c| c.right_double_click),
            (6, 1, |c| c.left_long_press),
            (7, 1, |c| c.right_long_press),
        ];
        let mut raw = self.buttons_raw.lock().expect("buttons mutex poisoned");
        let mut packets = Vec::new();
        for (order_index, button_id, get) in slots {
            let wanted = get(&button_configuration);
            let current_raw = raw[order_index as usize];
            let current = {
                let connected = current_raw & 0xF;
                ButtonConfiguration {
                    action: ButtonAction::from_repr(connected & 0x0F).unwrap_or_default(),
                    is_enabled: connected != 0xF,
                }
            };
            if current == wanted {
                continue;
            }
            let action_byte = if !wanted.is_enabled {
                0xFF
            } else {
                let action_id: u8 = wanted.action.into();
                // preserve the disconnected-state nibble of the device unless the slot
                // was fully disabled before
                let disconnected = if current_raw == 0xFF {
                    action_id
                } else {
                    current_raw >> 4
                };
                (disconnected << 4) | action_id
            };
            raw[order_index as usize] = action_byte;
            packets.push(
                SetButtonActionPacket {
                    order_index,
                    button_id,
                    action_byte,
                }
                .into(),
            );
        }
        let new_configuration = MultiButtonConfiguration {
            left_single_click: button_configuration.left_single_click,
            right_single_click: button_configuration.right_single_click,
            left_double_click: button_configuration.left_double_click,
            right_double_click: button_configuration.right_double_click,
            left_long_press: button_configuration.left_long_press,
            right_long_press: button_configuration.right_long_press,
        };
        Ok(CommandResponse {
            packets,
            new_state: DeviceState {
                button_configuration: Some(new_configuration),
                ..state
            },
        })
    }
}
