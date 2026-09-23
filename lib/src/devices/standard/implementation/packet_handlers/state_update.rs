use nom::{combinator::all_consuming, error::VerboseError};

use crate::devices::standard::{
    packets::inbound::{state_update_packet::StateUpdatePacket, InboundPacket},
    state::DeviceState,
};

pub fn state_update_handler(input: &[u8], state: DeviceState) -> DeviceState {
    let result: Result<_, nom::Err<VerboseError<_>>> =
        all_consuming(StateUpdatePacket::take)(input);
    let packet = match result {
        Ok((_, packet)) => packet,
        Err(err) => {
            tracing::error!("failed to parse packet: {err:?}");
            return state;
        }
    };

    DeviceState {
        device_features: state.device_features,
        tws_status: packet.tws_status,
        battery: state.battery,
        equalizer_configuration: state.equalizer_configuration.to_owned(),
        age_range: packet.age_range.or(state.age_range),
        gender: packet.gender.or(state.gender),
        button_configuration: packet.button_configuration.or(state.button_configuration),
        hear_id: packet.hear_id.to_owned().or(state.hear_id.to_owned()),
        firmware_version: packet
            .firmware_version
            .as_ref()
            .or(state.firmware_version.as_ref())
            .cloned(),
        serial_number: packet
            .serial_number
            .as_ref()
            .or(state.serial_number.as_ref())
            .cloned(),
        sound_modes: packet.sound_modes.or(state.sound_modes),
        sound_modes_type_two: packet.sound_modes_type_two.or(state.sound_modes_type_two),
        ambient_sound_mode_cycle: packet
            .ambient_sound_mode_cycle
            .or(state.ambient_sound_mode_cycle),
        sound_modes_type_three: packet
            .sound_modes_type_three
            .or(state.sound_modes_type_three),
        gaming_mode: packet.gaming_mode.or(state.gaming_mode),
        surround_sound: packet.surround_sound.or(state.surround_sound),
        dual_connections: packet.dual_connections.or(state.dual_connections),
        low_battery_prompt: packet.low_battery_prompt.or(state.low_battery_prompt),
    }
}
