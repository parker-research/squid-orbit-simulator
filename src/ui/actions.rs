use crate::{
    initial_state_model::{InitialSimulationState, TleData},
    satellite_state::{SimulationRun, SimulationStateAtStep},
    ui::fields::{
        GroundStationField, MyAppInputFields, OrbitalField, SatelliteField, SimulationBoolField,
        SimulationField,
    },
};
use iced::{
    Event, Subscription, Task, event,
    keyboard::{self, key},
    widget::{self, text_editor},
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

    // I/O for input_fields <-> JSON
    ExportInputFieldsRequested,
    ImportInputFieldsFromJson(String),
    InputsJsonEdited(text_editor::Action),

    // Async stepping results.
    RunTicked { result: Result<StepOutcome, String> },
}

/// Only data passed back from the async task.
#[derive(Debug, Clone)]
pub struct StepOutcome {
    pub done: bool,          // stop condition reached?
    pub status_line: String, // what to put into run_status
    pub latest_telemetry: Option<SimulationStateAtStep>,
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
    pub tle_data: Option<TleData>,

    pub input_fields: MyAppInputFields,

    /// Status message to display the result of the last run.
    pub run_status: String,

    // Use an Arc<Mutex<...>> so we can share it with the async task.
    // None = no simulation running.
    pub simulation_run: Option<Arc<Mutex<SimulationRun>>>,

    pub latest_telemetry: Option<SimulationStateAtStep>,

    pub is_running: bool,

    pub inputs_json_editor: text_editor::Content,
}

