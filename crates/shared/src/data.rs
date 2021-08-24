//! This module contains data structures that describe sensor information used
//! for tracking.

use core::fmt::Display;

use geo_types::Coordinate;
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
    /// collected.
    #[serde(with = "time::serde::timestamp")]
    pub timestamp: OffsetDateTime,
    /// GPS position.
    pub location: Option<Coordinate<f64>>,
    /// Movement direction.
    pub bearing: Option<Angle>,
    /// Moving speed.
    pub speed: Option<Velocity>,
}

#[cfg(test)]
mod tests {
    use float_eq::assert_float_eq;
    use time::macros::datetime;
    use uom::si::{angle::degree, velocity::kilometer_per_hour};
    use uuid::Uuid;

    use crate::data::{SourceId, Status};

    #[test]
    fn json_serialization_full() -> serde_json::Result<()> {
        let json = r###"
            {
                "sourceId": "0aaec05a-0e7d-4fd5-abc0-0ba69e3cfe11",
                "timestamp": 1627364719,
                "location": [59.437222, 24.745278],
                "bearing": 1.234,
                "speed": 15
            }
        "###;

        let decoded: Status = serde_json::from_str(json)?;

        assert_eq!(
            decoded.source_id,
            SourceId(Uuid::from_bytes([
                0x0a, 0xae, 0xc0, 0x5a, 0x0e, 0x7d, 0x4f, 0xd5, 0xab, 0xc0, 0x0b, 0xa6, 0x9e, 0x3c,
                0xfe, 0x11,
            ]))
        );
        assert_eq!(decoded.timestamp, datetime!(2021-07-27 08:45:19 +3));
        assert_float_eq!(decoded.location.map(|l| l.x), Some(59.437_222), abs <= Some(0.000_001));
        assert_float_eq!(decoded.location.map(|l| l.y), Some(24.745_278), abs <= Some(0.000_001));
        assert_float_eq!(
            decoded.bearing.map(|b| b.get::<degree>()),
            Some(70.7),
            abs <= Some(0.005)
        );
        assert_float_eq!(
            decoded.speed.map(|s| s.get::<kilometer_per_hour>()),
            Some(54.),
            abs <= Some(0.01)
        );

        Ok(())
    }

    #[test]
    fn json_serialization_minimal() -> serde_json::Result<()> {
        let json = r###"
            {
                "sourceId": "0aaec05a-0e7d-4fd5-abc0-0ba69e3cfe11",
                "timestamp": 1627364719
            }
        "###;

        let decoded: Status = serde_json::from_str(json)?;

        assert_eq!(
            decoded.source_id,
            SourceId(Uuid::from_bytes([
                0x0a, 0xae, 0xc0, 0x5a, 0x0e, 0x7d, 0x4f, 0xd5, 0xab, 0xc0, 0x0b, 0xa6, 0x9e, 0x3c,
                0xfe, 0x11,
            ]))
        );
        assert_eq!(decoded.timestamp, datetime!(2021-07-27 08:45:19 +3));
        assert_eq!(decoded.location, None);
        assert_eq!(decoded.bearing, None);
        assert_eq!(decoded.speed, None);

        Ok(())
    }
}
