//! Tokio codec for serial communication.
//!
//! Provides two codecs:
//! - `ControlCodec`: for control side (sends Commands, receives Reports)
//! - `EngineCodec`: for engine side (receives Commands, sends Reports)

use tokio_util::{
    bytes::BytesMut,
    codec::{Decoder, Encoder},
};

use crate::{Command, Report};

/// Maximum size for serialized message buffer.
/// If Command or Report exceeds this size, encoding will fail with SerializeBufferFull error.
const MAX_MESSAGE_SIZE: usize = 256;

/// Codec for control side: sends Commands, receives Reports.
#[derive(Debug, Clone)]
pub struct ControlCodec {
    /// Reusable buffer for encoding messages.
    encode_buf: [u8; MAX_MESSAGE_SIZE],
}

impl Default for ControlCodec {
    fn default() -> Self {
        Self {
            encode_buf: [0; MAX_MESSAGE_SIZE],
        }
    }
}

impl Decoder for ControlCodec {
    type Item = Report;
    type Error = error::CodecError;

    fn decode(&mut self, src: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        // find COBS frame delimiter (0x00)
        if let Some(pos) = src.iter().position(|b| *b == 0) {
            // take frame bytes including delimiter
            let mut frame = src.split_to(pos + 1);
            // remove delimiter
            frame.truncate(pos);
            let report = postcard::from_bytes_cobs(frame.as_mut())?;
            return Ok(Some(report));
        }
        Ok(None)
    }
}

impl Encoder<Command> for ControlCodec {
    type Error = error::CodecError;

    fn encode(&mut self, item: Command, dst: &mut BytesMut) -> Result<(), Self::Error> {
        let encoded = postcard::to_slice_cobs(&item, &mut self.encode_buf)?;
        dst.extend_from_slice(encoded);
        Ok(())
    }
}

/// Codec for engine side: receives Commands, sends Reports.
#[derive(Debug, Clone)]
pub struct EngineCodec {
    /// Reusable buffer for encoding messages.
    encode_buf: [u8; MAX_MESSAGE_SIZE],
}

impl Default for EngineCodec {
    fn default() -> Self {
        Self {
            encode_buf: [0; MAX_MESSAGE_SIZE],
        }
    }
}

impl Decoder for EngineCodec {
    type Item = Command;
    type Error = error::CodecError;

    fn decode(&mut self, src: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        if let Some(pos) = src.iter().position(|b| *b == 0) {
            let mut frame = src.split_to(pos + 1);
            frame.truncate(pos);
            let cmd = postcard::from_bytes_cobs(frame.as_mut())?;
            return Ok(Some(cmd));
        }
        Ok(None)
    }
}

impl Encoder<Report> for EngineCodec {
    type Error = error::CodecError;

    fn encode(&mut self, item: Report, dst: &mut BytesMut) -> Result<(), Self::Error> {
        let encoded = postcard::to_slice_cobs(&item, &mut self.encode_buf)?;
        dst.extend_from_slice(encoded);
        Ok(())
    }
}

mod error {
    /// Codec error that wraps both postcard and IO errors.
    #[derive(Debug)]
    pub enum CodecError {
        Postcard(postcard::Error),
        Io(std::io::Error),
    }

    impl std::fmt::Display for CodecError {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            match self {
                CodecError::Postcard(e) => write!(f, "Postcard error: {:?}", e),
                CodecError::Io(e) => write!(f, "IO error: {}", e),
            }
        }
    }

    impl std::error::Error for CodecError {}

    impl From<postcard::Error> for CodecError {
        fn from(e: postcard::Error) -> Self {
            CodecError::Postcard(e)
        }
    }

    impl From<std::io::Error> for CodecError {
        fn from(e: std::io::Error) -> Self {
            CodecError::Io(e)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{EngineEvent, movements::MoveCmd, system::SystemCmd};

    #[test]
    fn test_codec_roundtrip() {
        let mut codec = ControlCodec::default();
        let mut buf = BytesMut::new();

        // Encode Command
        let cmd = Command::System(SystemCmd::Ping(42));
        codec.encode(cmd.clone(), &mut buf).unwrap();

        assert!(buf.len() > 0);

        // Decode on engine side
        let mut engine_codec = EngineCodec::default();
        let decoded = engine_codec.decode(&mut buf).unwrap();
        assert_eq!(decoded, Some(cmd));
        buf.clear();

        // Encode Report
        let report = Report::Pong(123);
        engine_codec.encode(report.clone(), &mut buf).unwrap();

        // Decode on control side
        let mut codec = ControlCodec::default();
        let decoded = codec.decode(&mut buf).unwrap();
        assert_eq!(decoded, Some(report));
    }

    #[test]
    fn test_multiple_commands() {
        let mut codec = ControlCodec::default();
        let mut buf = BytesMut::new();

        let cmds = vec![
            Command::System(SystemCmd::Ping(1)),
            Command::Move(MoveCmd::stop()),
            Command::Move(MoveCmd::drive(0.5, 0.5)),
        ];

        for cmd in &cmds {
            codec.encode(cmd.clone(), &mut buf).unwrap();
            println!("Encoded command: {:?}, buffer length: {}", cmd, buf.len());
        }

        let mut engine_codec = EngineCodec::default();
        for expected_cmd in &cmds {
            let decoded = engine_codec.decode(&mut buf).unwrap();
            println!(
                "Decoded command: {:?}, buffer length after decode: {}",
                decoded,
                buf.len()
            );
            assert_eq!(decoded, Some(expected_cmd.clone()));
        }
        buf.clear();

        let reports = vec![
            Report::Pong(1),
            Report::Event(EngineEvent::Ready),
            Report::Event(EngineEvent::EmergencyStop),
        ];

        for report in &reports {
            engine_codec.encode(report.clone(), &mut buf).unwrap();
        }

        let mut codec = ControlCodec::default();
        for expected_report in &reports {
            let decoded = codec.decode(&mut buf).unwrap();
            assert_eq!(decoded, Some(expected_report.clone()));
        }
    }

    #[test]
    fn test_empty_buffer() {
        let mut codec = ControlCodec::default();
        let mut buf = BytesMut::new();

        let result = codec.decode(&mut buf).unwrap();
        assert_eq!(result, None);
    }

    #[test]
    fn test_message_size_limits() {
        let size_command = size_of::<Command>();
        let size_report = size_of::<Report>();
        // simple heuristic: 1.5 is a safe multiplier for overhead
        // wire size can be larger due to serialization overhead (COBS + postcard)
        let max_msg_size = (size_command.max(size_report) as f32 * 1.5) as usize;
        assert!(max_msg_size < MAX_MESSAGE_SIZE);
    }
}