const SIMULATION_MAX_UI_UPDATE_PERIOD_MS: usize = 600; // ms

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
                self.input_fields
                    .orbital_params
                    .insert(field.clone(), value.clone());
                self.update_tle_from_fields();
            }
            Message::GroundStationChanged(field, value) => {
                self.input_fields.ground_station_inputs.insert(field, value);
            }
            Message::SatelliteChanged(field, value) => {
                self.input_fields.satellite_inputs.insert(field, value);
            }
            Message::SimulationChanged(field, value) => {
                self.input_fields.simulation_inputs.insert(field, value);
            }
            Message::SimulationBoolToggled(field, value) => {
                self.input_fields.simulation_bools.insert(field, value);
            }

            // Import/Export of input fields as JSON.
            Message::ExportInputFieldsRequested => match self.export_inputs_json() {
                Ok(json) => {
                    self.inputs_json_editor = text_editor::Content::with_text(&json);
                    self.run_status = "Exported inputs to JSON buffer.".into();
                }
                Err(e) => {
                    self.run_status = format!("Failed to export inputs: {e}");
                }
            },
            Message::ImportInputFieldsFromJson(payload) => {
                match self.import_inputs_json(&payload) {
                    Ok(()) => {
                        self.run_status = "Imported inputs from JSON.".into();
                    }
                    Err(e) => {
                        self.run_status = format!("Failed to import inputs: {e}");
                    }
                }
            }
            Message::InputsJsonEdited(action) => {
                self.inputs_json_editor.perform(action); // Allow typing in the box.
            }

            Message::ButtonPressedRun => {
                return self.on_button_pressed_run();
            }

            Message::RunTicked { result } => {
                match result {
                    Ok(outcome) => {
                        self.run_status = outcome.status_line;
                        self.latest_telemetry = outcome.latest_telemetry;

                        if outcome.done {
                            // Stop permanently
                            self.is_running = false;
                            self.simulation_run = None;
                        } else if self.is_running {
                            // Keep going: schedule the next tick immediately
                            if let Some(run) = &self.simulation_run {
                                let run = run.clone();
                                return Task::perform(step_simulation_batch(run), |result| {
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
        if let Ok(satkit_tle) = TLE::load_2line(&self.tle_line1, &self.tle_line2) {
            self.tle_data = Some(TleData::from_satkit_tle(&satkit_tle));
            self.input_fields.orbital_params.insert(
                OrbitalField::Inclination,
                format!("{}", satkit_tle.inclination),
            );
            self.input_fields
                .orbital_params
                .insert(OrbitalField::Raan, format!("{}", satkit_tle.raan));
            self.input_fields
                .orbital_params
                .insert(OrbitalField::Eccentricity, format!("{}", satkit_tle.eccen));
            self.input_fields.orbital_params.insert(
                OrbitalField::ArgOfPerigee,
                format!("{}", satkit_tle.arg_of_perigee),
            );
            self.input_fields.orbital_params.insert(
                OrbitalField::MeanAnomaly,
                format!("{}", satkit_tle.mean_anomaly),
            );
            self.input_fields.orbital_params.insert(
                OrbitalField::MeanMotion,
                format!("{}", satkit_tle.mean_motion),
            );
            self.input_fields.orbital_params.insert(
                OrbitalField::Epoch,
                format!("{}", satkit_tle.epoch.as_iso8601()),
            );
            self.run_status.clear();
        } else {
            self.tle_data = None;
            self.run_status = "Invalid TLE.".to_string();
        }
    }

    fn update_tle_from_fields(&mut self) {
        if let Some(tle) = &mut self.tle_data {
            if let Some(val) = self
                .input_fields
                .orbital_params
                .get(&OrbitalField::Inclination)
            {
                if let Ok(v) = val.parse() {
                    tle.inclination = v;
                }
            }
            if let Some(val) = self.input_fields.orbital_params.get(&OrbitalField::Raan) {
                if let Ok(v) = val.parse() {
                    tle.raan = v;
                }
            }
            if let Some(val) = self
                .input_fields
                .orbital_params
                .get(&OrbitalField::Eccentricity)
            {
                if let Ok(v) = val.parse() {
                    tle.eccen = v;
                }
            }
            if let Some(val) = self
                .input_fields
                .orbital_params
                .get(&OrbitalField::ArgOfPerigee)
            {
                if let Ok(v) = val.parse() {
                    tle.arg_of_perigee = v;
                }
            }
            if let Some(val) = self
                .input_fields
                .orbital_params
                .get(&OrbitalField::MeanAnomaly)
            {
                if let Ok(v) = val.parse() {
                    tle.mean_anomaly = v;
                }
            }
            if let Some(val) = self
                .input_fields
                .orbital_params
                .get(&OrbitalField::MeanMotion)
            {
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
        Task::perform(step_simulation_batch(run), |result| Message::RunTicked {
            result,
        })
    }

    fn init_simulation_run(&mut self) -> Result<SimulationRun, String> {
        let ground_station_dom = self.read_ground_station()?;
        let satellite_dom = self.read_satellite()?;
        let simulation_settings_dom = self.read_simulation_settings()?;

        let tle_data = match &self.tle_data {
            Some(t) => t,
            None => return Err("No valid TLE available.".to_string()),
        };

        let ground_stations = [ground_station_dom];

        let initial_simulation_state = InitialSimulationState {
            tle: tle_data.clone(),
            ground_stations: ground_stations.to_vec(),
            satellite: satellite_dom,
            simulation_settings: simulation_settings_dom,
        };

        Ok(SimulationRun::new(initial_simulation_state))
    }
}

async fn step_simulation_batch(run: Arc<Mutex<SimulationRun>>) -> Result<StepOutcome, String> {
    tokio::task::spawn_blocking(move || {
        let real_time_start = std::time::Instant::now();

        let mut guard = run.lock().map_err(|_| "Poisoned mutex lock".to_string())?;
        let sim_run = &mut *guard;

        let max_hours = sim_run.initial.simulation_settings.max_days * 24.0;
        let step_interval_h = sim_run.initial.simulation_settings.step_interval_hours;

        // Run several simulation steps/ticks until real elapsed duration expires.
        loop {
            if sim_run.hours_since_epoch() >= max_hours {
                return Ok(StepOutcome {
                    done: true,
                    status_line: format!(
                        "Reached max time: {:.2} hours ({:.2} days).",
                        max_hours,
                        max_hours / 24.0
                    ),
                    latest_telemetry: sim_run.latest_telemetry.clone(),
                });
            }

            let telemetry = sim_run.step().map_err(|e| format!("{e}"))?;

            if telemetry.is_deorbited {
                let deorbit_h = (telemetry.hours_since_epoch - step_interval_h).max(0.0);
                return Ok(StepOutcome {
                    done: true,
                    status_line: format!(
                        "Satellite deorbited at {:.2} hours ({:.2} days).",
                        deorbit_h,
                        deorbit_h / 24.0
                    ),
                    latest_telemetry: sim_run.latest_telemetry.clone(),
                });
            }

            // Run minimum one iteration. Break if wallclock time exceeded between UI updates.
            if real_time_start.elapsed().as_millis() >= SIMULATION_MAX_UI_UPDATE_PERIOD_MS as u128 {
                break;
            }
        }

        let latest_telemetry = sim_run.latest_telemetry.as_ref();
        Ok(StepOutcome {
            done: latest_telemetry
                .map(|x| x.hours_since_epoch >= max_hours)
                .unwrap_or(false),
            status_line: match latest_telemetry {
                Some(tt) => format!("Sim running... t = {:.2} days", tt.hours_since_epoch / 24.0),
                None => "Sim running...".to_string(),
            },
            latest_telemetry: sim_run.latest_telemetry.clone(),
        })
    })
    .await
    .map_err(|_| "Join error stepping simulation".to_string())?
}

impl MyApp {
    /// Serialize the current `input_fields` to a pretty JSON string.
    pub fn export_inputs_json(&self) -> Result<String, String> {
        serde_json::to_string_pretty(&self.input_fields).map_err(|e| e.to_string())
    }

    /// Replace `input_fields` by deserializing from a JSON string.
    pub fn import_inputs_json(&mut self, json: &str) -> Result<(), String> {
        let parsed: MyAppInputFields = serde_json::from_str(json).map_err(|e| e.to_string())?;
        self.input_fields = parsed;

        // If your UI or simulation expects the TLE/derived fields to refresh,
        // you can optionally recalc/propagate here. For example:
        // self.update_tle_from_fields();   // only if your `input_fields` should drive TLE
        // self.run_status.clear();

        Ok(())
    }
}

pub fn main() -> iced::Result {
    iced::application("Squid Orbit Simulator", MyApp::update, MyApp::view)
        .subscription(MyApp::subscription)
        .run()
}
