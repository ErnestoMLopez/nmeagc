use crate::nmea::SolutionType;

use chrono::{DateTime, Utc};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Gnss {
    Gps,
    Glonass,
    Galileo,
    Beidou,
    Other,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GpsSignal {
    L1CA,
    L1C,
    L1P,
    L2C,
    L2P,
    L5I,
    L5Q,
    L5IQ,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GlonassSignal {
    L1OF,
    L1OC,
    L2OF,
    L2OC,
    L3OC,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GalileoSignal {
    E1B,
    E1C,
    E1BC,
    E6B,
    E6C,
    E6BC,
    E5BI,
    E5BQ,
    E5BIQ,
    E5AI,
    E5AQ,
    E5AIQ,
    E5Q,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BeidouSignal {
    B1I,
    B1Q,
    B1A,
    B1C,
    B2I,
    B2Q,
    B2A,
    B2B,
    B3I,
    B3Q,
    B3A,
}

#[derive(Debug, Clone)]
pub enum GnssSignal {
    Gps(GpsSignal),
    Glonass(GlonassSignal),
    Galileo(GalileoSignal),
    Beidou(BeidouSignal),
}

#[derive(Debug, Clone)]
pub struct SignalData {
    pub signal: GnssSignal,
    pub cn0: f32,
    pub is_active: bool,
    pub is_used: bool,
}

#[derive(Debug, Clone)]
pub struct SvData {
    /// GNSS constellation
    pub gnss: Gnss,
    /// Space vehicle ID
    pub svid: u8,
    /// Frequency channel number (for GLONASS)
    pub channel: Option<i8>,
    /// Signal data for the space vehicle
    pub signals: Vec<SignalData>,
    /// Elevation [degrees]
    pub elevation: Option<f32>,
    /// Azimuth [degrees]
    pub azimuth: Option<f32>,
}

#[derive(Debug, Clone)]
pub struct NavigationData {
    /// Navigation solution type
    pub solution_type: SolutionType,
    /// UTC time
    pub time: Option<DateTime<Utc>>,
    /// Geodetic latitude (°)
    pub latitude: Option<f64>,
    /// Geodetic longitude (°)
    pub longitude: Option<f64>,
    /// Mean-Sea-Level altitude (m)
    pub altitude: Option<f64>,
    /// Geoid-ellipsoid separation [m]
    pub geoid_separation: Option<f64>,
    /// Satellites used for the navigation solution.
    pub svs_used: u8,
    /// Age of differential data.
    pub differential_data_age: Option<f64>,
    /// Differential reference station ID.
    pub differential_ref_station_id: Option<u16>,
}

impl Default for NavigationData {
    fn default() -> Self {
        Self {
            solution_type: SolutionType::Invalid,
            time: None,
            latitude: None,
            longitude: None,
            altitude: None,
            geoid_separation: None,
            svs_used: 0,
            differential_data_age: None,
            differential_ref_station_id: None,
        }
    }
}
