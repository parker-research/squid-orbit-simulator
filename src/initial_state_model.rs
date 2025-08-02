use once_cell::unsync::OnceCell;

pub struct GroundStation {
    pub name: String,
    pub latitude_deg: f64,
    pub longitude_deg: f64,
    pub elevation_m: Option<f64>,
    pub altitude_m: f64,
    pub min_elevation_deg: f64,
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
    ) -> Self {
        Self {
            name,
            latitude_deg,
            longitude_deg,
            elevation_m,
            altitude_m,
            min_elevation_deg,
            ecef_cache: OnceCell::new(),
        }
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

pub struct Satellite {
    pub name: String,

    /// @brief Unitless drag coefficient of the satellite (C_d) for atmospheric drag calculations.
    pub drag_coefficient: f64,

    /// @brief Average cross-sectional area of the satellite (A) for atmospheric drag calculations.
    pub drag_area_m2: f64,
}

pub struct SimulationSettings {
    pub max_days: f64,

    pub step_interval_hours: f64,

    pub drag_power_enable_space_weather: bool,
}
