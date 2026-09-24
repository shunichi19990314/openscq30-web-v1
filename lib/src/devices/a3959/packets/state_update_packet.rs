use nom::{
    bytes::complete::take,
    combinator::all_consuming,
    error::{context, ContextError, ParseError},
    number::complete::le_u8,
    sequence::tuple,
    IResult,
};

use crate::devices::{
    a3959::device_profile::A3959_DEVICE_PROFILE,
    standard::{
        packets::{
            inbound::{state_update_packet::StateUpdatePacket, InboundPacket},
            parsing::take_bool,
        },
        structures::{
            AmbientSoundModeCycle, BatteryLevel, ButtonAction, ButtonConfiguration, DualBattery,
            EqualizerConfiguration, FirmwareVersion, IsBatteryCharging, MultiButtonConfiguration,
            SerialNumber, SingleBattery, SoundModesTypeThree, TwsStatus,
        },
    },
};

/// State update packet of the A3959 (Soundcore P30i / R50i NC).
///
/// Byte map of the body (offsets from 0), ported from the v2 implementation
/// (`devices/soundcore/a3959/packets/inbound/state_update.rs`) and
/// `tools/soundcore-device-faker/devices/a3959.toml`:
///
/// | off  | size | field |
/// |------|------|-------|
/// | 0-1  | 2    | tws status (host device, is connected) |
/// | 2-3  | 2    | battery left/right, 0-10 scale |
/// | 4-5  | 2    | unknown (0xff 0xff) |
/// | 6-15 | 10   | left firmware version (ascii) |
/// | 16-25| 10   | right firmware version (ascii) |
/// | 26-41| 16   | serial number (ascii) |
/// | 42-53| 12   | equalizer: profile id (le u16) + 10 bands (byte = value + 120) |
/// | 54-63| 10   | dynamic range compression block (meaning unknown, preserved) |
/// | 64   | 1    | unknown (must be preserved when sending) |
/// | 65-72| 8    | button actions (left/right x single/double/triple/long) |
/// | 73   | 1    | ambient sound mode cycle |
/// | 74-80| 7    | sound modes type three |
/// | 81   | 1    | unknown |
/// | 82   | 1    | touch tone |
/// | 83   | 1    | dual connections (multipoint) enabled |
/// | 84   | 1    | surround sound |
/// | 85-86| 2    | auto power off enabled + duration |
/// | 87   | 1    | low battery prompt |
/// | 88   | 1    | gaming mode (firmware >= 01.60 only) |
/// | 89-  | 12   | unknown |
/// state update captured from a real Soundcore P30i (2026-09-23, firmware 01.44,
/// ambient sound mode = transparency, noise canceling off)
#[cfg(test)]
pub(crate) const REAL_DEVICE_STATE_UPDATE: &[u8] = &[
    0x00, 0x01, 0x09, 0x09, 0xff, 0xff, 0x30, 0x31, 0x2e, 0x34, 0x34, 0x30, 0x31, 0x2e, 0x34,
    0x34, 0x33, 0x39, 0x35, 0x39, 0x39, 0x43, 0x32, 0x43, 0x33, 0x31, 0x33, 0x39, 0x43, 0x31,
    0x41, 0x34, 0x00, 0x00, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x0a, 0xff, 0xff, 0x63, 0x66, 0xff,
    0xff, 0x44, 0x44, 0x33, 0x01, 0x55, 0x00, 0x00, 0x00, 0xff, 0x00, 0x36, 0x01, 0x01, 0x00,
    0x01, 0x02, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00,
];


#[derive(Debug, Clone, PartialEq)]


pub struct A3959StateUpdatePacket {
    pub tws_status: TwsStatus,
    /// raw 0-10 battery scale
    pub left_battery: u8,
    pub right_battery: u8,
    pub firmware_version_left: FirmwareVersion,
    pub firmware_version_right: FirmwareVersion,
    pub serial_number: SerialNumber,
    pub equalizer_configuration: EqualizerConfiguration,
    /// dynamic range compression block, preserved verbatim when setting the equalizer
    pub drc_preserved: [u8; 10],
    /// unknown byte at offset 64, preserved verbatim
    pub preserve_byte: u8,
    /// raw button action bytes (left/right x single/double/triple/long)
    pub buttons_raw: [u8; 8],
    pub ambient_sound_mode_cycle: AmbientSoundModeCycle,
    pub sound_modes: SoundModesTypeThree,
    pub dual_connections: bool,
    pub surround_sound: bool,
    pub low_battery_prompt: bool,
    pub gaming_mode: bool,
}

