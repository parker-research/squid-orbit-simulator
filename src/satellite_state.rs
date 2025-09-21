use satkit::ITRFCoord;
use satkit::consts::{EARTH_RADIUS, SUN_RADIUS, WGS84_A};
use satkit::frametransform::qgcrf2itrf;
use satkit::frametransform::qteme2itrf;
use satkit::lpephem::sun::pos_gcrf;
use satkit::sgp4::{SGP4Error, sgp4};
use satkit::{Instant, types::Vec3};

use crate::initial_state_model::InitialSimulationState;

pub fn pythag_3(vector: &[f64; 3]) -> f64 {
    f64::sqrt(vector[0].powi(2) + vector[1].powi(2) + vector[2].powi(2))
}

/// Calculate the elevation (above sea level) in kilometers from the position vector in kilometers.
pub fn calculate_elevation_from_location_km(position_km: &[f64; 3]) -> f64 {
    let earth_radius_km = EARTH_RADIUS / 1000.0;
    let radius_km = pythag_3(position_km);

    let elevation_km = radius_km - earth_radius_km;
    elevation_km
}

/// Compute the satellite's local solar time in hours [0, 24).
pub fn calculate_local_solar_time_hours(longitude_deg: f64, time: &Instant) -> f64 {
    let jd = time.as_jd() + 0.5; // jd=0.0 happens at 12:00 UTC
    let fractional_day = jd.fract();
    let utc_hours = fractional_day * 24.0;
    let local_time = utc_hours + longitude_deg / 15.0;
    (local_time + 24.0) % 24.0
}

/// Calculate the elevation angle in degrees from a satellite's position to a ground station.
/// The elevation angle is the angle above the local horizontal plane at the ground station.
/// When >= 0 degrees, the satellite is above the horizon, and the ground station can communicate with it.
pub fn calculate_elevation_angle_degrees(
    position_km: &[f64; 3],
    ground_station: &crate::initial_state_model::GroundStation,
) -> f64 {
    let station_ecef_xyz_m = ground_station.ecef_xyz_m();
    let vector_to_satellite_m = [
        position_km[0] * 1000.0 - station_ecef_xyz_m[0],
        position_km[1] * 1000.0 - station_ecef_xyz_m[1],
        position_km[2] * 1000.0 - station_ecef_xyz_m[2],
    ];

    // Compute norms (magnitudes)
    let norm_ground = (station_ecef_xyz_m.iter().map(|x| x * x).sum::<f64>()).sqrt();
    let norm_los = (vector_to_satellite_m.iter().map(|x| x * x).sum::<f64>()).sqrt();

    // Compute dot product of ground station vector and line-of-sight vector
    let dot_product: f64 = station_ecef_xyz_m
        .iter()
        .zip(vector_to_satellite_m.iter())
        .map(|(a, b)| a * b)
        .sum();

    // Angle between vectors
    let cos_theta = dot_product / (norm_ground * norm_los);
    let theta_rad = cos_theta.acos(); // This is the angle from zenith

    // Convert to elevation angle: π/2 - θ (from zenith to elevation)
    let elevation_rad = std::f64::consts::FRAC_PI_2 - theta_rad;

    // Convert to degrees
    elevation_rad.to_degrees()
}

pub fn calculate_power_from_atmospheric_drag_watts(
    satellite: &crate::initial_state_model::Satellite,
    elevation_km: f64,
    latitude_deg: Option<f64>,
    longitude_deg: Option<f64>,
    speed_m_per_s: f64,
    time: Option<satkit::Instant>,
    enable_space_weather: bool,
) -> f64 {
    let (rho_density_kg_per_m3, _temperature_kelvin) = // TODO: Encorporate space weather data by passing in a date.
        satkit::nrlmsise::nrlmsise(elevation_km, latitude_deg, longitude_deg, time, enable_space_weather);

    let power_watts = 0.5
        * satellite.drag_coefficient
        * rho_density_kg_per_m3
        * satellite.drag_area_m2
        * speed_m_per_s.powi(3);
    power_watts
}

