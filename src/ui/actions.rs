use std::collections::HashMap;

use crate::{
    initial_state_model::InitialSimulationState,
    satellite_state::SimulationRun,
    ui::fields::{
        GroundStationField, OrbitalField, SatelliteField, SimulationBoolField, SimulationField,
    },
};
use iced::{
    Event, Subscription, Task, event,
    keyboard::{self, key},
    widget::{self},
};
use satkit::TLE;
use std::sync::{Arc, Mutex};

// -------------------------------------
// App messages
// -------------------------------------
#[derive(Debug, Clone)]
pub enum Message {
    Event(iced::Event),

    // Existing
    TleLine0Changed(String),
    TleLine1Changed(String),
    TleLine2Changed(String),
    OrbitalParamChanged(OrbitalField, String),
    ButtonPressedRun,

    // ground station / satellite / sim settings inputs
    GroundStationChanged(GroundStationField, String),
    SatelliteChanged(SatelliteField, String),
    SimulationChanged(SimulationField, String),
    SimulationBoolToggled(SimulationBoolField, bool),

    // NEW: async stepping results
    RunTicked { result: Result<StepOutcome, String> },
}

#[derive(Debug, Clone)]
pub struct StepOutcome {
    pub done: bool,          // stop condition reached?
    pub status_line: String, // what to put into run_status
}

// -------------------------------------
// State
// -------------------------------------
#[derive(Debug, Default)]
pub struct MyApp {
    // Existing
    pub tle_line0: String,
    pub tle_line1: String,
    pub tle_line2: String,
    pub tle: Option<TLE>,
    pub orbital_params: HashMap<OrbitalField, String>,

    // raw input states (strings for numeric fields, so we can validate lazily)
    pub ground_station_inputs: HashMap<GroundStationField, String>,
    pub satellite_inputs: HashMap<SatelliteField, String>,
    pub simulation_inputs: HashMap<SimulationField, String>,
    pub simulation_bools: HashMap<SimulationBoolField, bool>,

    /// Status message to display the result of the last run.
    pub run_status: String,

    // Use an Arc<Mutex<...>> so we can share it with the async task.
    // None = no simulation running.
    pub simulation_run: Option<Arc<Mutex<SimulationRun>>>,

    pub is_running: bool,
}

impl MyApp {
    fn subscription(&self) -> Subscription<Message> {
        event::listen().map(Message::Event)
    }

    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::Event(event) => match event {
                Event::Keyboard(keyboard::Event::KeyPressed {
                    key: keyboard::Key::Named(key::Named::Tab),
                    modifiers,
                    ..
                }) => {
                    if modifiers.shift() {
                        return widget::focus_previous();
                    } else {
                        return widget::focus_next();
                    }
                }
                _ => {}
            },

            // Existing
            Message::TleLine0Changed(text) => {
                self.tle_line0 = text;
            }
            Message::TleLine1Changed(text) => {
                self.tle_line1 = text;
                self.try_parse_tle();
            }
            Message::TleLine2Changed(text) => {
                self.tle_line2 = text;
                self.try_parse_tle();
            }
            Message::OrbitalParamChanged(field, value) => {
                self.orbital_params.insert(field.clone(), value.clone());
                self.update_tle_from_fields();
            }
            Message::GroundStationChanged(field, value) => {
                self.ground_station_inputs.insert(field, value);
            }
            Message::SatelliteChanged(field, value) => {
                self.satellite_inputs.insert(field, value);
            }
            Message::SimulationChanged(field, value) => {
                self.simulation_inputs.insert(field, value);
            }
            Message::SimulationBoolToggled(field, value) => {
                self.simulation_bools.insert(field, value);
            }

            Message::ButtonPressedRun => {
                return self.on_button_pressed_run();
            }