/// v2 displays the raw 0-10 scale; v1's battery levels are percentages.
fn to_percentage(raw: u8) -> u8 {
    (raw.saturating_mul(10)).min(100)
}
/// action id of one nibble; 15 means disabled
fn nibble_to_action(nibble: u8) -> Option<ButtonAction> {
    match nibble {
        0xF => None,
        other => Some(ButtonAction::from_repr(other).unwrap_or_default()),
    }
}

impl A3959StateUpdatePacket {
    /// Maps the six button slots supported by the v1 UI (single/double/long press per
    /// side) out of the eight raw action bytes. The triple press slots are kept on the
    /// device and never sent by us.
    pub fn button_configuration(&self) -> MultiButtonConfiguration {
        let slot = |index: usize| {
            let byte = self.buttons_raw[index];
            let connected = byte & 0xF;
            ButtonConfiguration {
                action: nibble_to_action(connected).unwrap_or_default(),
                is_enabled: nibble_to_action(connected).is_some(),
            }
        };
        MultiButtonConfiguration {
            left_single_click: slot(0),
            right_single_click: slot(1),
            left_double_click: slot(2),
            right_double_click: slot(3),
            left_long_press: slot(6),
            right_long_press: slot(7),
        }
    }
}

impl From<A3959StateUpdatePacket> for StateUpdatePacket {
    fn from(packet: A3959StateUpdatePacket) -> Self {
        let button_configuration = packet.button_configuration();
        Self {
            device_profile: &A3959_DEVICE_PROFILE,
            tws_status: Some(packet.tws_status),
            battery: DualBattery {
                left: SingleBattery {
                    is_charging: IsBatteryCharging::No,
                    level: BatteryLevel(to_percentage(packet.left_battery)),
                },
                right: SingleBattery {
                    is_charging: IsBatteryCharging::No,
                    level: BatteryLevel(to_percentage(packet.right_battery)),
                },
            }
            .into(),
            equalizer_configuration: packet.equalizer_configuration,
            sound_modes: None,
            sound_modes_type_two: None,
            sound_modes_type_three: Some(packet.sound_modes),
            age_range: None,
            gender: None,
            hear_id: None,
            button_configuration: Some(button_configuration),
            firmware_version: Some(packet.firmware_version_left),
            serial_number: Some(packet.serial_number),
            ambient_sound_mode_cycle: Some(packet.ambient_sound_mode_cycle),
            gaming_mode: Some(packet.gaming_mode),
            surround_sound: Some(packet.surround_sound),
            dual_connections: Some(packet.dual_connections),
            low_battery_prompt: Some(packet.low_battery_prompt),
        }
    }
}

impl InboundPacket for A3959StateUpdatePacket {
    fn command() -> crate::devices::standard::structures::Command {
        StateUpdatePacket::command()
    }

    fn take<'a, E: ParseError<&'a [u8]> + ContextError<&'a [u8]>>(
        input: &'a [u8],
    ) -> IResult<&'a [u8], A3959StateUpdatePacket, E> {
        context(
            "a3959 state update packet",
            all_consuming(|input| {
                let (input, (head, tail)) = tuple((
                    tuple((
                        TwsStatus::take,
                        le_u8,
                        le_u8,
                        take(2usize),
                        FirmwareVersion::take,
                        FirmwareVersion::take,
                        SerialNumber::take,
                        EqualizerConfiguration::take(10),
                        take(10usize),
                        le_u8,
                        take(8usize),
                    )),
                    tuple((
                        AmbientSoundModeCycle::take,
                        SoundModesTypeThree::take,
                        le_u8,
                        take_bool,
                        take_bool,
                        le_u8,
                        take_bool,
                        le_u8,
                        take_bool,
                        take_bool,
                        take(12usize),
                    )),
                ))(input)?;
                let (
                    tws_status,
                    left_battery,
                    right_battery,
                    _unknown0,
                    firmware_version_left,
                    firmware_version_right,
                    serial_number,
                    equalizer_configuration,
                    drc_preserved,
                    preserve_byte,
                    buttons_raw,
                ) = head;
                let (
                    ambient_sound_mode_cycle,
                    sound_modes,
                    _unknown1,
                    _touch_tone,
                    dual_connections,
                    surround_sound,
                    _auto_power_off_enabled,
                    _auto_power_off_duration,
                    low_battery_prompt,
                    gaming_mode,
                    _unknown2,
                ) = tail;

                Ok((
                    input,
                    A3959StateUpdatePacket {
                        tws_status,
                        left_battery,
                        right_battery,
                        firmware_version_left,
                        firmware_version_right,
                        serial_number,
                        equalizer_configuration,
                        drc_preserved: drc_preserved.try_into().expect("took exactly 10 bytes"),
                        preserve_byte,
                        buttons_raw: buttons_raw.try_into().expect("took exactly 8 bytes"),
                        ambient_sound_mode_cycle,
                        sound_modes,
                        dual_connections,
                        surround_sound: surround_sound != 0,
                        low_battery_prompt,
                        gaming_mode,
                    },
                ))
            }),
        )(input)
    }
}

