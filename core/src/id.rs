use std::{
    sync::Mutex,
    thread,
    time::{SystemTime, UNIX_EPOCH},
};

pub const TEKTALK_EPOCH_MS: i64 = 1_704_067_200_000; // 2024-01-01T00:00:00Z
pub const NODE_BITS: u8 = 10;
pub const SEQUENCE_BITS: u8 = 12;
pub const MAX_NODE_ID: u16 = (1 << NODE_BITS) - 1;
const MAX_SEQUENCE: u16 = (1 << SEQUENCE_BITS) - 1;
const NODE_SHIFT: u8 = SEQUENCE_BITS;
const TIMESTAMP_SHIFT: u8 = NODE_BITS + SEQUENCE_BITS;
const MAX_TIMESTAMP_DELTA: i64 = (1_i64 << 41) - 1;

#[derive(Debug, thiserror::Error, Eq, PartialEq)]
pub enum IdError {
    #[error("node id must be between 0 and {MAX_NODE_ID}")]
    InvalidNode,
    #[error("system clock is before the TEKtalk epoch")]
    BeforeEpoch,
    #[error("system clock moved backwards")]
    ClockRegression,
    #[error("Snowflake timestamp range is exhausted")]
    TimestampExhausted,
}

#[derive(Debug, Default)]
struct SnowflakeState {
    last_ms: i64,
    sequence: u16,
}

/// Thread-safe 64-bit Snowflake generator: 41-bit time, 10-bit node, 12-bit sequence.
#[derive(Debug)]
pub struct SnowflakeGenerator {
    node_id: u16,
    state: Mutex<SnowflakeState>,
}

impl SnowflakeGenerator {
    pub fn new(node_id: u16) -> Result<Self, IdError> {
        if node_id > MAX_NODE_ID {
            return Err(IdError::InvalidNode);
        }
        Ok(Self {
            node_id,
            state: Mutex::new(SnowflakeState::default()),
        })
    }

    pub fn next_id(&self) -> Result<i64, IdError> {
        loop {
            let now_ms = unix_ms()?;
            match self.next_id_at(now_ms) {
                Ok(id) => return Ok(id),
                Err(IdError::TimestampExhausted) => thread::yield_now(),
                Err(error) => return Err(error),
            }
        }
    }

    fn next_id_at(&self, unix_ms: i64) -> Result<i64, IdError> {
        let delta = unix_ms.checked_sub(TEKTALK_EPOCH_MS).ok_or(IdError::BeforeEpoch)?;
        if delta < 0 {
            return Err(IdError::BeforeEpoch);
        }
        if delta > MAX_TIMESTAMP_DELTA {
            return Err(IdError::TimestampExhausted);
        }
        let mut state = self.state.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
        if state.last_ms > unix_ms {
            return Err(IdError::ClockRegression);
        }
        if state.last_ms == unix_ms {
            if state.sequence == MAX_SEQUENCE {
                return Err(IdError::TimestampExhausted);
            }
            state.sequence += 1;
        } else {
            state.last_ms = unix_ms;
            state.sequence = 0;
        }
        Ok((delta << TIMESTAMP_SHIFT)
            | (i64::from(self.node_id) << NODE_SHIFT)
            | i64::from(state.sequence))
    }

    pub fn decode(id: i64) -> SnowflakeParts {
        SnowflakeParts {
            unix_ms: (id >> TIMESTAMP_SHIFT) + TEKTALK_EPOCH_MS,
            node_id: ((id >> NODE_SHIFT) & i64::from(MAX_NODE_ID)) as u16,
            sequence: (id & i64::from(MAX_SEQUENCE)) as u16,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SnowflakeParts {
    pub unix_ms: i64,
    pub node_id: u16,
    pub sequence: u16,
}

fn unix_ms() -> Result<i64, IdError> {
    let duration = SystemTime::now().duration_since(UNIX_EPOCH).map_err(|_| IdError::BeforeEpoch)?;
    i64::try_from(duration.as_millis()).map_err(|_| IdError::TimestampExhausted)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn snowflake_layout_round_trips() {
        let generator = SnowflakeGenerator::new(513).unwrap();
        let id = generator.next_id_at(TEKTALK_EPOCH_MS + 42).unwrap();
        assert_eq!(SnowflakeGenerator::decode(id), SnowflakeParts { unix_ms: TEKTALK_EPOCH_MS + 42, node_id: 513, sequence: 0 });
    }

    #[test]
    fn same_millisecond_increments_sequence() {
        let generator = SnowflakeGenerator::new(7).unwrap();
        let first = generator.next_id_at(TEKTALK_EPOCH_MS + 1).unwrap();
        let second = generator.next_id_at(TEKTALK_EPOCH_MS + 1).unwrap();
        assert_eq!(second, first + 1);
        assert_eq!(SnowflakeGenerator::decode(second).sequence, 1);
    }

    #[test]
    fn rejects_invalid_node_and_clock_regression() {
        assert_eq!(SnowflakeGenerator::new(MAX_NODE_ID + 1).unwrap_err(), IdError::InvalidNode);
        let generator = SnowflakeGenerator::new(1).unwrap();
        generator.next_id_at(TEKTALK_EPOCH_MS + 2).unwrap();
        assert_eq!(generator.next_id_at(TEKTALK_EPOCH_MS + 1), Err(IdError::ClockRegression));
    }
}
