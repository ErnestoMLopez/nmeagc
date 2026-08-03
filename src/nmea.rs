use crate::app::App;

#[derive(Clone, Debug)]
pub struct RawNmeaLog {
    pub sentence: String,
    pub status: RawNmeaStatus,
}

#[derive(Clone, Debug)]
pub enum RawNmeaStatus {
    Gnss,
    Unimplemented,
    Other,
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

impl From<nmea::sentences::FixType> for SolutionType {
    fn from(source: nmea::sentences::FixType) -> Self {
        match source {
            nmea::sentences::FixType::Invalid => Self::Invalid,
            nmea::sentences::FixType::Gps => Self::Sps3D,
            nmea::sentences::FixType::DGps => Self::Differential,
            nmea::sentences::FixType::Pps => Self::Pps,
            nmea::sentences::FixType::Rtk => Self::RtkFixed,
            nmea::sentences::FixType::FloatRtk => Self::RtkFloat,
            nmea::sentences::FixType::Estimated => Self::Extrapolated,
            nmea::sentences::FixType::Manual => Self::Manual,
            nmea::sentences::FixType::Simulation => Self::Simulated,
        }
    }
}

impl App {
    pub fn update_from_gga(&mut self) {
        // self.nav_data.gnss = msg.source.into();
        // self.nav_data.time = msg.timestamp;
        // self.nav_data.solution_type = msg.quality.into();
        // self.nav_data.svs_used = msg.satellite_count.unwrap_or_default();
        // self.nav_data.latitude = msg.latitude;
        // self.nav_data.longitude = msg.longitude;
        // self.nav_data.altitude = msg.altitude;
        // self.nav_data.geoid_separation = msg.geoid_separation;
        // self.nav_data.differential_data_age = msg.age_of_dgps;
        // self.nav_data.differential_ref_station_id = msg.ref_station_id;
    }

    pub fn update_from_rmc(&mut self) {
        // self.nav_data.gnss = msg.source.into();
        // self.nav_data.time = msg.timestamp;

        // TODO: Completar implementación
    }
}