            Message::RunTicked { result } => {
                match result {
                    Ok(outcome) => {
                        self.run_status = outcome.status_line;

                        if outcome.done {
                            // Stop permanently
                            self.is_running = false;
                            self.simulation_run = None;
                        } else if self.is_running {
                            // Keep going: schedule the next tick immediately
                            if let Some(run) = &self.simulation_run {
                                let run = run.clone();
                                return Task::perform(step_simulation_once(run), |result| {
                                    Message::RunTicked { result }
                                });
                            }
                        }
                    }
                    Err(err) => {
                        self.run_status = format!("Error during simulation step: {err}");
                        self.is_running = false;
                        self.simulation_run = None;
                    }
                }
            }
        }
        Task::none()
    }

    fn try_parse_tle(&mut self) {
        if let Ok(tle) = TLE::load_2line(&self.tle_line1, &self.tle_line2) {
            self.tle = Some(tle.clone());
            self.orbital_params
                .insert(OrbitalField::Inclination, format!("{}", tle.inclination));
            self.orbital_params
                .insert(OrbitalField::Raan, format!("{}", tle.raan));
            self.orbital_params
                .insert(OrbitalField::Eccentricity, format!("{}", tle.eccen));
            self.orbital_params.insert(
                OrbitalField::ArgOfPerigee,
                format!("{}", tle.arg_of_perigee),
            );
            self.orbital_params
                .insert(OrbitalField::MeanAnomaly, format!("{}", tle.mean_anomaly));
            self.orbital_params
                .insert(OrbitalField::MeanMotion, format!("{}", tle.mean_motion));
            self.orbital_params
                .insert(OrbitalField::Epoch, format!("{}", tle.epoch.as_iso8601()));
            self.run_status.clear();
        } else {
            self.tle = None;
            self.run_status = "Invalid TLE.".to_string();
        }
    }

    fn update_tle_from_fields(&mut self) {
        if let Some(tle) = &mut self.tle {
            if let Some(val) = self.orbital_params.get(&OrbitalField::Inclination) {
                if let Ok(v) = val.parse() {
                    tle.inclination = v;
                }
            }
            if let Some(val) = self.orbital_params.get(&OrbitalField::Raan) {
                if let Ok(v) = val.parse() {
                    tle.raan = v;
                }
            }
            if let Some(val) = self.orbital_params.get(&OrbitalField::Eccentricity) {
                if let Ok(v) = val.parse() {
                    tle.eccen = v;
                }
            }
            if let Some(val) = self.orbital_params.get(&OrbitalField::ArgOfPerigee) {
                if let Ok(v) = val.parse() {
                    tle.arg_of_perigee = v;
                }
            }
            if let Some(val) = self.orbital_params.get(&OrbitalField::MeanAnomaly) {
                if let Ok(v) = val.parse() {
                    tle.mean_anomaly = v;
                }
            }
            if let Some(val) = self.orbital_params.get(&OrbitalField::MeanMotion) {
                if let Ok(v) = val.parse() {
                    tle.mean_motion = v;
                }
            }
        }
    }
}

impl MyApp {
    fn on_button_pressed_run(&mut self) -> Task<Message> {
        // Initialize.
        let run = match self.init_simulation_run() {
            Ok(run) => run,
            Err(err) => {
                self.run_status = format!("Error initializing simulation: {err}");
                return Task::none();
            }
        };

        // Wrap for background stepping.
        let run = Arc::new(Mutex::new(run));
        self.simulation_run = Some(run.clone());
        self.is_running = true;
        self.run_status = "Starting simulation...".to_string();

        // Kick off the first tick..
        Task::perform(step_simulation_once(run), |result| Message::RunTicked {
            result,
        })
    }

    fn init_simulation_run(&mut self) -> Result<SimulationRun, String> {
        let ground_station_dom = self.read_ground_station()?;
        let satellite_dom = self.read_satellite()?;
        let simulation_settings_dom = self.read_simulation_settings()?;

        let tle = match &self.tle {
            Some(t) => t,
            None => return Err("No valid TLE available.".to_string()),
        };

        let ground_stations = [ground_station_dom];

        let initial_simulation_state = InitialSimulationState {
            tle: tle.clone(),
            ground_stations: ground_stations.to_vec(),
            satellite: satellite_dom,
            simulation_settings: simulation_settings_dom,
        };

        Ok(SimulationRun::new(initial_simulation_state))
    }
}

async fn step_simulation_once(run: Arc<Mutex<SimulationRun>>) -> Result<StepOutcome, String> {
    tokio::task::spawn_blocking(move || {
        // Lock the run for this step
        let mut guard = run.lock().map_err(|_| "Poisoned mutex lock".to_string())?;
        let sim_run = &mut *guard;

        let max_hours = sim_run.initial.simulation_settings.max_days * 24.0;
        let step_interval_h = sim_run.initial.simulation_settings.step_interval_hours;

        // Stop if we're already past max time
        if sim_run.hours_since_epoch() >= max_hours {
            return Ok(StepOutcome {
                done: true,
                status_line: format!(
                    "Reached max time: {:.2} hours ({:.2} days).",
                    max_hours,
                    max_hours / 24.0
                ),
            });
        }

        // Perform one step.
        let telemetry = sim_run.step().map_err(|e| format!("{e}"))?;

        // If deorbited, compute the previous tick time (like your comment).
        if telemetry.is_deorbited {
            let deorbit_h = (telemetry.hours_since_epoch - step_interval_h).max(0.0);
            return Ok(StepOutcome {
                done: true,
                status_line: format!(
                    "Satellite deorbited at {:.2} hours ({:.2} days).",
                    deorbit_h,
                    deorbit_h / 24.0
                ),
            });
        }

        // Otherwise, report progress and keep going.
        Ok(StepOutcome {
            done: telemetry.hours_since_epoch >= max_hours,
            status_line: format!(
                "Sim running… t = {:.2} h ({:.2} d)",
                telemetry.hours_since_epoch,
                telemetry.hours_since_epoch / 24.0
            ),
        })
    })
    .await
    .map_err(|_| "Join error stepping simulation".to_string())?
}

pub fn main() -> iced::Result {
    iced::application("Squid Orbit Simulator", MyApp::update, MyApp::view)
        .subscription(MyApp::subscription)
        .run()
}
