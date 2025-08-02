#[allow(dead_code)]
pub fn get_sample_demo_tle_constants() -> anyhow::Result<sgp4::Constants> {
    let elements = sgp4::Elements::from_tle(
        Some("Satellite 1".to_owned()),
        "1 25544U 98067A   20194.88612269 -.00002218  00000-0 -31515-4 0  9992".as_bytes(),
        "2 25544  51.6461 221.2784 0001413  89.1723 280.4612 15.49507896236008".as_bytes(),
    )?;
    let constants: sgp4::Constants = sgp4::Constants::from_elements(&elements)?;
    Ok(constants)
}

#[allow(dead_code)]
pub fn get_sample_demo_tle_constants_canx_5() -> anyhow::Result<sgp4::Constants> {
    let elements = sgp4::Elements::from_tle(
        Some("CANX-5".to_owned()),
        "1 40056U 14034D   25209.55901054  .00000935  00000+0  13011-3 0  9999".as_bytes(),
        "2 40056  98.3577  67.3174 0012454 336.2296  23.8338 14.80323878587440".as_bytes(),
    )?;
    let constants: sgp4::Constants = sgp4::Constants::from_elements(&elements)?;
    Ok(constants)
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
) -> f64 {
    let (rho, _temp) =
        satkit::nrlmsise::nrlmsise(elevation_km, latitude_deg, longitude_deg, Some(*t), true);

    let power_watts =
        0.5 * satellite.drag_coefficient * rho * satellite.drag_area_m2 * speed_m_per_s.powi(3);
    power_watts
}

pub fn propagate_to_deorbit(
    satellite: &crate::initial_state_model::Satellite,
    orbit_constants: &sgp4::Constants,
    max_hours: f64,
    ground_stations: &[crate::initial_state_model::GroundStation],
) -> anyhow::Result<f64> {
    let mut hours_since_epoch: f64 = 0.0;

    while hours_since_epoch < max_hours {
        let prediction =
            orbit_constants.propagate(sgp4::MinutesSinceEpoch(hours_since_epoch * 60.0))?;

        let elevation_km = calculate_elevation_from_location_km(&prediction.position);

        let elevation_angles_degrees = ground_stations
            .iter()
            .map(|station| calculate_elevation_angle_degrees(&prediction.position, station))
            .collect::<Vec<_>>();

        // Print the current position and velocity.
        println!(
            "Time = {:.2} days = {:.2} years",
            hours_since_epoch / 24.0,
            hours_since_epoch / (24.0 * 365.0)
        );
        println!(
            "Position = {:?} km = {:.2} km",
            prediction.position,
            calculate_elevation_from_location_km(&prediction.position)
        );
        println!(
            "Velocity = {:?} km/s = {:.2} km/s",
            prediction.velocity,
            pythag_3(&prediction.velocity)
        );

        // Assess the link to each ground station.
        for (station, angle_deg) in ground_stations.iter().zip(elevation_angles_degrees) {
            println!(
                "Ground station \"{}\" -> {} Elevation: {:.2} degrees (Distance: {:.2} km)",
                station.name,
                (if angle_deg > station.min_elevation_deg {
                    "✅"
                } else {
                    "❌"
                }),
                angle_deg,
                pythag_3(&[
                    prediction.position[0] - station.ecef_xyz_m()[0] / 1000.0,
                    prediction.position[1] - station.ecef_xyz_m()[1] / 1000.0,
                    prediction.position[2] - station.ecef_xyz_m()[2] / 1000.0,
                ])
            );
        }
        println!();

        // Evaluate deorbit condition.
        if elevation_km < 100.0 {
            println!(
                "Deorbit achieved at {:.2} days = {:.2} years since epoch",
                hours_since_epoch / 24.0,
                hours_since_epoch / (24.0 * 365.0)
            );
            println!(
                "Final Position = {:?} km = {:.2} km",
                prediction.position, elevation_km
            );
            println!("Final Velocity = {:?} km/s", prediction.velocity);
            return Ok(hours_since_epoch);
        }
        hours_since_epoch += 1.0;
    }

    // If we exit the loop without deorbiting, return an error.
    Err(anyhow::anyhow!(
        "Failed to deorbit within expected time frame."
    ))
}

pub fn demo_deorbit() -> anyhow::Result<()> {
    let constants = get_sample_demo_tle_constants()?;

    println!("Using TLE constants:");
    println!("{:?}", constants);
    println!();
    println!();

    let max_hours: f64 = 24.0 * 365.0 * 100.0; // 100 years
    let satellite = crate::initial_state_model::Satellite {
        name: "Demo Satellite".to_owned(),
        drag_coefficient: 2.5,
        drag_area_m2: 10.0,
    };
    let ground_stations = [crate::initial_state_model::GroundStation::new(
        "Rothney Astro Observatory".to_owned(),
        50.8684,
        -114.2910,
        Some(1269.0),
        2.5,
        5.0, // min_elevation_deg
    )];
    propagate_to_deorbit(&satellite, &constants, max_hours, &ground_stations)?;
    Ok(())
}
