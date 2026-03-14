//! COBS codec for serial communication (postcard + COBS framing).
//!
//! Two typed codecs:
//! - [`EngineCodec`]: decodes [`Command`], encodes [`Report`] (for STM32)
//! - [`ControlCodec`]: decodes [`Report`], encodes [`Command`] (for Jetson)
//!
//! Core API is `no_std`. With `tokio` feature, codecs also implement
//! `tokio_util::codec::Decoder`/`Encoder` traits that required 'std'.

use crate::{Command, MAX_MESSAGE_SIZE, Report};
use serde::{Serialize, de::DeserializeOwned};

#[cfg(feature = "tokio")]
mod tokio;
#[cfg(feature = "tokio")]
pub use self::tokio::CodecError;

/// Codec for engine side: decodes [`Command`], encodes [`Report`].
pub struct EngineCodec {
    acc: CobsAccumulator,
    encode_buf: [u8; MAX_MESSAGE_SIZE],
}

impl EngineCodec {
    pub const fn new() -> Self {
        Self {
            acc: CobsAccumulator::new(),
            encode_buf: [0; MAX_MESSAGE_SIZE],
        }
    }

    /// Feed a single byte from UART. Returns decoded [`Command`] when a complete frame arrives.
    pub fn decode(&mut self, byte: u8) -> DecodeResult<Command> {
        self.acc.push(byte)
    }

    /// Encode a [`Report`] into COBS-framed bytes, ready to write to UART.
    pub fn encode(&mut self, report: &Report) -> Result<&[u8], postcard::Error> {
        let len = encode_cobs(report, &mut self.encode_buf)?;
        Ok(&self.encode_buf[..len])
    }
}

impl Default for EngineCodec {
    fn default() -> Self {
        Self::new()
    }
}

/// Codec for control side: decodes [`Report`], encodes [`Command`].
pub struct ControlCodec {
    acc: CobsAccumulator,
    encode_buf: [u8; MAX_MESSAGE_SIZE],
}

impl ControlCodec {
    pub const fn new() -> Self {
        Self {
            acc: CobsAccumulator::new(),
            encode_buf: [0; MAX_MESSAGE_SIZE],
        }
    }

    /// Feed a single byte. Returns decoded [`Report`] when a complete frame arrives.
    pub fn decode(&mut self, byte: u8) -> DecodeResult<Report> {
        self.acc.push(byte)
    }

    /// Encode a [`Command`] into COBS-framed bytes, ready to write.
    pub fn encode(&mut self, cmd: &Command) -> Result<&[u8], postcard::Error> {
        let len = encode_cobs(cmd, &mut self.encode_buf)?;
        Ok(&self.encode_buf[..len])
    }
}

impl Default for ControlCodec {
    fn default() -> Self {
        Self::new()
    }
}

/// Result of feeding a byte into a codec.
#[derive(Debug)]
pub enum DecodeResult<T> {
    /// Frame complete, successfully decoded.
    Complete(T),
    /// Frame delimiter found but deserialization failed.
    DeserError(postcard::Error),
    /// Byte consumed, no complete frame yet.
    Pending,
    /// Oversized frame was discarded (returned on the closing 0x00 delimiter).
    Overflow,
}

/// Accumulator for COBS decoding. Maintains an internal buffer and state to decode incoming bytes
/// until a complete frame is formed or an overflow occurs.
struct CobsAccumulator {
    buf: [u8; MAX_MESSAGE_SIZE],
    len: usize,
    overflow: bool,
}

impl CobsAccumulator {
    const fn new() -> Self {
        Self {
            buf: [0; MAX_MESSAGE_SIZE],
            len: 0,
            overflow: false,
        }
    }

    fn push<T: DeserializeOwned>(&mut self, byte: u8) -> DecodeResult<T> {
        if byte == 0x00 {
            if self.overflow {
                self.overflow = false;
                self.len = 0;
                return DecodeResult::Overflow;
            }
            if self.len == 0 {
                return DecodeResult::Pending;
            }
            let result = postcard::from_bytes_cobs(&mut self.buf[..self.len]);
            self.len = 0;
            match result {
                Ok(val) => DecodeResult::Complete(val),
                Err(e) => DecodeResult::DeserError(e),
            }
        } else if self.overflow || self.len >= MAX_MESSAGE_SIZE {
            self.overflow = true;
            DecodeResult::Pending
        } else {
            self.buf[self.len] = byte;
            self.len += 1;
            DecodeResult::Pending
        }
    }

