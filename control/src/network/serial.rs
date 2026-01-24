use eyre::Result;
use futures::{
    stream::{SplitSink, SplitStream},
    StreamExt,
};
use protocol::{codec::ControlCodec, Command};
use std::time::Duration;
use tokio_serial::frame::SerialFramed;

pub struct SerialConnection {
    /// Sink for sending commands.
    pub sink: SplitSink<SerialFramed<ControlCodec>, Command>,
    /// Stream for receiving reports.
    pub stream: SplitStream<SerialFramed<ControlCodec>>,
}

impl SerialConnection {
    pub fn new(path: &str, baud_rate: u32) -> Result<Self> {
        let serial = tokio_serial::new(path, baud_rate).timeout(Duration::from_secs(3));
        let stream = tokio_serial::SerialStream::open(&serial)?;
        let framed_stream = SerialFramed::new(stream, ControlCodec::default());
        let (sink, stream) = framed_stream.split();
        Ok(Self { sink, stream })
    }
}
