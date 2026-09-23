use crate::devices::standard::{
    packets::outbound::OutboundPacket,
    structures::{Command, EqualizerConfiguration, SoundModesTypeThree},
};

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
        bytes.extend(self.configuration.volume_adjustments().bytes());
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
