use crate::app::{App, AppEvent};
use crate::event::{Event, EventThread};
use crate::gnss::{Gnss, GnssSignal, SignalData, SvData};

use std::{
    io::BufRead,
    sync::{Arc, Mutex},
};

use color_eyre::{Result, eyre::WrapErr};
use nmea::{Nmea, SentenceType};

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
                svid: satellite.prn() as u8,
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

/// Runs the NMEA event thread.
///
/// This function emits NMEA events.
pub fn run_nmea_handler<R: BufRead>(
    actor: EventThread,
    mut reader: R,
    parser: Arc<Mutex<Nmea>>,
) -> Result<()> {
    let mut line = String::new();

    loop {
        match reader.read_line(&mut line) {
            Err(e) => return Err(e).wrap_err("Error reading serial port"),
            Ok(0) => return Ok(()),
            _ => {
                let sentence = line.strip_suffix("\r\n").unwrap_or(line.as_str());

                let mut nmea_parser = parser.lock().expect("mutex poisoned");

                let raw_status = match nmea_parser.parse(sentence) {
                    Ok(sentence_type) => {
                        actor.send(Event::App(AppEvent::NmeaMessage(sentence_type)));
                        match sentence_type {
                            SentenceType::GGA
                            | SentenceType::RMC
                            | SentenceType::GNS
                            | SentenceType::GSA
                            | SentenceType::GSV
                            | SentenceType::GLL => RawNmeaStatus::Gnss,
                            _ => RawNmeaStatus::Other,
                        }
                    }
                    Err(err) => {
                        // Atajamos las sentencias que nos interesarían pero no son soportados por
                        // la crate nmea. Los marcamos como tales, y al resto como no importantes.
                        match err {
                            nmea::Error::Unsupported(SentenceType::GST) => {
                                RawNmeaStatus::Unimplemented
                            }
                            nmea::Error::Unsupported(_) => RawNmeaStatus::Other,
                            _ => RawNmeaStatus::Error,
                        }
                    }
                };

                let nmea_log = RawNmeaLog {
                    sentence: sentence.to_string(),
                    status: raw_status,
                };

                actor.send(Event::App(AppEvent::RawNmeaSentence(nmea_log)));
            }
        }

        line.clear();
    }

    // for sentence in TEST_SENTENCES {
    //     thread::sleep(Duration::from_millis(1000));

    //     let raw_status: RawNmeaStatus;

    //     let mut nmea_parser = parser.lock().expect("mutex poisoned");

    //     match nmea_parser.parse(sentence) {
    //         Ok(sentence_type) => {
    //             raw_status = match sentence_type {
    //                 SentenceType::GGA
    //                 | SentenceType::RMC
    //                 | SentenceType::GNS
    //                 | SentenceType::GSA
    //                 | SentenceType::GSV
    //                 | SentenceType::GLL => RawNmeaStatus::Gnss,
    //                 _ => RawNmeaStatus::Other,
    //             };

    //             actor.send(Event::App(AppEvent::NmeaMessage(sentence_type)))
    //         }
    //         Err(err) => {
    //             // Atajamos los casos que nos interesarían, pero como no son soportados por la librería nmea, los
    //             // marcamos como tales, dejando a todos los demás no categorizados.
    //             raw_status = match err {
    //                 nmea::Error::Unsupported(SentenceType::GST) => RawNmeaStatus::Unimplemented,
    //                 nmea::Error::Unsupported(_) => RawNmeaStatus::Other,
    //                 _ => RawNmeaStatus::Error,
    //             }
    //         }
    //     }

    //     let nmea_log = RawNmeaLog {
    //         sentence: sentence.to_string(),
    //         status: raw_status,
    //     };

    //     actor.send(Event::App(AppEvent::RawNmeaSentence(nmea_log)));
    // }

    // Ok(())
}

