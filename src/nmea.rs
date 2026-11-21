use std::collections::BTreeMap;
use std::io::{self, BufRead};
use std::mem;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

use color_eyre::Result;
use color_eyre::eyre::WrapErr;
use log::debug;
use nmea::ParseResult;
use nmea::sentences::{GgaData, GllData, GnsData, GsaData, GsvData, RmcData};

use crate::app::{App, AppEvent};
use crate::event::{Event, EventThread};
use crate::gnss::{Gnss, GnssSignal, SignalData, SvData};

/// NMEA messages relevant to the application, already parsed from their raw sentence.
#[derive(Debug)]
pub enum NmeaGnssMessage {
    Gga(GgaData),
    Rmc(RmcData),
    Gns(GnsData),
    Gsa(GsaData),
    Gsv(GsvData),
    Gll(GllData),
}

impl TryFrom<ParseResult> for NmeaGnssMessage {
    type Error = ();
    fn try_from(result: ParseResult) -> Result<Self, Self::Error> {
        match result {
            ParseResult::GGA(data) => Ok(Self::Gga(data)),
            ParseResult::RMC(data) => Ok(Self::Rmc(data)),
            ParseResult::GNS(data) => Ok(Self::Gns(data)),
            ParseResult::GSA(data) => Ok(Self::Gsa(data)),
            ParseResult::GSV(data) => Ok(Self::Gsv(data)),
            ParseResult::GLL(data) => Ok(Self::Gll(data)),
            _ => Err(()),
        }
    }
}

/// GSV cycle being assembled for a constellation.
#[derive(Debug, Default)]
pub struct GsvAssembler {
    /// Parts of the cycle being assembled, keyed by sentence number.
    pending: BTreeMap<u16, GsvData>,
    /// Number of sentences expected in the cycle being assembled.
    expected: u16,
}

impl GsvAssembler {
    /// Adds a part received from a GSV sentence.
    ///
    /// The sentences of a cycle describe the satellites at the same instant, so parts from
    /// different cycles must not be combined. The cycle is committed only once all its parts have
    /// been received; an incomplete cycle is discarded when the next one starts.
    fn push(&mut self, part: GsvData) {
        if part.sentence_num == 0 || part.sentence_num > part.number_of_sentences {
            return;
        }

        if part.sentence_num == 1 {
            self.pending.clear();
            self.expected = part.number_of_sentences;
        } else if self.pending.is_empty() || part.number_of_sentences != self.expected {
            // Without the first sentence there is no way to tell which cycle the part belongs
            // to, and a cycle with a different number of sentences cannot be mixed with the one
            // being assembled.
            return;
        }

        self.pending.insert(part.sentence_num, part);
    }

    /// Returns the parts of the cycle being assembled, ordered by sentence number, once all of
    /// them have been received.
    fn try_assemble(&mut self) -> Option<Vec<GsvData>> {
        if !self.pending.is_empty() && self.pending.len() == usize::from(self.expected) {
            Some(mem::take(&mut self.pending).into_values().collect())
        } else {
            None
        }
    }
}

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
#[derive(Clone, Copy, Debug, PartialEq, Default)]
pub enum SolutionType {
    #[default]
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

/// State accumulated from the received NMEA sentences that is needed to interpret them.
#[derive(Debug, Default)]
pub struct NmeaParserData {
    /// GSV cycle assemblers per constellation.
    gsv_assemblers: BTreeMap<Gnss, GsvAssembler>,
    /// Satellites used in the navigation solution per constellation, as reported by GSA.
    gsa_used_prns: BTreeMap<Gnss, Vec<u8>>,
}

impl App {
    pub fn update_from_gga(&mut self, gga: &GgaData) {
        let nav_data = &mut self.nav_data;

        nav_data.solution_type = gga
            .fix_type
            .map_or(SolutionType::Invalid, SolutionType::from);
        // GGA carries the time of day only. The date is taken from the last known UTC timestamp, so
        // the timestamp is only available once both are known.
        nav_data.time = gga.fix_time.and_then(|time| {
            let date = nav_data.time?.date_naive();
            Some(date.and_time(time).and_utc())
        });
        nav_data.latitude = gga.latitude;
        nav_data.longitude = gga.longitude;
        nav_data.altitude = gga.altitude.map(f64::from);
        nav_data.geoid_separation = gga.geoid_separation.map(f64::from);
        nav_data.svs_used = gga
            .fix_satellites
            .map_or(0, |svs| u8::try_from(svs).unwrap_or(u8::MAX));
    }