/// Estimate solar irradiance (W/m²) at the satellite's location, accounting for eclipse by Earth.
///
/// Returns 1361.0 in full sunlight, 0.0 in umbra, or a partial value in penumbra.
pub fn calculate_sun_irradiance_received_approx_w_per_m2(
    satellite_position_itrf_m: &[f64; 3],
    time: &Instant,
) -> f64 {
    const SOLAR_CONSTANT_W_PER_M2: f64 = 1361.0;
    // FIXME: This function appears to not work, and must be revisited/reimplemented/re-evaluated.

    // Step 1: Get Sun position in GCRF (in meters).
    let sun_gcrf_m: Vec3 = pos_gcrf(time);

    // Step 2: Transform Sun position from GCRF to ITRF.
    let transform_gcrf_to_itrf = qgcrf2itrf(time).to_rotation_matrix();
    let sun_itrf_m = transform_gcrf_to_itrf * sun_gcrf_m;

    // Step 3: Compute unit vectors and geometry.
    let sat_itrf_vec = nalgebra::Vector3::<f64>::from_row_slice(satellite_position_itrf_m);
    // Note: Must reconstruct the following as different nalgebra versions are used across crates.
    let sun_itrf_vec = nalgebra::Vector3::<f64>::from_row_slice(sun_itrf_m.as_slice());

    let sat_mag_m = sat_itrf_vec.norm(); // Distance from Earth center to Satellite.
    let sun_mag_m = sun_itrf_vec.norm(); // Distance from Earth center to Sun.

    // Angle between Earth→Sat vector and Earth→Sun vector.
    let cos_theta = (sat_itrf_vec.dot(&sun_itrf_vec) / (sat_mag_m * sun_mag_m)).clamp(-1.0, 1.0);
    let sun_earth_sat_angle_rad = cos_theta.acos();

    assert!(
        sun_mag_m > 0.9 * 1.496e11 && sun_mag_m < 1.1 * 1.496e11,
        "Sun-Earth distance is not within expected range (1 AU)."
    );
    assert!(
        sat_mag_m > EARTH_RADIUS && sat_mag_m < 5.0 * EARTH_RADIUS,
        "Satellite distance is not within expected range (above Earth's surface, max 5 Earth radii)."
    );

    // Angular radii.
    let alpha = (SUN_RADIUS / sun_mag_m).asin(); // Sun's angular radius
    let beta = (WGS84_A / sat_mag_m).asin(); // Earth's angular radius

    if sun_earth_sat_angle_rad < alpha - beta {
        // Full sunlight.
        SOLAR_CONSTANT_W_PER_M2
    } else if sun_earth_sat_angle_rad < alpha + beta {
        // Partial shadow (penumbra) - crude linear taper.
        // let visible_fraction =
        //     1.0 - ((sun_earth_sat_angle_rad - (alpha - beta)) / (2.0 * beta)).clamp(0.0, 1.0);

        let delta = sun_earth_sat_angle_rad - (alpha - beta);
        let visible_fraction = 1.0 - (delta / (2.0 * beta)).powi(2).clamp(0.0, 1.0);
        visible_fraction * SOLAR_CONSTANT_W_PER_M2
    } else {
        // Full umbra.
        0.0
    }
}

/// Estimate solar irradiance (W/m²) at the satellite's location, accounting for eclipse by Earth.
///
/// Returns 1361.0 in full sunlight, 0.0 in umbra, or a partial value in penumbra.
pub fn calculate_sun_irradiance_received_w_per_m2(
    satellite_position_itrf_m: &[f64; 3],
    time: &Instant,
) -> f64 {
    const SOLAR_CONSTANT_W_PER_M2: f64 = 1361.0;

    let sun_itrf_m = {
        // Step 1: Get Sun position in GCRF (in meters).
        let sun_gcrf_m: Vec3 = pos_gcrf(time);
        // Step 2: Transform Sun position from GCRF to ITRF.
        let transform_gcrf_to_itrf = qgcrf2itrf(time).to_rotation_matrix();
        transform_gcrf_to_itrf * sun_gcrf_m
    };

    // Step 3: Compute unit vectors and geometry.
    let sat_itrf_vec = nalgebra::Vector3::<f64>::from_row_slice(satellite_position_itrf_m);
    // Note: Must reconstruct the following as different nalgebra versions are used across crates.
    let sun_itrf_vec = nalgebra::Vector3::<f64>::from_row_slice(sun_itrf_m.as_slice());

    let sat_mag_m = sat_itrf_vec.norm(); // Distance from Earth center to Satellite.
    let sun_mag_m = sun_itrf_vec.norm(); // Distance from Earth center to Sun.

    assert!(
        sun_mag_m > 0.9 * 1.496e11 && sun_mag_m < 1.1 * 1.496e11,
        "Sun-Earth distance is not within expected range (1 AU)."
    );
    assert!(
        sat_mag_m > EARTH_RADIUS && sat_mag_m < 5.0 * EARTH_RADIUS,
        "Satellite distance is not within expected range (above Earth's sea level, max 5 Earth radii)."
    );

    let r_hat = sun_itrf_vec / sun_mag_m; // unit vector Earth → Sun
    let proj_length = sat_itrf_vec.dot(&r_hat); // distance along Earth→Sun axis
    let perpendicular_vector = sat_itrf_vec - proj_length * r_hat;
    let perpendicular_dist = perpendicular_vector.norm();

    let theta_umbra = (EARTH_RADIUS - SUN_RADIUS) / sun_mag_m; // small angle approximation // TODO: Use real calc.
    let r_umbra = (proj_length - sun_mag_m) * theta_umbra;
    let theta_penumbra = (EARTH_RADIUS + SUN_RADIUS) / sun_mag_m;
    let r_penumbra = (proj_length - sun_mag_m) * theta_penumbra;

    if proj_length < 0.0 {
        // Satellite is between Earth and Sun → always in sunlight.
        SOLAR_CONSTANT_W_PER_M2
    } else if perpendicular_dist < r_umbra {
        // Inside umbra
        0.0
    } else if perpendicular_dist < r_penumbra {
        // Inside penumbra
        let fraction = (perpendicular_dist - r_umbra) / (r_penumbra - r_umbra);
        let visible_fraction = 1.0 - fraction.clamp(0.0, 1.0);
        visible_fraction * SOLAR_CONSTANT_W_PER_M2
    } else {
        // Outside shadow cones → full sunlight
        SOLAR_CONSTANT_W_PER_M2
    }
}

