//! Tokio trait implementations for [`ControlCodec`] and [`EngineCodec`].

use tokio_util::{
    bytes::BytesMut,
    codec::{Decoder, Encoder},
};

use crate::{Command, Report};

use super::{FeedResult, encode_cobs};

pub use error::CodecError;

impl Decoder for super::ControlCodec {
    type Item = Report;
    type Error = CodecError;

    fn decode(&mut self, src: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        decode_from_buf(&mut self.acc, src)
    }
}

impl Encoder<Command> for super::ControlCodec {
    type Error = CodecError;

    fn encode(&mut self, item: Command, dst: &mut BytesMut) -> Result<(), Self::Error> {
        let len = encode_cobs(&item, &mut self.encode_buf)?;
        dst.extend_from_slice(&self.encode_buf[..len]);
        Ok(())
    }
}

impl Decoder for super::EngineCodec {
    type Item = Command;
    type Error = CodecError;

    fn decode(&mut self, src: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        decode_from_buf(&mut self.acc, src)
    }
}

impl Encoder<Report> for super::EngineCodec {
    type Error = CodecError;

    fn encode(&mut self, item: Report, dst: &mut BytesMut) -> Result<(), Self::Error> {
        let len = encode_cobs(&item, &mut self.encode_buf)?;
        dst.extend_from_slice(&self.encode_buf[..len]);
        Ok(())
    }
}

fn decode_from_buf<T: serde::de::DeserializeOwned>(
    acc: &mut super::CobsAccumulator,
    src: &mut BytesMut,
) -> Result<Option<T>, CodecError> {
    match acc.feed(src.as_ref()) {
        FeedResult::Complete(val, consumed) => {
            let _ = src.split_to(consumed);
            Ok(Some(val))
        }
        FeedResult::DeserError(e, consumed) => {
            let _ = src.split_to(consumed);
            Err(e.into())
        }
        FeedResult::Overflow(consumed) => {
            let _ = src.split_to(consumed);
            Err(CodecError::Overflow)
        }
        FeedResult::Pending => {
            // Accumulator already copied these bytes — drain src so they
            // are not re-fed on the next poll.
            let len = src.len();
            let _ = src.split_to(len);
            Ok(None)
        }
    }
}

mod error {
    /// Codec error that wraps postcard, IO, and framing errors.
    #[derive(Debug)]
    pub enum CodecError {
        Postcard(postcard::Error),
        Io(std::io::Error),
        Overflow,
    }

    impl std::fmt::Display for CodecError {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            match self {
                CodecError::Postcard(e) => write!(f, "postcard error: {:?}", e),
                CodecError::Io(e) => write!(f, "IO error: {}", e),
                CodecError::Overflow => write!(f, "frame exceeded max message size"),
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
    use tokio_util::{
        bytes::BytesMut,
        codec::{Decoder, Encoder},
    };

    use crate::{
        Command, EngineEvent, Report, movements::MoveCmd, sensors::Data, system::SystemCmd,
    };

    use super::super::{ControlCodec, EngineCodec};

    #[test]
    fn test_tokio_codec_roundtrip() {
        let mut control = ControlCodec::default();
        let mut engine = EngineCodec::default();
        let mut buf = BytesMut::new();

        let cmd = Command::System(SystemCmd::Ping(42));
        Encoder::<Command>::encode(&mut control, cmd.clone(), &mut buf).unwrap();
        assert!(buf.len() > 0);

        let decoded = Decoder::decode(&mut engine, &mut buf).unwrap();
        assert_eq!(decoded, Some(cmd));
        buf.clear();

        let report = Report::Pong(123);
        Encoder::<Report>::encode(&mut engine, report.clone(), &mut buf).unwrap();

        let decoded = Decoder::decode(&mut control, &mut buf).unwrap();
        assert_eq!(decoded, Some(report));
    }

    #[test]
    fn test_tokio_multiple_messages() {
        let mut control = ControlCodec::default();
        let mut engine = EngineCodec::default();
        let mut buf = BytesMut::new();

        let cmds = vec![
            Command::System(SystemCmd::Ping(1)),
            Command::Move(MoveCmd::stop()),
            Command::Move(MoveCmd::drive(0.5, 0.5)),
        ];

        for cmd in &cmds {
            Encoder::<Command>::encode(&mut control, cmd.clone(), &mut buf).unwrap();
        }

        for expected in &cmds {
            let decoded = Decoder::decode(&mut engine, &mut buf).unwrap();
            assert_eq!(decoded, Some(expected.clone()));
        }
        buf.clear();

        let reports = vec![
            Report::Pong(1),
            Report::Event(EngineEvent::Ready),
            Report::Telemetry(Data::Encoder {
                left_mm: 150.5,
                right_mm: 148.2,
                left_speed: 120.0,
                right_speed: 118.5,
            }),
        ];

        for report in &reports {
            Encoder::<Report>::encode(&mut engine, report.clone(), &mut buf).unwrap();
        }

        for expected in &reports {
            let decoded = Decoder::decode(&mut control, &mut buf).unwrap();
            assert_eq!(decoded, Some(expected.clone()));
        }
    }

    #[test]
    fn test_tokio_empty_buffer() {
        let mut control = ControlCodec::default();
        let mut buf = BytesMut::new();

        let result = Decoder::decode(&mut control, &mut buf).unwrap();
        assert_eq!(result, None);
    }

    /// Cross-codec: tokio ControlCodec encodes, no_std EngineCodec decodes — and vice versa.
    #[test]
    fn test_cross_codec_compatibility() {
        use super::super::DecodeResult;

        let mut tokio_control = ControlCodec::default();
        let mut nostd_engine = EngineCodec::new();
        let mut buf = BytesMut::new();

        // Control (tokio) sends Command
        let cmd = Command::Move(MoveCmd::drive(0.5, -0.3));
        Encoder::<Command>::encode(&mut tokio_control, cmd.clone(), &mut buf).unwrap();

        // Engine (no_std) decodes
        let mut decoded = None;
        for &b in buf.as_ref() {
            if let DecodeResult::Complete(c) = nostd_engine.decode(b) {
                decoded = Some(c);
            }
        }
        assert_eq!(decoded, Some(cmd));

        // Engine (no_std) sends Report
        let report = Report::Pong(99);
        let bytes = nostd_engine.encode(&report).unwrap().to_vec();

        // Control (tokio) decodes
        let mut buf = BytesMut::from(bytes.as_slice());
        let decoded = Decoder::decode(&mut tokio_control, &mut buf).unwrap();
        assert_eq!(decoded, Some(report));
    }
}
