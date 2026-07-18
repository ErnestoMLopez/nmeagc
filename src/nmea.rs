use crate::app::App;
use crate::gnss::{NavigationGnss, Position};

use nmea_parser::gnss::{GgaData, RmcData};

#[derive(Clone, Debug)]
pub struct RawNmeaLog {
    pub sentence: String,
    pub status: RawNmeaStatus,
}

#[derive(Clone, Debug)]
pub enum RawNmeaStatus {
    Gnss,
    Other,
    Incomplete,
    Error,
}

/// Solution type indicator
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum SolutionType {
    Invalid,
    Sps2D,
    Sps3D,
    Pps,
    RtkFixed,
    RtkFloat,
    Differential,
    Extrapolated,
    Manual,
    Simulated,
}

impl Default for SolutionType {
    fn default() -> Self {
        Self::Invalid
    }
}

impl From<nmea_parser::gnss::GgaQualityIndicator> for SolutionType {
    fn from(source: nmea_parser::gnss::GgaQualityIndicator) -> Self {
        match source {
            nmea_parser::gnss::GgaQualityIndicator::Invalid => Self::Invalid,
            nmea_parser::gnss::GgaQualityIndicator::GpsFix => Self::Sps3D,
            nmea_parser::gnss::GgaQualityIndicator::DGpsFix => Self::Differential,
            nmea_parser::gnss::GgaQualityIndicator::PpsFix => Self::Pps,
            nmea_parser::gnss::GgaQualityIndicator::RealTimeKinematic => Self::RtkFixed,
            nmea_parser::gnss::GgaQualityIndicator::RealTimeKinematicFloat => Self::RtkFloat,
            nmea_parser::gnss::GgaQualityIndicator::DeadReckoning => Self::Extrapolated,
            nmea_parser::gnss::GgaQualityIndicator::ManualInputMode => Self::Manual,
            nmea_parser::gnss::GgaQualityIndicator::SimulationMode => Self::Simulated,
        }
    }
}

impl From<nmea_parser::gnss::NavigationSystem> for NavigationGnss {
    fn from(source: nmea_parser::gnss::NavigationSystem) -> Self {
        match source {
            nmea_parser::gnss::NavigationSystem::Combination => Self::Combined,
            nmea_parser::gnss::NavigationSystem::Gps => Self::Gps,
            nmea_parser::gnss::NavigationSystem::Galileo => Self::Galileo,
            nmea_parser::gnss::NavigationSystem::Glonass => Self::Glonass,
            nmea_parser::gnss::NavigationSystem::Beidou => Self::Beidou,
            _ => Self::Other,
        }
    }
}

impl App {
    pub fn update_from_gga(&mut self, msg: &GgaData) {
        self.nav_data.gnss = msg.source.into();
        self.nav_data.time = msg.timestamp;
        self.nav_data.solution_type = msg.quality.into();
        self.nav_data.svs_used = msg.satellite_count.unwrap_or_default();

        if let (Some(lat), Some(lon), Some(hei), Some(geoid_sep)) = (
            msg.latitude,
            msg.longitude,
            msg.altitude,
            msg.geoid_separation,
        ) {
            self.nav_data.position = Some(Position {
                latitude: lat,
                longitude: lon,
                altitude: hei + geoid_sep,
            });
        } else {
            self.nav_data.position = None;
        }

        self.nav_data.geoid_separation = msg.geoid_separation;
        self.nav_data.differential_data_age = msg.age_of_dgps;
        self.nav_data.differential_ref_station_id = msg.ref_station_id;
    }

    pub fn update_from_rmc(&mut self, msg: &RmcData) {
        self.nav_data.gnss = msg.source.into();
        self.nav_data.time = msg.timestamp;

        // TODO: Completar implementación
    }
}
