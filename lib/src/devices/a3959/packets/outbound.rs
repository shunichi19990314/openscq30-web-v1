use crate::devices::standard::{
    packets::outbound::OutboundPacket,
    structures::{Command, EqualizerConfiguration, SoundModesTypeThree},
};

/// Sets a single button action of the A3959 (command `[0x04, 0x81]`, 3 byte body:
/// `[side, button id, action byte]`, side: 0 = left, 1 = right). The action byte packs two action ids:
/// low nibble = action while TWS connected, high nibble = action while disconnected
/// (15 = disabled), matching the v2 `ActionKind::TwsLowBits` encoding.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SetButtonActionPacket {
    pub side: u8,
    pub button_id: u8,
    pub action_byte: u8,
}

impl OutboundPacket for SetButtonActionPacket {
    fn command(&self) -> Command {
        Command::new([0x08, 0xee, 0x00, 0x00, 0x00, 0x04, 0x81])
    }

    fn body(&self) -> Vec<u8> {
        vec![self.side, self.button_id, self.action_byte]
    }
}

/// Sets the sound modes of the A3959 (command `[0x06, 0x81]`, 7 byte body).
/// Same command as type two devices, but with the type three body layout.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SetSoundModeTypeThreePacket {
    pub sound_modes: SoundModesTypeThree,
}

impl OutboundPacket for SetSoundModeTypeThreePacket {
    fn command(&self) -> Command {
        Command::new([0x08, 0xee, 0x00, 0x00, 0x00, 0x06, 0x81])
    }

    fn body(&self) -> Vec<u8> {
        self.sound_modes.bytes().to_vec()
    }
}

/// Sets the equalizer of the A3959 (command `[0x02, 0x83]`).
///
/// Body: profile id (le u16) + 10 eq bands + 10 dynamic range compression bytes.
/// The meaning of the DRC block is not fully understood (the v2 implementation has an open
/// TODO about it), so the bytes received from the device are preserved verbatim, similar to
/// how the A3945 preserves its two extra eq bands.
#[derive(Debug, Clone, PartialEq)]
pub struct SetEqualizerMonoPreservedDrcPacket<'a> {
    pub configuration: &'a EqualizerConfiguration,
    pub preserved_drc: &'a [u8; 10],
}

impl OutboundPacket for SetEqualizerMonoPreservedDrcPacket<'_> {
    fn command(&self) -> Command {
        Command::new([0x08, 0xee, 0x00, 0x00, 0x00, 0x02, 0x83])
    }

    fn body(&self) -> Vec<u8> {
        let mut bytes: Vec<u8> = Vec::with_capacity(22);
        bytes.extend(self.configuration.profile_id().to_le_bytes());
        let bands: Vec<u8> = self.configuration.volume_adjustments().bytes().collect();
        bytes.extend(bands.iter().copied());
        // the A3959 equalizer has 10 bands; pad with 0.0 dB when the configuration
        // carries fewer values (e.g. seeded from an 8 band preset profile)
        for _ in bands.len()..10 {
            bytes.push(120);
        }
        bytes.extend(self.preserved_drc);
        bytes
    }
}

#[cfg(test)]
mod tests {
    use crate::devices::standard::packets::outbound::OutboundPacketBytesExt;
    use crate::devices::standard::structures::{
        AmbientSoundMode, MultiSceneNoiseCanceling, NoiseCancelingModeTypeThree,
    };

    use super::*;

    #[test]
    fn it_pads_an_eight_band_configuration_to_ten_bands() {
        use crate::devices::standard::structures::PresetEqualizerProfile;
        let configuration =
            EqualizerConfiguration::new_from_preset_profile(PresetEqualizerProfile::Acoustic);
        let packet = SetEqualizerMonoPreservedDrcPacket {
            configuration: &configuration,
            preserved_drc: &[0; 10],
        };
        let bytes = packet.bytes();
        // 7 command + len + 0x00 + 22 body + checksum
        assert_eq!(bytes.len(), 32);
        let body = &bytes[9..31];
        assert_eq!(&body[0..2], &[0x01, 0x00]); // Acoustic profile id (le)
        // 8 preset bands + 2 padded 0.0 dB bands
        assert_eq!(body[2..12].iter().filter(|b| **b == 120).count(), 2);
        assert_eq!(&body[12..22], &[0; 10]); // preserved drc
    }

    #[test]
    fn set_sound_mode_type_three_matches_the_v2_packet() {
        // body taken from the v2 test `set_manual_noise_canceling` (command [0x06, 0x81],
        // body [0, 37, 0, 0, 1, 255, 1]); v1 framing adds the length byte and checksum suffix
        const EXPECTED: &[u8] = &[
            0x08, 0xee, 0x00, 0x00, 0x00, 0x06, 0x81, 0x11, 0x00, 0, 37, 0, 0, 1, 255, 1, 0xb4,
        ];
        let packet = SetSoundModeTypeThreePacket {
            sound_modes: SoundModesTypeThree {
                ambient_sound_mode: AmbientSoundMode::NoiseCanceling,
                manual_noise_canceling: 2,
                adaptive_noise_canceling: 5,
                noise_canceling_mode: NoiseCancelingModeTypeThree::Manual,
                wind_noise_suppression: true,
                wind_noise_detected: false,
                noise_canceling_adaptive_sensitivity_level: 255,
                multi_scene_noise_canceling: MultiSceneNoiseCanceling::Outdoor,
            },
        };
        assert_eq!(EXPECTED, packet.bytes().as_slice());
    }
}
