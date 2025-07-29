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

pub fn calculate_altitude_from_location_km(position_km: &[f64; 3]) -> f64 {
    let earth_radius_km = sgp4::WGS84.ae; // Earth's radius in km
    let radius_km = pythag_3(position_km);

    let altitude_km = radius_km - earth_radius_km;
    altitude_km
}

pub fn propagate_to_deorbit(constants: &sgp4::Constants, max_hours: f64) -> anyhow::Result<f64> {
    let mut hours_since_epoch: f64 = 0.0;

    while hours_since_epoch < max_hours {
        let prediction = constants.propagate(sgp4::MinutesSinceEpoch(hours_since_epoch * 60.0))?;

        // Print the current position and velocity.
        println!(
            "Time = {:.2} days = {:.2} years",
            hours_since_epoch / 24.0,
            hours_since_epoch / (24.0 * 365.0)
        );
        println!(
            "Position = {:?} km = {:.2} km",
            prediction.position,
            calculate_altitude_from_location_km(&prediction.position)
        );
        println!(
            "Velocity = {:?} km/s = {:.2} km/s",
            prediction.velocity,
            pythag_3(&prediction.velocity)
        );
        println!();

        let altitude_km = calculate_altitude_from_location_km(&prediction.position);

        if altitude_km < 100.0 {
            println!(
                "Deorbit achieved at {:.2} days = {:.2} years since epoch",
                hours_since_epoch / 24.0,
                hours_since_epoch / (24.0 * 365.0)
            );
            println!(
                "Final Position = {:?} km = {:.2} km",
                prediction.position, altitude_km
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

    let max_hours: f64 = 24.0 * 365.0 * 100.0; // 100 years
    propagate_to_deorbit(&constants, max_hours)?;
    Ok(())
}
