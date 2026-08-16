use crate::nmea::SolutionType;

use chrono::{DateTime, Utc};
use strum::IntoStaticStr;

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Gnss {
    #[default]
    Gps,
    Glonass,
    Galileo,
    Beidou,
    Other,
}

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, Hash, IntoStaticStr)]
pub enum GpsSignal {
    #[default]
    L1CA,
    L1C,
    L1P,
    L2C,
    L2P,
    L5I,
    L5Q,
    L5IQ,
}

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, Hash, IntoStaticStr)]
pub enum GlonassSignal {
    #[default]
    L1OF,
    L1OC,
    L2OF,
    L2OC,
    L3OC,
}

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, Hash, IntoStaticStr)]
pub enum GalileoSignal {
    E1B,
    E1C,
    #[default]
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

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, Hash, IntoStaticStr)]
pub enum BeidouSignal {
    #[default]
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum GnssSignal {
    Gps(GpsSignal),
    Glonass(GlonassSignal),
    Galileo(GalileoSignal),
    Beidou(BeidouSignal),
    Other,
}

impl From<Gnss> for GnssSignal {
    fn from(gnss: Gnss) -> Self {
        match gnss {
            Gnss::Gps => Self::Gps(GpsSignal::default()),
            Gnss::Glonass => Self::Glonass(GlonassSignal::default()),
            Gnss::Galileo => Self::Galileo(GalileoSignal::default()),
            Gnss::Beidou => Self::Beidou(BeidouSignal::default()),
            Gnss::Other => Self::Other,
        }
    }
}

impl From<GnssSignal> for Gnss {
    fn from(signal: GnssSignal) -> Self {
        match signal {
            GnssSignal::Gps(_) => Self::Gps,
            GnssSignal::Glonass(_) => Self::Glonass,
            GnssSignal::Galileo(_) => Self::Galileo,
            GnssSignal::Beidou(_) => Self::Beidou,
            GnssSignal::Other => Self::Other,
        }
    }
}

impl Gnss {
    pub const fn as_str(&self) -> &'static str {
        match self {
            Gnss::Gps => "GPS",
            Gnss::Glonass => "GLONASS",
            Gnss::Galileo => "Galileo",
            Gnss::Beidou => "BeiDou",
            Gnss::Other => "Other",
        }
    }

    pub const fn as_char(&self) -> char {
        match self {
            Gnss::Gps => 'G',
            Gnss::Glonass => 'R',
            Gnss::Galileo => 'E',
            Gnss::Beidou => 'B',
            Gnss::Other => '-',
        }
    }

    pub const fn as_code_str(&self) -> &'static str {
        match self {
            Gnss::Gps => "GPS",
            Gnss::Glonass => "GLO",
            Gnss::Galileo => "GAL",
            Gnss::Beidou => "BDS",
            Gnss::Other => "---",
        }
    }
}

impl GnssSignal {
    pub fn as_signal_code_str(&self) -> &'static str {
        match self {
            GnssSignal::Gps(signal) => signal.into(),
            GnssSignal::Glonass(signal) => signal.into(),
            GnssSignal::Galileo(signal) => signal.into(),
            GnssSignal::Beidou(signal) => signal.into(),
            GnssSignal::Other => "UNK",
        }
    }
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
    pub svid: u32,
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
