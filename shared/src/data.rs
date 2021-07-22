use geo_types::Coordinate;
use num_traits::float::FloatCore;
use serde::{Deserialize, Serialize};
use time::OffsetDateTime;
use uuid::Uuid;

/// Globally unique identifier of a data source.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct SourceId(Uuid);

/// The angle between true North and the direction of an object. When
/// represented in degrees, North is 0°, East is 90°, South is 180°, and West
/// is 270°.
#[derive(Debug, Copy, Clone, PartialOrd, PartialEq, Serialize, Deserialize)]
pub struct Bearing<T>(T);

impl<T: FloatCore> Bearing<T> {
    pub fn new(degrees: T) -> Self {
        Self(degrees)
    }

    /// Clockwise, from `0.0` (north) to `360.0`.
    pub fn as_degrees(self) -> T {
        self.0
    }

    /// Clockwise, from `0.0` (north).
    pub fn as_radians(self) -> T {
        self.0.to_radians()
    }
}

impl Bearing<f32> {
    pub const NORTH: Self = Self(0.0);
    pub const EAST: Self = Self(90.0);
    pub const SOUTH: Self = Self(180.0);
    pub const WEST: Self = Self(270.0);
}

impl Bearing<f64> {
    pub const NORTH: Self = Self(0.0);
    pub const EAST: Self = Self(90.0);
    pub const SOUTH: Self = Self(180.0);
    pub const WEST: Self = Self(270.0);
}

/// A data packet from a given source, created at a given time. May optionally
/// contain geopositional data.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Status {
    pub source_id: SourceId,
    pub timestamp: OffsetDateTime,
    pub location: Option<Coordinate<f64>>,
    pub bearing: Option<Bearing<f64>>,
}
