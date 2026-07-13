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
    pub fn update_from_gga(&mut self, msg: GgaData) {
        if let Some(_) = msg.timestamp {
            //
        }

        if let (Some(lat), Some(lon), Some(hei)) = (msg.latitude, msg.longitude, msg.altitude) {
            self.nav_data.position = Some(Position::Lla(lat, lon, hei));
        } else {
            self.nav_data.position = None;
        }
    }
}
