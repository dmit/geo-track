use geo_types::Coordinate;
use serde::{Deserialize, Serialize};
use time::OffsetDateTime;
use uom::si::f64::{Angle, Velocity};
use uuid::Uuid;

/// Globally unique identifier of a data source.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct SourceId(Uuid);

/// A data packet from a given source, created at a given time. May optionally
/// contain geopositional data.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct Status {
    pub source_id: SourceId,
    #[serde(with = "time::serde::timestamp")]
    pub timestamp: OffsetDateTime,
    pub location: Option<Coordinate<f64>>,
    pub bearing: Option<Angle>,
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
    fn deserialization_full() -> serde_json::Result<()> {
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
    fn deserialization_min() -> serde_json::Result<()> {
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