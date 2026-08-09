use crate::app::App;
use crate::gnss::{Gnss, GnssSignal, SignalData, SvData};

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

impl From<nmea::sentences::GnssType> for Gnss {
    fn from(source: nmea::sentences::GnssType) -> Self {
        match source {
            nmea::sentences::GnssType::Gps => Self::Gps,
            nmea::sentences::GnssType::Glonass => Self::Glonass,
            nmea::sentences::GnssType::Galileo => Self::Galileo,
            nmea::sentences::GnssType::Beidou => Self::Beidou,
            _ => Self::Other,
        }
    }
}

impl App {
    pub fn update_from_gga(&mut self) {
        // TODO
    }

    pub fn update_from_rmc(&mut self) {
        // TODO
    }

    pub fn update_from_gsv(&mut self) {
        let nmea_data = self.nmea_data.lock().expect("mutex posioned");

        let nmea_sv_data = nmea_data.satellites();
        let sv_data = &mut self.sv_data;

        // El llenado de la tabla de datos de satélites y de señales de cada satélite se hace a partir de los datos de
        // las sentencias GSV, las cuales dan información solamente de los satélites, no de las señales en sí. Esto
        // parece una copia inútil e ineficiente de información, pero está pensado para que la intefaz a la que acceda
        // la UI sea lo más genérica posible, y permita su uso en clientes para mensajes propietarios que sí posean
        // información adicional de cada señal.

        sv_data.clear();
        sv_data.extend(nmea_sv_data.iter().map(|satellite| {
            let gnss: Gnss = satellite.gnss_type().into();
            SvData {
                gnss: gnss,
                svid: satellite.prn(),
                channel: None,
                signals: vec![SignalData {
                    signal: GnssSignal::from(gnss),
                    cn0: satellite.snr().unwrap_or_default(),
                    is_active: true,
                    is_used: true,
                }],
                elevation: satellite.elevation(),
                azimuth: satellite.azimuth(),
            }
        }));
    }
}
