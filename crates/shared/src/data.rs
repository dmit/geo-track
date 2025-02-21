//! This module contains data structures that describe sensor information used
//! for tracking.

use core::fmt::Display;

use geo_types::Coord;
use serde::{Deserialize, Serialize};
use time::OffsetDateTime;
use uom::si::f64::{Angle, Velocity};
use uuid::Uuid;

/// Globally unique identifier of a data source (sensor, vehicle, etc).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[repr(transparent)]
#[serde(transparent)]
pub struct SourceId(Uuid);

impl Display for SourceId {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        self.0.fmt(f)
    }
}

/// A data packet from a given source, created at a given time. May optionally
/// contain geopositional data.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct Status {
    /// Globally unique identifier of the sensor.
    pub source_id: SourceId,
    /// Timestamp of the moment the data in this `Status` packet has been
    /// collected. Serialized as seconds since UNIX epoch.
    #[serde(with = "time::serde::timestamp")]
    pub timestamp: OffsetDateTime,
    /// GPS position. Serialized as [lon, lat].
    #[serde(skip_serializing_if = "Option::is_none")]
    pub position: Option<Coord<f64>>,
    /// Movement direction. From 0 at North clockwise. Serialized as radians.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bearing: Option<Angle>,
    /// Moving speed. Serialized as meters/second.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub speed: Option<Velocity>,
}

impl Status {
    /// Merges optional fields of two [`Status`] values to produce a new value,
    /// ignoring `source_id` and `timestamp` of `rhs`. If both source values
    /// have a given field set, the one from `rhs` is used.
    #[must_use]
    pub fn merge(&self, rhs: &Self) -> Self {
        Self {
            source_id: self.source_id,
            timestamp: self.timestamp,
            position: rhs.position.or(self.position),
            bearing: rhs.bearing.or(self.bearing),
            speed: rhs.speed.or(self.speed),
        }
    }
}

#[cfg(test)]
mod tests {
    use core::marker::PhantomData;

    use float_eq::assert_float_eq;
    use geo_types::Coord;
    use time::macros::datetime;
    use uom::si::{Quantity, angle::degree, velocity::kilometer_per_hour};
    use uuid::Uuid;

    use crate::data::{SourceId, Status};

    const FULL: Status = Status {
        source_id: SourceId(Uuid::from_u128(0x0aaec05a_0e7d_4fd5_abc0_0ba69e3cfe11)),
        timestamp: datetime!(2021-07-27 08:45:19 +3),
        position: Some(Coord { x: 24.745_278, y: 59.437_222 }),
        bearing: Some(Quantity { dimension: PhantomData, units: PhantomData, value: 1.234 }),
        speed: Some(Quantity { dimension: PhantomData, units: PhantomData, value: 15. }),
    };

    const MINIMAL: Status = Status {
        source_id: SourceId(Uuid::from_u128(0x0aaec05a_0e7d_4fd5_abc0_0ba69e3cfe11)),
        timestamp: datetime!(2021-07-27 08:45:19 +3),
        position: None,
        bearing: None,
        speed: None,
    };