// const TEST_SENTENCES: &[&str] = &[
//     "!AIVDM,1,1,,A,H42O55i18tMET00000000000000,2*6D",
//     "!AIVDM,1,1,,A,H42O55lti4hhhilD3nink000?050,0*40",
//     "$GPGST,182141.000,15.5,15.3,7.2,21.8,0.9,0.5,0.8*54",
//     "$GAGGA,123519,4807.038,N,01131.000,E,1,08,0.9,545.4,M,46.9,M,,*56",
//     "$GPGGA,172400.00,3455.1280,S,05757.7830,W,1,08,0.9,25.0,M,10.0,M,,*54",
//     "$GPRMC,172400.00,A,3455.1280,S,05757.7830,W,60.0,180.0,110726,,,A*6D",
//     "$GNGNS,172400.00,3455.1280,S,05757.7830,W,AA,08,0.9,25.0,10.0,,0000*60",
//     "$GPGSA,A,3,02,05,09,12,17,19,23,28,,,,,1.5,0.9,1.2*34",
//     "$GPGSV,3,1,10,2,45,120,45,5,30,250,40,9,60,300,42,12,15,180,35*4C",
//     "$GPGSV,3,2,10,17,25,45,38,19,50,210,44,23,10,315,30,28,40,90,41*7A",
//     "$GPGSV,3,3,10,31,20,270,39,32,35,150,37*70",
//     "$GPGLL,3455.1280,S,05757.7830,W,172400.00,A,A*66",
//     "$GPGGA,172401.00,3455.1340,S,05757.7860,W,1,08,0.9,25.0,M,10.0,M,,*5D",
//     "$GPRMC,172401.00,A,3455.1340,S,05757.7860,W,60.0,180.0,110726,,,A*64",
//     "$GNGNS,172401.00,3455.1340,S,05757.7860,W,AA,08,0.9,25.0,10.0,,0000*69",
//     "$GPGSA,A,3,02,05,09,12,17,19,23,28,,,,,1.5,0.9,1.2*34",
//     "$GPGSV,3,1,10,2,45,120,45,5,30,250,40,9,60,300,42,12,15,180,35*4C",
//     "$GPGSV,3,2,10,17,25,45,38,19,50,210,44,23,10,315,30,28,40,90,41*7A",
//     "$GPGSV,3,3,10,31,20,270,39,32,35,150,37*70",
//     "$GPGLL,3455.1340,S,05757.7860,W,172401.00,A,A*6F",
//     "$GPGGA,172402.00,3455.1400,S,05757.7890,W,1,08,0.9,25.0,M,10.0,M,,*52",
//     "$GPRMC,172402.00,A,3455.1400,S,05757.7890,W,60.0,180.0,110726,,,A*6B",
//     "$GNGNS,172402.00,3455.1400,S,05757.7890,W,AA,08,0.9,25.0,10.0,,0000*66",
//     "$GPGSA,A,3,02,05,09,12,17,19,23,28,,,,,1.5,0.9,1.2*34",
//     "$GPGSV,3,1,10,2,45,120,45,5,30,250,40,9,60,300,42,12,15,180,35*4C",
//     "$GPGSV,3,2,10,17,25,45,38,19,50,210,44,23,10,315,30,28,40,90,41*7A",
//     "$GPGSV,3,3,10,31,20,270,39,32,35,150,37*70",
//     "$GPGLL,3455.1400,S,05757.7890,W,172402.00,A,A*60",
//     "$GPGGA,172403.00,3455.1460,S,05757.7920,W,1,08,0.9,25.0,M,10.0,M,,*5F",
//     "$GPRMC,172403.00,A,3455.1460,S,05757.7920,W,60.0,180.0,110726,,,A*66",
//     "$GNGNS,172403.00,3455.1460,S,05757.7920,W,AA,08,0.9,25.0,10.0,,0000*6B",
//     "$GPGSA,A,3,02,05,09,12,17,19,23,28,,,,,1.5,0.9,1.2*34",
//     "$GPGSV,3,1,10,2,45,120,45,5,30,250,40,9,60,300,42,12,15,180,35*4C",
//     "$GPGSV,3,2,10,17,25,45,38,19,50,210,44,23,10,315,30,28,40,90,41*7A",
//     "$GPGSV,3,3,10,31,20,270,39,32,35,150,37*70",
//     "$GPGLL,3455.1460,S,05757.7920,W,172403.00,A,A*6D",
//     "$GPGGA,172404.00,3455.1520,S,05757.7950,W,1,08,0.9,25.0,M,10.0,M,,*5A",
//     "$GPRMC,172404.00,A,3455.1520,S,05757.7950,W,60.0,180.0,110726,,,A*63",
//     "$GNGNS,172404.00,3455.1520,S,05757.7950,W,AA,08,0.9,25.0,10.0,,0000*6E",
//     "$GPGSA,A,3,02,05,09,12,17,19,23,28,,,,,1.5,0.9,1.2*34",
//     "$GPGSV,3,1,10,2,45,120,45,5,30,250,40,9,60,300,42,12,15,180,35*4C",
//     "$GPGSV,3,2,10,17,25,45,38,19,50,210,44,23,10,315,30,28,40,90,41*7A",
//     "$GPGSV,3,3,10,31,20,270,39,32,35,150,37*70",
//     "$GPGLL,3455.1520,S,05757.7950,W,172404.00,A,A*68",
//     "$GPRMC,172404.00,A,3455.1500,S,05757.5900,W,60.0,180.0,110726,,,A*00",
//     "$GPGGA,172405.00,3455.1580,S,05757.7980,W,1,08,0.9,25.0,M,10.0,M,,*5C",
//     "$GPRMC,172405.00,A,3455.1580,S,05757.7980,W,60.0,180.0,110726,,,A*65",
//     "$GNGNS,172405.00,3455.1580,S,05757.7980,W,AA,08,0.9,25.0,10.0,,0000*68",
//     "$GPGSA,A,3,02,05,09,12,17,19,23,28,,,,,1.5,0.9,1.2*34",
//     "$GPGSV,3,1,10,2,45,120,45,5,30,250,40,9,60,300,42,12,15,180,35*4C",
//     "$GPGSV,3,2,10,17,25,45,38,19,50,210,44,23,10,315,30,28,40,90,41*7A",
//     "$GPGSV,3,3,10,31,20,270,39,32,35,150,37*70",
//     "$GPGLL,3455.1580,S,05757.7980,W,172405.00,A,A*6E",
//     "$GPGGA,172406.00,3455.1640,S,05757.8010,W,1,08,0.9,25.0,M,10.0,M,,*5F",
//     "$GPRMC,172406.00,A,3455.1640,S,05757.8010,W,60.0,180.0,110726,,,A*66",
//     "$GNGNS,172406.00,3455.1640,S,05757.8010,W,AA,08,0.9,25.0,10.0,,0000*6B",
//     "$GPGSA,A,3,02,05,09,12,17,19,23,28,,,,,1.5,0.9,1.2*34",
//     "$GPGSV,3,1,10,2,45,120,45,5,30,250,40,9,60,300,42,12,15,180,35*4C",
//     "$GPGSV,3,2,10,17,25,45,38,19,50,210,44,23,10,315,30,28,40,90,41*7A",
//     "$GPGSV,3,3,10,31,20,270,39,32,35,150,37*70",
//     "$GPGLL,3455.1640,S,05757.8010,W,172406.00,A,A*6D",
//     "$GPRMC,172406.00,A,3455.1500,S,05757.5900,W,60.0,180.0,110726,,,A*00",
//     "$GPGGA,172407.00,3455.1700,S,05757.8040,W,1,08,0.9,25.0,M,10.0,M,,*5E",
//     "$GPRMC,172407.00,A,3455.1700,S,05757.8040,W,60.0,180.0,110726,,,A*67",
//     "$GNGNS,172407.00,3455.1700,S,05757.8040,W,AA,08,0.9,25.0,10.0,,0000*6A",
//     "$GPGSA,A,3,02,05,09,12,17,19,23,28,,,,,1.5,0.9,1.2*34",
//     "$GPGSV,3,1,10,2,45,120,45,5,30,250,40,9,60,300,42,12,15,180,35*4C",
//     "$GPGSV,3,2,10,17,25,45,38,19,50,210,44,23,10,315,30,28,40,90,41*7A",
//     "$GPGSV,3,3,10,31,20,270,39,32,35,150,37*70",
//     "$GPGLL,3455.1700,S,05757.8040,W,172407.00,A,A*6C",
//     "$GPGGA,172408.00,3455.1760,S,05757.8070,W,1,08,0.9,25.0,M,10.0,M,,*54",
//     "$GPRMC,172408.00,A,3455.1760,S,05757.8070,W,60.0,180.0,110726,,,A*6D",
//     "$GNGNS,172408.00,3455.1760,S,05757.8070,W,AA,08,0.9,25.0,10.0,,0000*60",
//     "$GPGSA,A,3,02,05,09,12,17,19,23,28,,,,,1.5,0.9,1.2*34",
//     "$GPGSV,3,1,10,2,45,120,45,5,30,250,40,9,60,300,42,12,15,180,35*4C",
//     "$GPGSV,3,2,10,17,25,45,38,19,50,210,44,23,10,315,30,28,40,90,41*7A",
//     "$GPGSV,3,3,10,31,20,270,39,32,35,150,37*70",
//     "$GPGLL,3455.1760,S,05757.8070,W,172408.00,A,A*66",
//     "$GPRMC,172408.00,A,3455.1500,S,05757.5900,W,60.0,180.0,110726,,,A*00",
//     "$GPGGA,172409.00,3455.1820,S,05757.8100,W,1,08,0.9,25.0,M,10.0,M,,*58",
//     "$GPRMC,172409.00,A,3455.1820,S,05757.8100,W,60.0,180.0,110726,,,A*61",
//     "$GNGNS,172409.00,3455.1820,S,05757.8100,W,AA,08,0.9,25.0,10.0,,0000*6C",
//     "$GPGSA,A,3,02,05,09,12,17,19,23,28,,,,,1.5,0.9,1.2*34",
//     "$GPGSV,3,1,10,2,45,120,45,5,30,250,40,9,60,300,42,12,15,180,35*4C",
//     "$GPGSV,3,2,10,17,25,45,38,19,50,210,44,23,10,315,30,28,40,90,41*7A",
//     "$GPGSV,3,3,10,31,20,270,39,32,35,150,37*70",
//     "$GPGLL,3455.1820,S,05757.8100,W,172409.00,A,A*6A",
// ];