    pub fn update_from_rmc(&mut self, rmc: &RmcData) {
        let nav_data = &mut self.nav_data;

        nav_data.time = rmc
            .fix_date
            .zip(rmc.fix_time)
            .map(|(date, time)| date.and_time(time).and_utc());
        nav_data.latitude = rmc.lat;
        nav_data.longitude = rmc.lon;
    }

    pub fn update_from_gsv(&mut self, gsv: &GsvData) {
        let gnss: Gnss = gsv.gnss_type.into();
        let gsv_assembler = self
            .nmea_parser_data
            .gsv_assemblers
            .entry(gnss)
            .or_default();

        gsv_assembler.push(gsv.clone());

        if let Some(gsv_block) = gsv_assembler.try_assemble() {
            self.update_sv_data(gnss, &gsv_block);
        }
    }

    pub fn update_from_gsa(&mut self, gsa: &GsaData) {
        // GN-talker receivers may omit the system ID, making the PRNs ambiguous across
        // constellations, so they cannot be attributed to any of them.
        let Some(system_id) = gsa.system_id else {
            return;
        };
        let gnss: Gnss = nmea::sentences::GnssType::from(system_id).into();

        // Each GSA sentence reports the satellites used for one constellation during the current
        // epoch, so a new sentence for the same constellation replaces the previous set.
        let used_prns = gsa
            .fix_sats_prn
            .iter()
            .filter_map(|prn| u8::try_from(*prn).ok())
            .collect::<Vec<_>>();

        for sv in &mut self.sv_data {
            if sv.gnss == gnss {
                for signal in &mut sv.signals {
                    signal.is_used = used_prns.contains(&sv.svid);
                }
            }
        }

        self.nmea_parser_data.gsa_used_prns.insert(gnss, used_prns);
    }

