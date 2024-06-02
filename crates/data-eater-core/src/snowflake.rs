//! A snowflake is a special type that is used to store both a unique ID and
//! some metadata about what it identifies

use std::fmt::{Display, Formatter};
use std::time::SystemTime;

use bitfield::bitfield;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use thiserror::Error;
use crate::hashing::consistent_hash;

const SNOWFLAKE_TIMESTAMP_MASK: u64 = 0xFFFFFFFFFFFF;

bitfield! {
    #[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash, Serialize, Deserialize)]
    #[serde(transparent)]
    pub struct Snowflake(u64);
    impl Debug;

    u64, timestamp_raw, set_timestamp_raw: 62, 20;
    /// The id of the machine that created this snowflake
    u16, machine_id_, set_machine_id_: 19, 10;
    /// The id of this type created within the given millisecond
    u16, sequence_id_, set_sequence_id_: 9,0;
}

impl Snowflake {
    /// Creates a new snowflake with a given machine id and sequence id
    fn new(machine_id: u16, sequence_id: u16, time: SystemTime) -> Self {
        let mut snowflake = Self(0);
        snowflake.set_timestamp_raw(time.duration_since(SystemTime::UNIX_EPOCH).unwrap().as_millis() as u64);
        snowflake.set_machine_id_(machine_id);
        snowflake.set_sequence_id_(sequence_id);
        snowflake
    }

    /// Gets the timestamp this snowflake was created
    pub fn timestamp(&self) -> DateTime<Utc> {
        DateTime::from_timestamp_micros(self.timestamp_raw() as i64)
            .expect("Should always be valid")
    }

    /// Gets the id of the machine that made this
    pub fn machine_id(&self) -> u16 {
        self.machine_id_()
    }

    /// The id of this type created within the given millisecond
    pub fn sequence_id(&self) -> u16 {
        self.sequence_id_()
    }

    /// Decomposes this snowflake
    pub fn decompose(self) -> (DateTime<Utc>, u16, u16) {
        (self.timestamp(), self.machine_id(), self.sequence_id())
    }
}
impl Display for Snowflake {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:x}|{:x}|{:x}", self.timestamp_raw(), self.machine_id(), self.sequence_id())
    }
}

impl From<Snowflake> for u64 {
    fn from(value: Snowflake) -> Self {
        value.0
    }
}

impl TryFrom<u64> for Snowflake {
    type Error = SnowflakeFormatError;

    fn try_from(value: u64) -> Result<Self, Self::Error> {
        if (value >> 63) & 0b1 == 1 {
            return Err(SnowflakeFormatError);
        }

        Ok(Snowflake(value))
    }
}

#[derive(Debug, Error)]
#[error("Snowflake not in correct format")]
pub struct SnowflakeFormatError;

/// Factory for creating snowflakes.
#[derive(Debug)]
pub struct SnowflakeFactory {
    machine_id: u16,
    sequence_id: u16,
    last_timestamp_millis: u64,
}

impl SnowflakeFactory {

    /// Creates a new snowflake factory
    pub fn new() -> Self {
        let id = machine_uid::get().expect("could not get unique id");
        let id_hash = consistent_hash(id) as u16;

        Self {
            machine_id: id_hash,
            sequence_id: 0,
            last_timestamp_millis: 0,
        }
    }

    /// Gets the next snowflake
    pub fn next(&mut self) -> Snowflake {
        let timestamp = SystemTime::now();
        let timestamp_millis = timestamp.duration_since(SystemTime::UNIX_EPOCH).unwrap().as_millis() as u64 & SNOWFLAKE_TIMESTAMP_MASK;
        if self.last_timestamp_millis == timestamp_millis {
            self.sequence_id += 1;
        } else {
            self.last_timestamp_millis = timestamp_millis;
            self.sequence_id = 0;
        }
        let sequence_id = self.sequence_id;
        Snowflake::new(self.machine_id, sequence_id, timestamp)
    }
}

#[cfg(test)]
mod tests {
    use crate::snowflake::{Snowflake, SnowflakeFactory};

    #[test]
    fn sequential_ids_are_unique() {
        let mut factory = SnowflakeFactory::new();
        let first = factory.next();
        let second = factory.next();
        assert_ne!(first, second, "Two sequential ids should not be equal");
        assert!(second > first, "first should be less than second but {first:?} >= {second:?}");
        assert_eq!(first.machine_id(), second.machine_id(), "machine id should be the same");
        assert!(
            first.timestamp() < second.timestamp() || first.sequence_id() < second.sequence_id(),
            "either timestamp or sequence id of first id should be lower, but {first:?} >= {second:?}"
        )
    }

    #[test]
    fn u64_conversion() {
        let valid = 0x0;
        assert!(Snowflake::try_from(valid).is_ok());
        let invalid = u64::MAX;
        assert!(Snowflake::try_from(invalid).is_err());
    }

}