#[cfg(test)]
mod tests {
    use nom::error::VerboseError;

    use crate::devices::standard::structures::{
        AmbientSoundMode, Battery, HostDevice, MultiSceneNoiseCanceling,
        NoiseCancelingModeTypeThree,
    };

    use super::*;

    /// response body of the state update command from
    /// tools/soundcore-device-faker/devices/a3959.toml (master branch)
    const FAKER_STATE_UPDATE: &[u8] = &[
        1, // host device
        1, // is tws connected
        0, // left battery (0-10)
        9, // right battery (0-10)
        255, 255, // unknown
        48, 49, 46, 54, 52, // left firmware 01.64
        48, 49, 46, 54, 52, // right firmware 01.64
        51, 57, 53, 57, 68, 69, 68, 54, 54, 57, 50, 68, 66, 54, 70, 52, // serial 3959DED6692DB6F4
        254, 254, // eq profile (custom)
        101, 120, 161, 171, 171, 152, 144, 60, 120, 120, // eq bands
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, // drc block
        10, // unknown preserve byte
        241, 240, 102, 102, 242, 243, 68, 68, // buttons
        51, // ambient sound mode cycle
        0, // ambient sound mode
        0x55, // manual + adaptive noise canceling
        0, // ambient sound mode (duplicated)
        0, // noise canceling mode type
        1, // wind noise suppression
        255, // adaptive sensitivity
        1, // multi scene anc
        49, // unknown
        1, // touch tone
        1, // dual connections
        0, // surround sound
        1, // auto power off enabled
        0, // auto power off duration
        1, // low battery prompt
        0, // gaming mode
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, // unknown
    ];

    #[test]
    fn it_parses_the_faker_state_update() {
        let (remaining, packet) =
            A3959StateUpdatePacket::take::<VerboseError<_>>(FAKER_STATE_UPDATE)
                .expect("should parse");
        assert!(remaining.is_empty());

        assert_eq!(packet.tws_status.host_device, HostDevice::Right);
        assert!(packet.tws_status.is_connected);
        assert_eq!(packet.left_battery, 0);
        assert_eq!(packet.right_battery, 9);
        assert_eq!(packet.firmware_version_left, FirmwareVersion::new(1, 64));
        assert_eq!(packet.serial_number.as_str(), "3959DED6692DB6F4");

        // equalizer: custom profile with 10 bands, byte = value + 120
        let bands: Vec<u8> = packet
            .equalizer_configuration
            .volume_adjustments()
            .bytes()
            .collect();
        assert_eq!(bands, vec![101, 120, 161, 171, 171, 152, 144, 60, 120, 120]);
        assert_eq!(packet.drc_preserved, [0; 10]);
        assert_eq!(packet.preserve_byte, 10);

        assert_eq!(
            packet.sound_modes,
            SoundModesTypeThree {
                ambient_sound_mode: AmbientSoundMode::NoiseCanceling,
                manual_noise_canceling: 5,
                adaptive_noise_canceling: 5,
                noise_canceling_mode: NoiseCancelingModeTypeThree::Manual,
                wind_noise_suppression: true,
                wind_noise_detected: false,
                noise_canceling_adaptive_sensitivity_level: 255,
                multi_scene_noise_canceling: MultiSceneNoiseCanceling::Outdoor,
            }
        );

        assert!(packet.dual_connections);
        assert!(!packet.surround_sound);
        assert!(packet.low_battery_prompt);
        assert!(!packet.gaming_mode);
    }



