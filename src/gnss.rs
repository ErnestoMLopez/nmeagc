use chrono::{DateTime, NaiveTime, Utc};
use strum::{EnumIter, IntoStaticStr};

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, EnumIter)]
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
            Gnss::Other => "UNK",
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

/// Navigation solution type.
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

#[derive(Debug, Clone)]
pub struct SignalData {
    /// GNSS signal identifier
    pub signal: GnssSignal,
    /// Carrier-to-noise density ratio [dB-Hz]
    pub cn0: f32,
    /// Indicates if the signal is used in the navigation solution
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
    signals: Vec<SignalData>,
    /// Elevation [degrees]
    pub elevation: Option<f32>,
    /// Azimuth [degrees]
    pub azimuth: Option<f32>,
}

impl SvData {
    /// Creates a satellite entry without signals or geometry.
    pub fn new(gnss: Gnss, svid: u8) -> Self {
        Self {
            gnss,
            svid,
            channel: None,
            signals: Vec::new(),
            elevation: None,
            azimuth: None,
        }
    }

    /// Updates the position of the satellite in the sky.
    pub fn update_geometry(&mut self, elevation: Option<f32>, azimuth: Option<f32>) {
        self.elevation = elevation;
        self.azimuth = azimuth;
    }

    /// Returns the signals tracked for the satellite.
    pub fn signals(&self) -> &[SignalData] {
        &self.signals
    }

    /// Updates a signal observation, adding the signal if it is not tracked yet.
    ///
    /// Fields that are `None` keep their current value. A signal can only be added when a C/N0
    /// is provided, so an update that only carries the used state never creates an untracked
    /// signal.
    pub fn update_signal(&mut self, signal: GnssSignal, cn0: Option<f32>, is_used: Option<bool>) {
        if let Some(data) = self.signals.iter_mut().find(|data| data.signal == signal) {
            if let Some(cn0) = cn0 {
                data.cn0 = cn0;
            }
            if let Some(is_used) = is_used {
                data.is_used = is_used;
            }
        } else if let Some(cn0) = cn0 {
            let data = SignalData {
                signal,
                cn0,
                is_used: is_used.unwrap_or_default(),
            };

            self.signals.push(data);
        }
    }
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

impl NavigationData {
    /// Updates the solution type and the number of satellites used.
    pub fn update_solution(&mut self, solution_type: SolutionType, svs_used: u8) {
        self.solution_type = solution_type;
        self.svs_used = svs_used;
    }

    /// Updates the UTC timestamp.
    pub fn update_time(&mut self, time: Option<DateTime<Utc>>) {
        self.time = time;
    }

    /// Updates the time of day, keeping the date of the last known timestamp.
    ///
    /// This allows sources that only provide the time of day to update the timestamp once a full
    /// timestamp is known from another source.
    pub fn update_time_of_day(&mut self, time: Option<NaiveTime>) {
        self.time = time.and_then(|time| {
            let date = self.time?.date_naive();
            Some(date.and_time(time).and_utc())
        });
    }

    /// Updates the geodetic position.
    pub fn update_position(&mut self, latitude: Option<f64>, longitude: Option<f64>) {
        self.latitude = latitude;
        self.longitude = longitude;
    }

    /// Updates the altitude and the geoid-ellipsoid separation.
    pub fn update_altitude(&mut self, altitude: Option<f64>, geoid_separation: Option<f64>) {
        self.altitude = altitude;
        self.geoid_separation = geoid_separation;
    }
}

#[cfg(test)]
mod tests {
    use chrono::NaiveDate;

    use super::*;

    fn datetime(hour: u32, minute: u32, second: u32) -> DateTime<Utc> {
        NaiveDate::from_ymd_opt(2026, 11, 21)
            .unwrap()
            .and_hms_opt(hour, minute, second)
            .unwrap()
            .and_utc()
    }

    #[test]
    fn time_of_day_update_preserves_the_known_date() {
        let mut nav_data = NavigationData::default();
        nav_data.update_time(Some(datetime(12, 0, 0)));
        nav_data.update_time_of_day(Some(NaiveTime::from_hms_opt(18, 30, 15).unwrap()));

        assert_eq!(nav_data.time, Some(datetime(18, 30, 15)));
    }

    #[test]
    fn time_of_day_update_without_a_date_clears_the_timestamp() {
        let mut nav_data = NavigationData::default();
        nav_data.update_time_of_day(Some(NaiveTime::from_hms_opt(18, 30, 15).unwrap()));

        assert_eq!(nav_data.time, None);
    }

    #[test]
    fn satellite_signal_update_adds_and_updates_signals() {
        let mut sv = SvData::new(Gnss::Gps, 2);
        sv.update_geometry(Some(45.0), Some(120.0));
        sv.update_signal(GnssSignal::Gps(GpsSignal::L1CA), Some(30.0), Some(false));
        sv.update_signal(GnssSignal::Gps(GpsSignal::L1CA), Some(45.0), Some(true));
        sv.update_signal(GnssSignal::Gps(GpsSignal::L5I), Some(20.0), None);

        assert_eq!(sv.elevation, Some(45.0));
        assert_eq!(sv.azimuth, Some(120.0));
        assert_eq!(sv.signals().len(), 2);

        let l1ca = sv
            .signals()
            .iter()
            .find(|signal| signal.signal == GnssSignal::Gps(GpsSignal::L1CA))
            .unwrap();
        assert_eq!(l1ca.cn0, 45.0);
        assert!(l1ca.is_used);

        // Fields that are not provided keep their current value.
        sv.update_signal(GnssSignal::Gps(GpsSignal::L1CA), None, Some(false));
        let l1ca = sv
            .signals()
            .iter()
            .find(|signal| signal.signal == GnssSignal::Gps(GpsSignal::L1CA))
            .unwrap();
        assert_eq!(l1ca.cn0, 45.0);
        assert!(!l1ca.is_used);
    }

    #[test]
    fn satellite_signal_update_uses_the_constellation_default_signal() {
        let mut sv = SvData::new(Gnss::Glonass, 3);
        sv.update_signal(GnssSignal::from(Gnss::Glonass), Some(25.0), Some(true));

        assert_eq!(sv.signals().len(), 1);
        assert_eq!(sv.signals()[0].signal, GnssSignal::from(Gnss::Glonass));
        assert_eq!(sv.signals()[0].cn0, 25.0);
        assert!(sv.signals()[0].is_used);
    }

    #[test]
    fn satellite_signal_without_cn0_does_not_add_it() {
        let mut sv = SvData::new(Gnss::Gps, 2);

        // A used-state update cannot create a signal without tracking information.
        sv.update_signal(GnssSignal::Gps(GpsSignal::L1CA), None, Some(true));

        assert!(sv.signals().is_empty());
    }
}