    fn feed<T: DeserializeOwned>(&mut self, data: &[u8]) -> FeedResult<T> {
        for (i, &byte) in data.iter().enumerate() {
            match self.push(byte) {
                DecodeResult::Complete(val) => return FeedResult::Complete(val, i + 1),
                DecodeResult::DeserError(e) => return FeedResult::DeserError(e, i + 1),
                DecodeResult::Overflow => return FeedResult::Overflow(i + 1),
                DecodeResult::Pending => {}
            }
        }
        FeedResult::Pending
    }
}

enum FeedResult<T> {
    Complete(T, usize),
    DeserError(postcard::Error, usize),
    Overflow(usize),
    Pending,
}

fn encode_cobs<T: Serialize>(val: &T, buf: &mut [u8]) -> Result<usize, postcard::Error> {
    let encoded = postcard::to_slice_cobs(val, buf)?;
    Ok(encoded.len())
}

#[cfg(test)]
mod tests {
    extern crate std;

    use std::{vec, vec::Vec};

    use super::*;
    use crate::{EngineEvent, movements::MoveCmd, sensors::Data, system::SystemCmd};

    #[test]
    fn test_engine_codec_roundtrip() {
        let mut control = ControlCodec::new();
        let mut engine = EngineCodec::new();
        let mut wire = Vec::new();

        // Control encodes Command → wire
        let cmd = Command::System(SystemCmd::Ping(42));
        wire.extend_from_slice(control.encode(&cmd).unwrap());

        // Engine decodes Command from wire
        let mut decoded = None;
        for &b in &wire {
            if let DecodeResult::Complete(c) = engine.decode(b) {
                decoded = Some(c);
            }
        }
        assert_eq!(decoded, Some(cmd));

        // Engine encodes Report → wire
        wire.clear();
        let report = Report::Pong(42);
        wire.extend_from_slice(engine.encode(&report).unwrap());

        // Control decodes Report from wire
        let mut decoded = None;
        for &b in &wire {
            if let DecodeResult::Complete(r) = control.decode(b) {
                decoded = Some(r);
            }
        }
        assert_eq!(decoded, Some(report));
    }

    #[test]
    fn test_multiple_messages() {
        let mut control = ControlCodec::new();
        let mut engine = EngineCodec::new();
        let mut wire = Vec::new();

        let cmds = vec![
            Command::System(SystemCmd::Ping(1)),
            Command::Move(MoveCmd::stop()),
            Command::Move(MoveCmd::drive(0.5, 0.5, 0.0)),
        ];

        for cmd in &cmds {
            wire.extend_from_slice(control.encode(cmd).unwrap());
        }

        let mut decoded_cmds = Vec::new();
        for &b in &wire {
            if let DecodeResult::Complete(c) = engine.decode(b) {
                decoded_cmds.push(c);
            }
        }
        assert_eq!(decoded_cmds, cmds);

        // Engine sends reports back
        wire.clear();
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
            wire.extend_from_slice(engine.encode(report).unwrap());
        }

        let mut decoded_reports = Vec::new();
        for &b in &wire {
            if let DecodeResult::Complete(r) = control.decode(b) {
                decoded_reports.push(r);
            }
        }
        assert_eq!(decoded_reports, reports);
    }

    #[test]
    fn test_empty_delimiter() {
        let mut engine = EngineCodec::new();
        assert!(matches!(engine.decode(0x00), DecodeResult::Pending));
    }

    #[test]
    fn test_overflow_recovery() {
        let mut engine = EngineCodec::new();

        // Overflow the buffer
        for _ in 0..MAX_MESSAGE_SIZE {
            assert!(matches!(engine.decode(0xFF), DecodeResult::Pending));
        }
        assert!(matches!(engine.decode(0xFF), DecodeResult::Pending));
        assert!(matches!(engine.decode(0x00), DecodeResult::Overflow));

        // Next valid frame should work
        let mut control = ControlCodec::new();
        let cmd = Command::System(SystemCmd::Ping(99));
        let bytes = control.encode(&cmd).unwrap().to_vec();

        let mut decoded = None;
        for &b in &bytes {
            if let DecodeResult::Complete(c) = engine.decode(b) {
                decoded = Some(c);
            }
        }
        assert_eq!(decoded, Some(cmd));
    }

    #[test]
    fn test_message_size_limits() {
        let size_command = size_of::<Command>();
        let size_report = size_of::<Report>();
        let max_msg_size = (size_command.max(size_report) as f32 * 1.5) as usize;
        assert!(max_msg_size < MAX_MESSAGE_SIZE);
    }
}
