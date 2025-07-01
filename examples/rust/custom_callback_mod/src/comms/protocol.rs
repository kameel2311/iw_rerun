use serde::{Deserialize, Serialize};
use std::io::{self, ErrorKind};

/// Messages that can be sent between the client and server.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum Message {
    Timeline {
        offset_percentage: f32,
    },
    BagAndBuffer {
        bag_duration: f32,
        buffer_length: f32,
    },
    LabelingTool {
        key_sequence: String,
    },
    Disconnect,
}

impl Message {
    pub fn encode(&self) -> io::Result<Vec<u8>> {
        bincode::serialize(self).map_err(|err| io::Error::new(ErrorKind::InvalidData, err))
    }

    pub fn encode_into(&self, buffer: &mut [u8]) -> io::Result<()> {
        bincode::serialize_into(buffer, self)
            .map_err(|err| io::Error::new(ErrorKind::InvalidData, err))
    }

    pub fn decode(data: &[u8]) -> io::Result<Self> {
        bincode::deserialize(data).map_err(|err| io::Error::new(ErrorKind::InvalidData, err))
    }
}