    #[test]
    fn it_parses_a_real_device_state_update() {
        let (remaining, packet) =
            A3959StateUpdatePacket::take::<VerboseError<_>>(REAL_DEVICE_STATE_UPDATE)
                .expect("should parse");
        assert!(remaining.is_empty());

        assert_eq!(packet.tws_status.host_device, HostDevice::Left);
        assert!(packet.tws_status.is_connected);
        assert_eq!(packet.left_battery, 9);
        assert_eq!(packet.right_battery, 9);
        assert_eq!(packet.firmware_version_left, FirmwareVersion::new(1, 44));
        assert_eq!(packet.serial_number.as_str(), "39599C2C3139C1A4");
        assert_eq!(packet.preserve_byte, 0x0a);
        assert_eq!(packet.buttons_raw, [0xff, 0xff, 0x63, 0x66, 0xff, 0xff, 0x44, 0x44]);

        assert_eq!(
            packet.sound_modes,
            SoundModesTypeThree {
                ambient_sound_mode: AmbientSoundMode::Transparency,
                manual_noise_canceling: 5,
                adaptive_noise_canceling: 5,
                noise_canceling_mode: NoiseCancelingModeTypeThree::Manual,
                wind_noise_suppression: false,
                wind_noise_detected: false,
                noise_canceling_adaptive_sensitivity_level: 255,
                multi_scene_noise_canceling: MultiSceneNoiseCanceling::Transport,
            }
        );

        assert!(packet.dual_connections);
        assert!(!packet.surround_sound);
        assert!(packet.low_battery_prompt);
        assert!(!packet.gaming_mode);

        let state_update: StateUpdatePacket = packet.into();
        match &state_update.battery {
            Battery::DualBattery(dual) => {
                assert_eq!(dual.left.level.0, 90);
                assert_eq!(dual.right.level.0, 90);
            }
            Battery::SingleBattery(_) => panic!("expected dual battery"),
        }
    }

    #[test]
    fn it_maps_the_raw_button_bytes_to_the_six_ui_slots() {
        let (_, packet) =
            A3959StateUpdatePacket::take::<VerboseError<_>>(REAL_DEVICE_STATE_UPDATE)
                .expect("should parse");
        let buttons = packet.button_configuration();
        // ff ff 63 66 ff ff 44 44
        assert!(!buttons.left_single_click.is_enabled);
        assert!(!buttons.right_single_click.is_enabled);
        assert_eq!(
            buttons.left_double_click,
            ButtonConfiguration {
                action: ButtonAction::NextSong,
                is_enabled: true
            }
        );
        assert_eq!(
            buttons.right_double_click,
            ButtonConfiguration {
                action: ButtonAction::PlayPause,
                is_enabled: true
            }
        );
        assert_eq!(
            buttons.left_long_press,
            ButtonConfiguration {
                action: ButtonAction::AmbientSoundMode,
                is_enabled: true
            }
        );
        assert_eq!(buttons.right_long_press, buttons.left_long_press);
    }

    #[test]
    fn it_maps_battery_to_percentage() {
        assert_eq!(to_percentage(0), 0);
        assert_eq!(to_percentage(5), 50);
        assert_eq!(to_percentage(9), 90);
        assert_eq!(to_percentage(10), 100);
    }

    #[test]
    fn it_converts_to_the_standard_state_update_packet() {
        let (_, packet) = A3959StateUpdatePacket::take::<VerboseError<_>>(FAKER_STATE_UPDATE)
            .expect("should parse");
        let state_update: StateUpdatePacket = packet.into();
        assert!(state_update.sound_modes_type_three.is_some());
        assert_eq!(state_update.gaming_mode, Some(false));
        assert_eq!(state_update.surround_sound, Some(false));
        assert_eq!(state_update.dual_connections, Some(true));
        assert_eq!(state_update.low_battery_prompt, Some(true));
        assert!(state_update.serial_number.is_some());
    }
}
