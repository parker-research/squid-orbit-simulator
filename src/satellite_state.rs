use satkit::ITRFCoord;
use satkit::frametransform::qteme2itrf;
use satkit::sgp4::{SGP4Error, sgp4};
use satkit::tle::TLE;

#[allow(dead_code)]
pub fn get_sample_demo_tle() -> anyhow::Result<TLE> {
    let tle = TLE::load_3line(
        "Satellite 1",
        "1 25544U 98067A   20194.88612269 -.00002218  00000-0 -31515-4 0  9992",
        "2 25544  51.6461 221.2784 0001413  89.1723 280.4612 15.49507896236008",
    )?;
    Ok(tle)
}

#[allow(dead_code)]
pub fn get_sample_demo_tle_canx5() -> anyhow::Result<TLE> {
    let tle = TLE::load_3line(
        "CANX-5",
        "1 40056U 14034D   25209.55901054  .00000935  00000+0  13011-3 0  9999",
        "2 40056  98.3577  67.3174 0012454 336.2296  23.8338 14.80323878587440",
    )?;
    Ok(tle)
}

#[allow(dead_code)]
pub fn get_sample_demo_tle_intelsat_902() -> anyhow::Result<TLE> {
    let line0: &str = "0 INTELSAT 902";
    let line1: &str = "1 26900U 01039A   06106.74503247  .00000045  00000-0  10000-3 0  8290";
    let line2: &str =
        "2 26900   0.0164 266.5378 0003319  86.1794 182.2590  1.00273847 16981   9300.";
    let tle = TLE::load_3line(&line0.to_string(), &line1.to_string(), &line2.to_string())?;
    Ok(tle)
}

pub fn pythag_3(vector: &[f64; 3]) -> f64 {
    f64::sqrt(vector[0].powi(2) + vector[1].powi(2) + vector[2].powi(2))
}

/// Calculate the elevation (above sea level) in kilometers from the position vector in kilometers.
pub fn calculate_elevation_from_location_km(position_km: &[f64; 3]) -> f64 {
    let earth_radius_km = sgp4::WGS84.ae; // Earth's radius in km
    let radius_km = pythag_3(position_km);

    let elevation_km = radius_km - earth_radius_km;
    elevation_km
}

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
) -> f64 {
    let (rho_density_kg_per_m3, _temperature_kelvin) = // TODO: Encorporate space weather data by passing in a date.
        satkit::nrlmsise::nrlmsise(elevation_km, latitude_deg, longitude_deg, time, true);

    let power_watts = 0.5
        * satellite.drag_coefficient
        * rho_density_kg_per_m3
        * satellite.drag_area_m2
        * speed_m_per_s.powi(3);
    power_watts
}

pub fn propagate_to_deorbit(
    satellite: &crate::initial_state_model::Satellite,
    tle: &TLE,
    max_hours: f64,
    ground_stations: &[crate::initial_state_model::GroundStation],
) -> anyhow::Result<f64> {
    let epoch = tle.epoch;

    let mut hours_since_epoch: f64 = 0.0;

    let mut tle_mut = tle.clone();

    while hours_since_epoch < max_hours {
        let time = epoch + satkit::Duration::from_hours(hours_since_epoch);

        // SGP4 runs on a slice of times
        let (position_teme, velocity_teme, errs) = sgp4(&mut tle_mut, &[time]);
        if let Some(err) = errs.first() {
            if *err != SGP4Error::SGP4Success {
                return Err(anyhow::anyhow!("SGP4 error: {}", err));
            }
        }
        // Fetch a transform matrix, maxing out when it's too far into the future.
        let transform_matrix = if time < satkit::Instant::new(1767250888000 * 1000) {
            qteme2itrf(&time).to_rotation_matrix()
        } else {
            qteme2itrf(&satkit::Instant::new(1767250888000 * 1000)).to_rotation_matrix()
        };
        let position_itrf_matrix = transform_matrix * position_teme;
        let velocity_itrf_matrix = transform_matrix * velocity_teme;

        let position_itrf = ITRFCoord::from_slice(position_itrf_matrix.as_slice()).unwrap();
        let velocity_itrf = ITRFCoord::from_slice(velocity_itrf_matrix.as_slice()).unwrap(); // TODO: Probably just get the array out of the Matrix
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

        let elevation_angles_degrees = ground_stations
            .iter()
            .map(|station| calculate_elevation_angle_degrees(&position_km, station))
            .collect::<Vec<_>>();

        let drag_power_watts = calculate_power_from_atmospheric_drag_watts(
            satellite,
            elevation_km,
            Some(position_itrf.latitude_deg()),
            Some(position_itrf.longitude_deg()),
            speed_m_per_s,
            Some(time),
        );

        println!(
            "Time = {:.2} days = {:.2} years = {}",
            hours_since_epoch / 24.0,
            hours_since_epoch / (24.0 * 365.0),
            time
        );
        println!("Position = {}", position_itrf);
        println!(
            "Position = {:?} km = {:.2} km = ({:.5}, {:.5}, h={:.3} km)",
            position_km,
            elevation_km,
            position_itrf.latitude_deg(),
            position_itrf.longitude_deg(),
            position_itrf.hae() / 1000.0
        );
        println!(
            "Velocity = {:?} km/s = {:.2} km/s",
            [
                velocity_itrf.itrf[0] / 1000.0,
                velocity_itrf.itrf[1] / 1000.0,
                velocity_itrf.itrf[2] / 1000.0
            ],
            speed_m_per_s / 1000.0
        );
        println!(
            "Drag power = {:.3} W (Elevation: {:.2} km, Speed: {:.2} km/s)",
            drag_power_watts,
            elevation_km,
            speed_m_per_s / 1000.0
        );

        for (station, angle_deg) in ground_stations.iter().zip(elevation_angles_degrees) {
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

        if elevation_km < 100.0 {
            println!(
                "Deorbit achieved at {:.2} days = {:.2} years since epoch = {}",
                hours_since_epoch / 24.0,
                hours_since_epoch / (24.0 * 365.0),
                time
            );
            return Ok(hours_since_epoch);
        }

        hours_since_epoch += 1.0;
    }

    Err(anyhow::anyhow!(
        "Failed to deorbit within expected time frame."
    ))
}

pub fn demo_deorbit() -> anyhow::Result<()> {
    let tle = get_sample_demo_tle()?;

    let max_hours: f64 = 24.0 * 365.0 * 100.0;
    let satellite = crate::initial_state_model::Satellite {
        name: "Demo Satellite".to_owned(),
        drag_coefficient: 2.5,
        drag_area_m2: (10.0e-2 * 10.0e-2), // 10 cm x 10 cm cross-sectional area
    };
    let ground_stations = [crate::initial_state_model::GroundStation::new(
        "Rothney Astro Observatory".to_owned(),
        50.8684,
        -114.2910,
        Some(1269.0),
        2.5,
        5.0,
    )];

    println!("Starting simulation for \"{}\":", satellite.name);
    println!("Using TLE:");
    println!("{:?}", tle);
    println!();

    propagate_to_deorbit(&satellite, &tle, max_hours, &ground_stations)?;
    Ok(())
}
