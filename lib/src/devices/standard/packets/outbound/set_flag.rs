use crate::devices::standard::structures::Command;

use super::outbound_packet::OutboundPacket;

/// Generic single byte flag setter, used by the A3959 for gaming mode (`[0x01, 0x87]`),
/// surround sound (`[0x02, 0x86]`) and the low battery prompt (`[0x10, 0x82]`).
/// Command bytes match the v2 `SET_*_COMMAND` constants.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SetFlagPacket {
    pub command: Command,
    pub enabled: bool,
}

impl OutboundPacket for SetFlagPacket {
    fn command(&self) -> Command {
        self.command
    }

    fn body(&self) -> Vec<u8> {
        vec![self.enabled.into()]
    }
}

#[cfg(test)]
mod tests {
    use crate::devices::standard::packets::outbound::OutboundPacketBytesExt;

    use super::*;

    #[test]
    fn it_matches_the_expected_surround_sound_packet() {
        // v2 test `set_surround_sound`: command [0x02, 0x86], body [1]; v1 framing adds
        // the length byte and checksum
        let packet = SetFlagPacket {
            command: Command::new([0x08, 0xee, 0x00, 0x00, 0x00, 0x02, 0x86]),
            enabled: true,
        };
        let bytes = packet.bytes();
        assert_eq!(&bytes[..7], &[0x08, 0xee, 0x00, 0x00, 0x00, 0x02, 0x86]);
        assert_eq!(&bytes[7..9], &[0x0b, 0x00]);
        assert_eq!(&bytes[9], &1);
        assert_eq!(bytes.len(), 11);
    }
}
