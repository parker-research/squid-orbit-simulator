use once_cell::unsync::OnceCell;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GroundStation {
    pub name: String,
    pub latitude_deg: f64,
    pub longitude_deg: f64,
    pub elevation_m: Option<f64>,
    pub altitude_m: f64,
    pub min_elevation_deg: f64,

    #[serde(skip, default)]
    ecef_cache: OnceCell<[f64; 3]>,
}

impl GroundStation {
    pub fn new(
        name: String,
        latitude_deg: f64,
        longitude_deg: f64,
        elevation_m: Option<f64>,
        altitude_m: f64,
        min_elevation_deg: f64,
    ) -> Result<Self, String> {
        if !(-90.0..=90.0).contains(&latitude_deg) {
            return Err("Latitude must be between -90 and 90 degrees".to_string());
        }
        if !(-180.0..=180.0).contains(&longitude_deg) {
            return Err("Longitude must be between -180 and 180 degrees".to_string());
        }

        Ok(Self {
            name,
            latitude_deg,
            longitude_deg,
            elevation_m,
            altitude_m,
            min_elevation_deg,
            ecef_cache: OnceCell::new(),
        })
    }

    pub fn ecef_xyz_m(&self) -> [f64; 3] {
        *self.ecef_cache.get_or_init(|| {
            let elevation_m = self.elevation_m.unwrap_or(0.0) + self.altitude_m;

            let station = nav_types::WGS84::from_degrees_and_meters(
                self.latitude_deg,
                self.longitude_deg,
                elevation_m,
            );

            let ecef = nav_types::ECEF::from(station);
            [ecef.x(), ecef.y(), ecef.z()]
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Satellite {
    pub name: String,

    /// @brief Unitless drag coefficient of the satellite (C_d) for atmospheric drag calculations.
    pub drag_coefficient: f64,

    /// @brief Average cross-sectional area of the satellite (A) for atmospheric drag calculations.
    pub drag_area_m2: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimulationSettings {
    pub max_days: f64,

    pub step_interval_hours: f64,

    pub drag_power_enable_space_weather: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TleData {
    /// Name of satellite
    pub name: String,
    /// String describing launch
    pub intl_desig: String,
    /// Satellite NORAD number
    pub sat_num: i32,
    /// Launch year
    pub desig_year: i32,
    /// Numbered launch of year
    pub desig_launch: i32,
    /// Piece of launch
    pub desig_piece: String,
    /// TLE epoch
    pub epoch: satkit::Instant,
    /// One half of 1st derivative of mean motion wrt time, in revs/day^2
    pub mean_motion_dot: f64,
    /// One sixth of 2nd derivative of mean motion wrt tim, in revs/day^3
    pub mean_motion_dot_dot: f64,
    /// Starred ballistic coefficient, in units of inverse Earth radii
    pub bstar: f64,
    /// Usually 0
    pub ephem_type: u8,
    /// Bulliten number
    pub element_num: i32,
    /// Inclination, degrees
    pub inclination: f64,
    /// Right ascension of ascending node, degrees
    pub raan: f64,
    /// Eccentricity
    pub eccen: f64,
    /// Argument of perigee, degrees
    pub arg_of_perigee: f64,
    /// Mean anomaly, degrees
    pub mean_anomaly: f64,
    /// Mean motion, revs / day
    pub mean_motion: f64,
    /// Revolution number
    pub rev_num: i32,
}

impl TleData {
    pub fn to_satkit_tle(&self) -> satkit::TLE {
        let mut satkit_tle = satkit::TLE::new();

        satkit_tle.name = self.name.clone();
        satkit_tle.intl_desig = self.intl_desig.clone();
        satkit_tle.sat_num = self.sat_num;
        satkit_tle.desig_year = self.desig_year;
        satkit_tle.desig_launch = self.desig_launch;
        satkit_tle.desig_piece = self.desig_piece.clone();
        satkit_tle.epoch = self.epoch;
        satkit_tle.mean_motion_dot = self.mean_motion_dot;
        satkit_tle.mean_motion_dot_dot = self.mean_motion_dot_dot;
        satkit_tle.bstar = self.bstar;
        satkit_tle.ephem_type = self.ephem_type;
        satkit_tle.element_num = self.element_num;
        satkit_tle.inclination = self.inclination;
        satkit_tle.raan = self.raan;
        satkit_tle.eccen = self.eccen;
        satkit_tle.arg_of_perigee = self.arg_of_perigee;
        satkit_tle.mean_anomaly = self.mean_anomaly;
        satkit_tle.mean_motion = self.mean_motion;
        satkit_tle.rev_num = self.rev_num;

        satkit_tle
    }

    pub fn from_satkit_tle(tle: &satkit::TLE) -> Self {
        Self {
            name: tle.name.clone(),
            intl_desig: tle.intl_desig.clone(),
            sat_num: tle.sat_num,
            desig_year: tle.desig_year,
            desig_launch: tle.desig_launch,
            desig_piece: tle.desig_piece.clone(),
            epoch: tle.epoch,
            mean_motion_dot: tle.mean_motion_dot,
            mean_motion_dot_dot: tle.mean_motion_dot_dot,
            bstar: tle.bstar,
            ephem_type: tle.ephem_type,
            element_num: tle.element_num,
            inclination: tle.inclination,
            raan: tle.raan,
            eccen: tle.eccen,
            arg_of_perigee: tle.arg_of_perigee,
            mean_anomaly: tle.mean_anomaly,
            mean_motion: tle.mean_motion,
            rev_num: tle.rev_num,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InitialSimulationState {
    pub tle: TleData,
    pub ground_stations: Vec<GroundStation>,
    pub satellite: Satellite,
    pub simulation_settings: SimulationSettings,
}