#[derive(Debug, Clone)]
pub struct SimulationStateAtStep {
    pub time: Instant,
    pub hours_since_epoch: f64,
    pub position_itrf: [f64; 3],
    pub velocity_itrf: [f64; 3],
    pub speed_m_per_s: f64,
    pub elevation_km: f64,
    pub elevation_angles_degrees: Vec<f64>,
    pub drag_power_watts: f64,
    pub irradiance_approx_w_per_m2: f64,
    pub irradiance_w_per_m2: f64,
    pub local_time_hours: f64,
    pub is_deorbited: bool,
}

// --- Stateful simulator ---
#[derive(Debug)]
pub struct SimulationRun {
    // Fixed inputs
    pub initial: InitialSimulationState,

    // Evolving state
    satkit_tle_mut: satkit::TLE,
    current_sim_time: Instant,

    pub latest_telemetry: Option<SimulationStateAtStep>,
}

impl SimulationRun {
    /// Seed a new run from the initial state bundle.
    pub fn new(initial: InitialSimulationState) -> Self {
        let epoch = initial.tle.epoch;
        Self {
            satkit_tle_mut: initial.tle.to_satkit_tle(),
            initial,
            current_sim_time: epoch,
            latest_telemetry: None,
        }
    }

    pub fn hours_since_epoch(&self) -> f64 {
        (self.current_sim_time - self.initial.tle.epoch).as_hours()
    }

