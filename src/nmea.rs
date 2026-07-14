use crate::app::App;
use crate::gnss::Position;

use nmea_parser::gnss::GgaData;

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

impl App {
    pub fn update_from_gga(&mut self, msg: &GgaData) {
        if let Some(utc) = msg.timestamp {
            self.nav_data.time = Some(utc);
        } else {
            self.nav_data.time = None;
        }

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
    }
}
