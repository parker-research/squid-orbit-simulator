use std::collections::HashMap;

use satkit::TLE;

use crate::ui::{
    GroundStationField, Message, SatelliteField, SimulationBoolField, SimulationField,
};

async fn run_sim_streaming(
    tle: Option<TLE>,
    gs_inputs: HashMap<GroundStationField, String>,
    sat_inputs: HashMap<SatelliteField, String>,
    sim_inputs: HashMap<SimulationField, String>,
    sim_bools: HashMap<SimulationBoolField, bool>,
    out: &mut iced::futures::channel::mpsc::Sender<Message>,
) -> anyhow::Result<f64> {
    // Build domain types (GroundStation, Satellite, SimulationSettings) exactly as you do now...
    // let (sim_settings, satellite, tle, ground_stations) = ...;

    // Then run your loop but send progress every iteration (or every N iterations):
    let mut hours_since_epoch = 0.0;
    let mut tle_mut = tle.clone().unwrap();
    while hours_since_epoch < sim_settings.max_days * 24.0 {
        let time = tle.epoch + satkit::Duration::from_hours(hours_since_epoch);

        // --- your existing step computations here ---
        // position/velocity transform, elevation, angles, drag_power_watts, etc.

        // Emit a progress message (throttle if needed)
        let _ = out
            .send(Message::SimProgress(SimStep {
                hours_since_epoch,
                elevation_km,
                speed_km_s: speed_m_per_s / 1000.0,
                drag_power_w: drag_power_watts,
                lat_deg: position_itrf.latitude_deg(),
                lon_deg: position_itrf.longitude_deg(),
            }))
            .await;

        if elevation_km < 100.0 {
            return Ok(hours_since_epoch);
        }
        hours_since_epoch += sim_settings.step_interval_hours;
        // Optional: yield to the executor so UI stays snappy
        iced::futures::future::yield_now().await;
    }

    Err(anyhow::anyhow!(
        "Failed to deorbit within expected time frame."
    ))
}