    /// Advance one simulation step.
    ///
    /// Returns per-step telemetry. `telemetry.deorbited == true` when elevation < 100 km.
    pub fn step(&mut self) -> anyhow::Result<SimulationStateAtStep> {
        let settings = &self.initial.simulation_settings;
        let gs = &self.initial.ground_stations;
        let sat = &self.initial.satellite;

        let time = self.current_sim_time;

        // SGP4 over a single timestamp (slice)
        let (position_teme, velocity_teme, errs) = sgp4(&mut self.satkit_tle_mut, &[time]);
        if let Some(err) = errs.first() {
            if *err != SGP4Error::SGP4Success {
                return Err(anyhow::anyhow!("SGP4 error: {}", err));
            }
        }

        // Transform TEME -> ITRF (cap matrix far in the future like your original)
        let max_tf_time = Instant::new(1767250888000 * 1000);
        let tf_time = if time < max_tf_time {
            time
        } else {
            max_tf_time
        };
        let transform_matrix = qteme2itrf(&tf_time).to_rotation_matrix();
        let position_itrf_matrix = transform_matrix * position_teme;
        let velocity_itrf_matrix = transform_matrix * velocity_teme;

        let position_itrf = ITRFCoord::from_slice(position_itrf_matrix.as_slice()).unwrap();
        let velocity_itrf = ITRFCoord::from_slice(velocity_itrf_matrix.as_slice()).unwrap();

        let speed_m_per_s = pythag_3(&[
            velocity_itrf.itrf[0],
            velocity_itrf.itrf[1],
            velocity_itrf.itrf[2],
        ]);

        let position_km = [
            position_itrf.itrf[0] / 1000.0,
            position_itrf.itrf[1] / 1000.0,
            position_itrf.itrf[2] / 1000.0,
        ];

        let elevation_km = calculate_elevation_from_location_km(&position_km);

        let elevation_angles_degrees = gs
            .iter()
            .map(|station| calculate_elevation_angle_degrees(&position_km, station))
            .collect::<Vec<_>>();

        let drag_power_watts = calculate_power_from_atmospheric_drag_watts(
            sat,
            elevation_km,
            Some(position_itrf.latitude_deg()),
            Some(position_itrf.longitude_deg()),
            speed_m_per_s,
            Some(time),
            settings.drag_power_enable_space_weather,
        );

        let local_time_hours: f64 =
            calculate_local_solar_time_hours(position_itrf.longitude_deg(), &time);

        // (Keep your prints for now; you can remove or gate them with a flag later.)
        println!(
            "Time: TLE Epoch + {:.2} days = {:.2} years => UTC {} => Local Time: {:.2}h = {}:{:02}",
            self.hours_since_epoch() / 24.0,
            self.hours_since_epoch() / (24.0 * 365.0),
            time,
            local_time_hours,
            local_time_hours.floor() as u32,
            (local_time_hours % 1.0 * 60.0).round() as u32
        );
        println!("Position: {}", position_itrf);
        println!(
            "Position: {:?} km = {:.2} km = ({:.5}, {:.5}, h={:.3} km)",
            position_km,
            elevation_km,
            position_itrf.latitude_deg(),
            position_itrf.longitude_deg(),
            position_itrf.hae() / 1000.0
        );
        println!(
            "Velocity: {:?} km/s = {:.2} km/s",
            [
                velocity_itrf.itrf[0] / 1000.0,
                velocity_itrf.itrf[1] / 1000.0,
                velocity_itrf.itrf[2] / 1000.0
            ],
            speed_m_per_s / 1000.0
        );
        println!(
            "Drag Power: {:.3} W (Elevation: {:.2} km, Speed: {:.2} km/s)",
            drag_power_watts,
            elevation_km,
            speed_m_per_s / 1000.0
        );

        let irradiance_approx_w_per_m2 = calculate_sun_irradiance_received_approx_w_per_m2(
            &[
                position_itrf.itrf[0],
                position_itrf.itrf[1],
                position_itrf.itrf[2],
            ],
            &time,
        );
        let irradiance_w_per_m2 = calculate_sun_irradiance_received_w_per_m2(
            &[
                position_itrf.itrf[0],
                position_itrf.itrf[1],
                position_itrf.itrf[2],
            ],
            &time,
        );
        println!(
            "Solar Irradiance (Way 1, Approx): {:.2} W/m²",
            irradiance_approx_w_per_m2
        );
        println!(
            "Solar Irradiance (Way 2, Cones): {:.2} W/m²",
            irradiance_w_per_m2
        );

        for (station, angle_deg) in gs.iter().zip(elevation_angles_degrees.iter().copied()) {
            println!(
                "Ground station \"{}\" -> {} Elevation: {:.2} degrees (Distance: {:.2} km)",
                station.name,
                if angle_deg > station.min_elevation_deg {
                    "✅"
                } else {
                    "❌"
                },
                angle_deg,
                pythag_3(&[
                    position_km[0] - station.ecef_xyz_m()[0] / 1000.0,
                    position_km[1] - station.ecef_xyz_m()[1] / 1000.0,
                    position_km[2] - station.ecef_xyz_m()[2] / 1000.0,
                ])
            );
        }
        println!();

        let is_deorbited = elevation_km < 100.0;
        if is_deorbited {
            println!(
                "Deorbit achieved at {:.2} days = {:.2} years since epoch = {}",
                self.hours_since_epoch() / 24.0,
                self.hours_since_epoch() / (24.0 * 365.0),
                time
            );
        }

        // Advance the clock for the *next* call to step().
        self.current_sim_time += satkit::Duration::from_hours(settings.step_interval_hours);

        let simulation_state = SimulationStateAtStep {
            time,
            hours_since_epoch: self.hours_since_epoch(), // now points to the *next* tick
            position_itrf: [
                position_itrf.itrf[0],
                position_itrf.itrf[1],
                position_itrf.itrf[2],
            ],
            velocity_itrf: [
                velocity_itrf.itrf[0],
                velocity_itrf.itrf[1],
                velocity_itrf.itrf[2],
            ],
            speed_m_per_s,
            elevation_km,
            elevation_angles_degrees,
            drag_power_watts,
            irradiance_approx_w_per_m2,
            irradiance_w_per_m2,
            local_time_hours,
            is_deorbited,
        };
        self.latest_telemetry = Some(simulation_state.clone());
        Ok(simulation_state)
    }
}