    /// Replaces the satellites of a constellation with those of a completed GSV cycle.
    fn update_sv_data(&mut self, gnss: Gnss, gsv_block: &[GsvData]) {
        // The satellite table is filled from the GSV sentences, which give information about the
        // satellites, not about the signals themselves. A signal entry is created per satellite
        // anyway, so the UI can work with a generic interface that also supports proprietary
        // messages with actual per-signal data.
        let used_prns = self.nmea_parser_data.gsa_used_prns.get(&gnss);
        let mut sv_data = Vec::new();

        // Newest sentences first, so duplicated satellites keep their latest report.
        for part in gsv_block.iter().rev() {
            for satellite in part.sats_info.iter().flatten() {
                let svid = satellite.prn() as u8;

                sv_data.push(SvData {
                    gnss,
                    svid,
                    channel: None,
                    signals: vec![SignalData {
                        signal: GnssSignal::from(gnss),
                        cn0: satellite.snr().unwrap_or_default(),
                        is_used: used_prns.is_some_and(|prns| prns.contains(&svid)),
                    }],
                    elevation: satellite.elevation(),
                    azimuth: satellite.azimuth(),
                });
            }
        }

        // The sort is stable, so the newest report of each satellite stays first and is the one
        // kept by the deduplication.
        sv_data.sort_by_key(|sv| (sv.gnss, sv.svid));
        sv_data.dedup_by(|a, b| a.gnss == b.gnss && a.svid == b.svid);

        // Replace the previous report of this constellation, leaving the others untouched.
        self.sv_data.retain(|sv| sv.gnss != gnss);
        self.sv_data.extend(sv_data);
        self.sv_data.sort_by_key(|sv| (sv.gnss, sv.svid));
    }
}

/// Runs the NMEA event thread.
///
/// This function emits NMEA events.
pub fn run_nmea_handler<R: BufRead>(
    actor: EventThread,
    app_running: Arc<AtomicBool>,
    mut reader: R,
) -> Result<()> {
    let mut line = Vec::new();

    debug!("NMEA handler worker thread started");

    while app_running.load(Ordering::Relaxed) {
        match reader.read_until(b'\n', &mut line) {
            Err(e) if is_read_timeout(&e) => continue,
            Err(e) => return Err(e).wrap_err("Error reading input source"),
            Ok(0) => return Ok(()),
            _ => {
                let sentence =
                    std::str::from_utf8(&line).wrap_err("Error decoding NMEA sentence")?;
                let sentence = sentence.strip_suffix("\r\n").unwrap_or(sentence);

                let parsed = nmea::parse_str(sentence);

                let raw_status = match &parsed {
                    Ok(ParseResult::GGA(_))
                    | Ok(ParseResult::RMC(_))
                    | Ok(ParseResult::GSA(_))
                    | Ok(ParseResult::GSV(_)) => RawNmeaStatus::Gnss,
                    Ok(ParseResult::GNS(_)) | Ok(ParseResult::GLL(_)) | Ok(ParseResult::GST(_)) => {
                        RawNmeaStatus::Unimplemented
                    }
                    Ok(_) => RawNmeaStatus::Other,
                    Err(_) => RawNmeaStatus::Error,
                };

                if let Ok(result) = parsed
                    && let Ok(message) = NmeaGnssMessage::try_from(result)
                {
                    actor.send(Event::App(AppEvent::NmeaMessage(Box::new(message))));
                }

                let nmea_log = RawNmeaLog {
                    sentence: sentence.to_string(),
                    status: raw_status,
                };

                actor.send(Event::App(AppEvent::RawNmeaSentence(nmea_log)));
            }
        }

        line.clear();
    }

    Ok(())

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
    //             // Atajamos los casos que nos interesarían, pero como no son soportados por la
    // librería nmea, los             // marcamos como tales, dejando a todos los demás no
    // categorizados.             raw_status = match err {
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

fn is_read_timeout(err: &io::Error) -> bool {
    matches!(
        err.kind(),
        io::ErrorKind::TimedOut | io::ErrorKind::WouldBlock
    )
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

#[cfg(test)]
mod tests {
    use std::sync::mpsc;
    use std::thread;
    use std::time::Duration;

    use chrono::NaiveDate;
    use clap::Parser;

    use super::*;
    use crate::cli::Cli;

    const RMC: &str = "$GPRMC,092750.000,A,5321.6802,N,00630.3372,W,0.02,31.66,280511,,,A*43";
    const GGA: &str = "$GPGGA,092750.000,5321.6802,N,00630.3372,W,1,8,1.03,61.7,M,55.2,M,,*76";
    const GSA: &str = "$GPGSA,A,3,02,05,09,12,17,19,23,28,,,,,1.5,0.9,1.2*34";
    const GSV_CYCLE: [&str; 3] = [
        "$GPGSV,3,1,10,2,45,120,45,5,30,250,40,9,60,300,42,12,15,180,35*4C",
        "$GPGSV,3,2,10,17,25,45,38,19,50,210,44,23,10,315,30,28,40,90,41*7A",
        "$GPGSV,3,3,10,31,20,270,39,32,35,150,37*70",
    ];

    fn app() -> App {
        let args = Cli::try_parse_from(["nmeagc", "-i"]).unwrap();
        App::new(args)
    }

    fn gga(sentence: &str) -> GgaData {
        match nmea::parse_str(sentence).unwrap() {
            ParseResult::GGA(data) => data,
            _ => unreachable!("expected a GGA sentence"),
        }
    }

    fn rmc(sentence: &str) -> RmcData {
        match nmea::parse_str(sentence).unwrap() {
            ParseResult::RMC(data) => data,
            _ => unreachable!("expected an RMC sentence"),
        }
    }

    fn gsa(sentence: &str) -> GsaData {
        match nmea::parse_str(sentence).unwrap() {
            ParseResult::GSA(data) => data,
            _ => unreachable!("expected a GSA sentence"),
        }
    }

    fn gsv(sentence: &str) -> GsvData {
        match nmea::parse_str(sentence).unwrap() {
            ParseResult::GSV(data) => data,
            _ => unreachable!("expected a GSV sentence"),
        }
    }

    #[test]
    fn gga_updates_navigation_data() {
        let mut app = app();

        app.update_from_gga(&gga(GGA));

        let latitude = app.nav_data.latitude.unwrap();
        let longitude = app.nav_data.longitude.unwrap();

        assert_eq!(app.nav_data.solution_type, SolutionType::Sps3D);
        assert!((latitude - 53.3613366667).abs() < 1e-9);
        assert!((longitude - -6.50562).abs() < 1e-9);
        assert_eq!(app.nav_data.altitude, Some(f64::from(61.7_f32)));
        assert_eq!(app.nav_data.geoid_separation, Some(f64::from(55.2_f32)));
        assert_eq!(app.nav_data.svs_used, 8);
        // GGA has no date, so the UTC timestamp can't be built from it alone
        assert_eq!(app.nav_data.time, None);
    }

    #[test]
    fn gga_time_uses_date_from_last_rmc() {
        let mut app = app();

        app.update_from_rmc(&rmc(RMC));
        app.update_from_gga(&gga(GGA));

        let expected = NaiveDate::from_ymd_opt(2011, 5, 28)
            .unwrap()
            .and_hms_opt(9, 27, 50)
            .unwrap()
            .and_utc();

        assert_eq!(app.nav_data.time, Some(expected));
    }

    #[test]
    fn gsv_parts_are_accumulated_and_deduplicated() {
        let mut app = app();

        for part in GSV_CYCLE {
            app.update_from_gsv(&gsv(part));
        }

        let svids = app
            .sv_data
            .iter()
            .map(|sv| (sv.gnss, sv.svid))
            .collect::<Vec<_>>();
        let expected = [
            (Gnss::Gps, 2),
            (Gnss::Gps, 5),
            (Gnss::Gps, 9),
            (Gnss::Gps, 12),
            (Gnss::Gps, 17),
            (Gnss::Gps, 19),
            (Gnss::Gps, 23),
            (Gnss::Gps, 28),
            (Gnss::Gps, 31),
            (Gnss::Gps, 32),
        ];
        assert_eq!(svids, expected);

        let sv = app.sv_data.iter().find(|sv| sv.svid == 2).unwrap();
        assert_eq!(sv.elevation, Some(45.0));
        assert_eq!(sv.azimuth, Some(120.0));
        assert_eq!(sv.signals.len(), 1);
        assert_eq!(sv.signals[0].cn0, 45.0);
        assert!(!sv.signals[0].is_used);

        // Repeating the cycle updates the existing satellites instead of duplicating them.
        for part in GSV_CYCLE {
            app.update_from_gsv(&gsv(part));
        }
        assert_eq!(app.sv_data.len(), expected.len());
    }

    #[test]
    fn old_gsv_cycle_is_dropped() {
        let mut app = app();

        for part in GSV_CYCLE {
            app.update_from_gsv(&gsv(part));
        }

        for part in [
            "$GPGSV,3,1,03,01,10,100,30*4A",
            "$GPGSV,3,2,03,02,20,200,31*4B",
            "$GPGSV,3,3,03,03,30,300,32*48",
        ] {
            app.update_from_gsv(&gsv(part));
        }

        let svids = app.sv_data.iter().map(|sv| sv.svid).collect::<Vec<_>>();
        assert_eq!(svids, [1, 2, 3]);
    }

    #[test]
    fn incomplete_gsv_cycle_does_not_mix_with_previous_data() {
        let mut app = app();

        for part in GSV_CYCLE {
            app.update_from_gsv(&gsv(part));
        }

        // A cycle with a missing part is discarded instead of being combined with the previous
        // complete one, so the previous satellites remain visible.
        app.update_from_gsv(&gsv("$GPGSV,3,1,03,01,10,100,30*4A"));
        app.update_from_gsv(&gsv("$GPGSV,3,3,03,03,30,300,32*48"));

        let svids = app.sv_data.iter().map(|sv| sv.svid).collect::<Vec<_>>();
        assert_eq!(svids, [2, 5, 9, 12, 17, 19, 23, 28, 31, 32]);

        // Once the missing part arrives, the new cycle replaces the previous data as a whole.
        app.update_from_gsv(&gsv("$GPGSV,3,2,03,02,20,200,31*4B"));

        let svids = app.sv_data.iter().map(|sv| sv.svid).collect::<Vec<_>>();
        assert_eq!(svids, [1, 2, 3]);
    }

    #[test]
    fn gsv_parts_without_a_cycle_start_are_ignored() {
        let mut app = app();

        // This part cannot be attributed to any cycle because its first sentence was not seen.
        app.update_from_gsv(&gsv("$GPGSV,3,2,03,02,20,200,31*4B"));
        assert!(app.sv_data.is_empty());

        for part in [
            "$GPGSV,3,1,03,01,10,100,30*4A",
            "$GPGSV,3,2,03,02,20,200,31*4B",
            "$GPGSV,3,3,03,03,30,300,32*48",
        ] {
            app.update_from_gsv(&gsv(part));
        }

        let svids = app.sv_data.iter().map(|sv| sv.svid).collect::<Vec<_>>();
        assert_eq!(svids, [1, 2, 3]);
    }

    #[test]
    fn duplicated_satellite_keeps_the_newest_report() {
        let mut app = app();

        // PRN 02 appears in both parts of the cycle; the one from the later sentence wins.
        for part in [
            "$GPGSV,2,1,02,02,10,100,20,05,20,200,30*7E",
            "$GPGSV,2,2,02,02,40,400,50*4C",
        ] {
            app.update_from_gsv(&gsv(part));
        }

        let sv = app.sv_data.iter().find(|sv| sv.svid == 2).unwrap();
        assert_eq!(sv.elevation, Some(40.0));
        assert_eq!(sv.signals[0].cn0, 50.0);
        assert_eq!(app.sv_data.len(), 2);
    }

    #[test]
    fn satellites_with_the_same_svid_in_different_constellations_are_kept() {
        let mut app = app();

        app.update_from_gsv(&gsv("$GPGSV,1,1,01,02,20,200,30*49"));
        app.update_from_gsv(&gsv("$GLGSV,1,1,01,02,20,200,30*55"));

        let satellites = app
            .sv_data
            .iter()
            .map(|sv| (sv.gnss, sv.svid))
            .collect::<Vec<_>>();
        assert_eq!(satellites, [(Gnss::Gps, 2), (Gnss::Glonass, 2)]);
    }

    #[test]
    fn gsa_marks_the_satellites_used_in_the_solution() {
        let mut app = app();

        for part in GSV_CYCLE {
            app.update_from_gsv(&gsv(part));
        }
        app.update_from_gsa(&gsa(GSA));

        let used = app
            .sv_data
            .iter()
            .filter(|sv| sv.signals[0].is_used)
            .map(|sv| sv.svid)
            .collect::<Vec<_>>();
        assert_eq!(used, [2, 5, 9, 12, 17, 19, 23, 28]);

        // A new GSA for the same constellation replaces the previous used set.
        app.update_from_gsa(&gsa("$GPGSA,A,3,02,,,,,,,,,,,,1.5,0.9,1.2*3E"));

        let used = app
            .sv_data
            .iter()
            .filter(|sv| sv.signals[0].is_used)
            .map(|sv| sv.svid)
            .collect::<Vec<_>>();
        assert_eq!(used, [2]);
    }

    #[test]
    fn gsa_received_before_gsv_is_applied_to_new_satellites() {
        let mut app = app();

        app.update_from_gsa(&gsa(GSA));

        for part in GSV_CYCLE {
            app.update_from_gsv(&gsv(part));
        }

        let used = app
            .sv_data
            .iter()
            .filter(|sv| sv.signals[0].is_used)
            .map(|sv| sv.svid)
            .collect::<Vec<_>>();
        assert_eq!(used, [2, 5, 9, 12, 17, 19, 23, 28]);
    }

    struct TimeoutReader;

    impl io::Read for TimeoutReader {
        fn read(&mut self, _buf: &mut [u8]) -> io::Result<usize> {
            Err(io::Error::new(io::ErrorKind::WouldBlock, "timeout"))
        }
    }

    impl io::BufRead for TimeoutReader {
        fn fill_buf(&mut self) -> io::Result<&[u8]> {
            Err(io::Error::new(io::ErrorKind::WouldBlock, "timeout"))
        }

        fn consume(&mut self, _amt: usize) {}
    }

    #[test]
    fn read_timeouts_are_retryable() {
        assert!(is_read_timeout(&io::Error::from(io::ErrorKind::TimedOut)));
        assert!(is_read_timeout(&io::Error::from(io::ErrorKind::WouldBlock)));
        assert!(!is_read_timeout(&io::Error::from(
            io::ErrorKind::UnexpectedEof
        )));
    }

    #[test]
    fn handler_exits_after_shutdown_when_reader_times_out() {
        let (sender, _receiver) = mpsc::channel();
        let actor = EventThread::new(sender);
        let running = Arc::new(AtomicBool::new(true));
        let worker_running = Arc::clone(&running);

        let handle = thread::spawn(move || run_nmea_handler(actor, worker_running, TimeoutReader));

        thread::sleep(Duration::from_millis(50));
        running.store(false, Ordering::Relaxed);

        let result = handle.join().expect("worker thread panicked");
        assert!(result.is_ok());
    }
}