    const FULL_JSON: &str = r###"{
  "sourceId": "0aaec05a-0e7d-4fd5-abc0-0ba69e3cfe11",
  "timestamp": 1627364719,
  "position": {
    "x": 24.745278,
    "y": 59.437222
  },
  "bearing": 1.234,
  "speed": 15.0
}"###;

    const MINIMAL_JSON: &str = r###"{
  "sourceId": "0aaec05a-0e7d-4fd5-abc0-0ba69e3cfe11",
  "timestamp": 1627364719
}"###;

    #[rustfmt::skip]
    const FULL_CBOR: &[u8] = &[
        // header
        0xa5,
        //    /---------------- "sourceId" ----------------\
        0x68, 0x73, 0x6f, 0x75, 0x72, 0x63, 0x65, 0x49, 0x64,
        //    /- source_id (verbatim, 16 bytes)
        0x50, 0x0a, 0xae, 0xc0, 0x5a, 0x0e, 0x7d, 0x4f, 0xd5, 0xab, 0xc0, 0x0b, 0xa6, 0x9e, 0x3c, 0xfe, 0x11,
        //    /------------------ "timestamp" -------------------\
        0x69, 0x74, 0x69, 0x6d, 0x65, 0x73, 0x74, 0x61, 0x6d, 0x70,
        //    /- timestamp (u64 containing number of seconds since UNIX epoch)
        0x1a, 0x60, 0xff, 0x9d, 0x6f,
        // position
        //                      "position"
        //    /--------------------------------------------\
        0x68, 0x70, 0x6f, 0x73, 0x69, 0x74, 0x69, 0x6f, 0x6e,
        0xa2, 0x61, 0x78, 0xfb, 0x40, 0x38, 0xbe, 0xca, 0x89, 0xfc, 0x6d, 0xa4, 0x61, 0x79, 0xfb, 0x40, 0x4d, 0xb7, 0xf6, 0xe3, 0xf7, 0x8b, 0xbd, 0x67, 0x62, 0x65, 0x61, 0x72, 0x69, 0x6e, 0x67, 0xfb, 0x3f, 0xf3, 0xbe, 0x76, 0xc8, 0xb4, 0x39, 0x58, 0x65, 0x73, 0x70, 0x65, 0x65, 0x64, 0xf9, 0x4b, 0x80
    ];

    #[rustfmt::skip]
    const MINIMAL_CBOR: &[u8] = &[
        // header
        0xa2,
        //    /---------------- "sourceId" ----------------\
        0x68, 0x73, 0x6f, 0x75, 0x72, 0x63, 0x65, 0x49, 0x64,
        //    /- source_id (verbatim, 16 bytes)
        0x50, 0x0a, 0xae, 0xc0, 0x5a, 0x0e, 0x7d, 0x4f, 0xd5, 0xab, 0xc0, 0x0b, 0xa6, 0x9e, 0x3c, 0xfe, 0x11,
        //    /------------------ "timestamp" -------------------\
        0x69, 0x74, 0x69, 0x6d, 0x65, 0x73, 0x74, 0x61, 0x6d, 0x70,
        //    /- timestamp (u64 containing number of seconds since UNIX epoch)
        0x1a, 0x60, 0xff, 0x9d, 0x6f,
    ];

    fn cbor_to_bytes<T: serde::Serialize>(
        val: &T,
    ) -> Result<Vec<u8>, ciborium::ser::Error<std::io::Error>> {
        let mut bytes = Vec::new();
        ciborium::ser::into_writer(val, &mut bytes)?;
        Ok(bytes)
    }

    #[test]
    fn status_merge() {
        let merged1 = FULL.merge(&MINIMAL);
        let merged2 = MINIMAL.merge(&FULL);

        assert_eq!(merged1.source_id, FULL.source_id);
        assert_eq!(merged1.timestamp, FULL.timestamp);
        assert_eq!(merged1.position, FULL.position);
        assert_eq!(merged1.bearing, FULL.bearing);
        assert_eq!(merged1.speed, FULL.speed);

        assert_eq!(merged2.source_id, FULL.source_id);
        assert_eq!(merged2.timestamp, FULL.timestamp);
        assert_eq!(merged2.position, FULL.position);
        assert_eq!(merged2.bearing, FULL.bearing);
        assert_eq!(merged2.speed, FULL.speed);
    }

    #[test]
    fn json_serialization_full() -> serde_json::Result<()> {
        let encoded = serde_json::to_string_pretty(&FULL)?;
        assert_eq!(encoded, FULL_JSON);
        Ok(())
    }

    #[test]
    fn json_serialization_minimal() -> serde_json::Result<()> {
        let encoded = serde_json::to_string_pretty(&MINIMAL)?;
        assert_eq!(encoded, MINIMAL_JSON);
        Ok(())
    }

    #[test]
    fn cbor_serialization_full() -> Result<(), ciborium::ser::Error<std::io::Error>> {
        let encoded = cbor_to_bytes(&FULL)?;
        assert_eq!(encoded, FULL_CBOR);
        Ok(())
    }

    #[test]
    fn cbor_serialization_minimal() -> Result<(), ciborium::ser::Error<std::io::Error>> {
        let encoded = cbor_to_bytes(&MINIMAL)?;
        assert_eq!(encoded, MINIMAL_CBOR);
        Ok(())
    }

    #[test]
    fn cbor_deserialization_minimal() -> Result<(), ciborium::de::Error<std::io::Error>> {
        let decoded: Status = ciborium::de::from_reader(MINIMAL_CBOR)?;

        assert_eq!(decoded.source_id, MINIMAL.source_id);
        assert_eq!(decoded.timestamp, MINIMAL.timestamp);
        assert_eq!(decoded.position, None);
        assert_eq!(decoded.bearing, None);
        assert_eq!(decoded.speed, None);

        Ok(())
    }

    #[test]
    fn json_deserialization_full() -> serde_json::Result<()> {
        let decoded: Status = serde_json::from_str(FULL_JSON)?;

        assert_eq!(decoded.source_id, FULL.source_id);
        assert_eq!(decoded.timestamp, FULL.timestamp);
        assert_float_eq!(
            decoded.position.map(|l| l.x),
            FULL.position.map(|l| l.x),
            abs <= Some(0.000_001)
        );
        assert_float_eq!(
            decoded.position.map(|l| l.y),
            FULL.position.map(|l| l.y),
            abs <= Some(0.000_001)
        );
        assert_float_eq!(
            decoded.bearing.map(|b| b.get::<degree>()),
            FULL.bearing.map(|b| b.get::<degree>()),
            abs <= Some(0.005)
        );
        assert_float_eq!(
            decoded.speed.map(|s| s.get::<kilometer_per_hour>()),
            FULL.speed.map(|s| s.get::<kilometer_per_hour>()),
            abs <= Some(0.01)
        );

        Ok(())
    }

    #[test]
    fn json_deserialization_minimal() -> serde_json::Result<()> {
        let decoded: Status = serde_json::from_str(MINIMAL_JSON)?;

        assert_eq!(decoded.source_id, MINIMAL.source_id);
        assert_eq!(decoded.timestamp, MINIMAL.timestamp);
        assert_eq!(decoded.position, None);
        assert_eq!(decoded.bearing, None);
        assert_eq!(decoded.speed, None);

        Ok(())
    }
}
