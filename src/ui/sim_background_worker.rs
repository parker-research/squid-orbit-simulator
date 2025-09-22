// ui_egui.rs
use crate::{
    satellite_state::SimulationRun,
    ui::actions::{SIMULATION_MAX_UI_UPDATE_PERIOD_MS, StepOutcome, StepTx},
};
use std::sync::{Arc, Mutex};
use std::time::Instant;

pub fn spawn_stepper_loop(run: Arc<Mutex<SimulationRun>>, tx: StepTx) {
    std::thread::spawn(move || {
        // Loop until done, sending periodic StepOutcome updates
        loop {
            let real_time_start = Instant::now();

            // Scope the lock
            let mut guard = match run.lock() {
                Ok(g) => g,
                Err(_) => {
                    let _ = tx.send(Err("Poisoned mutex lock".to_string()));
                    break;
                }
            };
            let sim_run = &mut *guard;

            let max_hours = sim_run.initial.simulation_settings.max_days * 24.0;
            let step_interval_h = sim_run.initial.simulation_settings.step_interval_hours;

            // Inner loop: do work for up to SIMULATION_MAX_UI_UPDATE_PERIOD_MS, then send update
            let outcome = loop {
                if sim_run.hours_since_epoch() >= max_hours {
                    break Ok(StepOutcome {
                        done: true,
                        status_line: format!(
                            "Reached max time: {:.2} hours ({:.2} days).",
                            max_hours,
                            max_hours / 24.0
                        ),
                        latest_telemetry: sim_run.latest_telemetry.clone(),
                    });
                }

                match sim_run.step().map_err(|e| format!("{e}")) {
                    Ok(telemetry) => {
                        if telemetry.is_deorbited {
                            let deorbit_h =
                                (telemetry.hours_since_epoch - step_interval_h).max(0.0);
                            break Ok(StepOutcome {
                                done: true,
                                status_line: format!(
                                    "Satellite deorbited at {:.2} hours ({:.2} days).",
                                    deorbit_h,
                                    deorbit_h / 24.0
                                ),
                                latest_telemetry: sim_run.latest_telemetry.clone(),
                            });
                        }
                    }
                    Err(e) => break Err(e),
                }

                if real_time_start.elapsed().as_millis()
                    >= SIMULATION_MAX_UI_UPDATE_PERIOD_MS as u128
                {
                    let latest_telemetry = sim_run.latest_telemetry.as_ref().cloned();
                    let status = latest_telemetry.as_ref().map(|tt| {
                        format!("Sim running... t = {:.2} days", tt.hours_since_epoch / 24.0)
                    });
                    break Ok(StepOutcome {
                        done: latest_telemetry
                            .as_ref()
                            .map(|x| x.hours_since_epoch >= max_hours)
                            .unwrap_or(false),
                        status_line: status.unwrap_or_else(|| "Sim running...".to_string()),
                        latest_telemetry,
                    });
                }
            };

            drop(guard);

            // Send update
            let done_now = match &outcome {
                Ok(o) => o.done,
                Err(_) => true,
            };
            if tx.send(outcome).is_err() {
                // UI dropped the receiver
                break;
            }
            if done_now {
                break;
            }
        }
    });
}
