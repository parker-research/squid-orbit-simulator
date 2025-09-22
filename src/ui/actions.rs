// ui_egui.rs
use crate::{
    initial_state_model::{InitialSimulationState, TleData},
    satellite_state::{SimulationRun, SimulationStateAtStep},
    ui::fields::{
        GroundStationField, MyAppInputFields, OrbitalField, SatelliteField, SimulationBoolField,
        SimulationField,
    },
};
use eframe::egui::{self, FontId, RichText};
use satkit::TLE;
use std::sync::{Arc, Mutex, mpsc};
use std::time::{Duration, Instant};

// -------------------------------------
// Background worker messages
// -------------------------------------
#[derive(Debug, Clone)]
pub struct StepOutcome {
    pub done: bool,          // stop condition reached?
    pub status_line: String, // what to put into run_status
    pub latest_telemetry: Option<SimulationStateAtStep>,
}

type StepTx = mpsc::Sender<Result<StepOutcome, String>>;
type StepRx = mpsc::Receiver<Result<StepOutcome, String>>;

// -------------------------------------
// App State (egui)
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

    // Simulation
    pub simulation_run: Option<Arc<Mutex<SimulationRun>>>,
    pub latest_telemetry: Option<SimulationStateAtStep>,
    pub is_running: bool,

    // JSON I/O buffer
    pub inputs_json_buffer: String,

    // Worker channel
    worker_rx: Option<StepRx>,
}

const SIMULATION_MAX_UI_UPDATE_PERIOD_MS: usize = 600; // ms

impl MyApp {
    pub fn new() -> Self {
        Self::default()
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

    fn on_export_inputs_json(&mut self) {
        match self.export_inputs_json() {
            Ok(json) => {
                self.inputs_json_buffer = json;
                self.run_status = "Exported inputs to JSON buffer.".into();
            }
            Err(e) => self.run_status = format!("Failed to export inputs: {e}"),
        }
    }

    fn on_import_inputs_json(&mut self) {
        let json = self.inputs_json_buffer.clone();
        match self.import_inputs_json(&json) {
            Ok(()) => self.run_status = "Imported inputs from JSON.".into(),
            Err(e) => self.run_status = format!("Failed to import inputs: {e}"),
        }
    }

    fn on_button_pressed_run(&mut self, ctx: &egui::Context) {
        // Initialize.
        let run = match self.init_simulation_run() {
            Ok(run) => run,
            Err(err) => {
                self.run_status = format!("Error initializing simulation: {err}");
                return;
            }
        };

        // Wrap for background stepping.
        let run = Arc::new(Mutex::new(run));
        self.simulation_run = Some(run.clone());
        self.is_running = true;
        self.run_status = "Starting simulation...".to_string();

        // Create channel and spawn worker that streams StepOutcome results.
        let (tx, rx): (StepTx, StepRx) = mpsc::channel();
        self.worker_rx = Some(rx);

        spawn_stepper_loop(run, tx);

        // Make sure UI keeps polling while running.
        ctx.request_repaint();
    }

    fn poll_worker(&mut self, ctx: &egui::Context) {
        let mut should_make_worker_rx_null: bool = false;

        if let Some(rx) = &self.worker_rx {
            for msg in rx.try_iter() {
                match msg {
                    Ok(outcome) => {
                        self.run_status = outcome.status_line;
                        self.latest_telemetry = outcome.latest_telemetry;

                        if outcome.done {
                            self.is_running = false;
                            self.simulation_run = None;
                            should_make_worker_rx_null = true;
                        }
                    }
                    Err(err) => {
                        self.run_status = format!("Error during simulation step: {err}");
                        self.is_running = false;
                        self.simulation_run = None;
                        should_make_worker_rx_null = true;
                    }
                }
            }

            // While running, ask egui to repaint periodically.
            if self.is_running {
                ctx.request_repaint_after(Duration::from_millis(
                    SIMULATION_MAX_UI_UPDATE_PERIOD_MS as u64,
                ));
            }
        }

        if should_make_worker_rx_null {
            self.worker_rx = None;
        }
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

    /// Serialize the current `input_fields` to a pretty JSON string.
    pub fn export_inputs_json(&self) -> Result<String, String> {
        serde_json::to_string_pretty(&self.input_fields).map_err(|e| e.to_string())
    }

    /// Replace `input_fields` by deserializing from a JSON string.
    pub fn import_inputs_json(&mut self, json: &str) -> Result<(), String> {
        let parsed: MyAppInputFields = serde_json::from_str(json).map_err(|e| e.to_string())?;
        self.input_fields = parsed;
        Ok(())
    }
}

// -------------------------------------
// Background worker
// -------------------------------------
fn spawn_stepper_loop(run: Arc<Mutex<SimulationRun>>, tx: StepTx) {
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

// -------------------------------------
// egui UI
// -------------------------------------
use strum::IntoEnumIterator;

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Poll worker (if running)
        self.poll_worker(ctx);

        egui::TopBottomPanel::top("top_bar").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.heading("Squid Orbit Simulator");
                if self.is_running {
                    ui.label(RichText::new("Running…").strong());
                }
            });
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            egui::ScrollArea::vertical()
                .auto_shrink([false, false])
                .show(ui, |ui| {
                    ui.add_space(8.0);

                    // ------------------------------
                    // TLE inputs
                    // ------------------------------
                    ui.heading("TLE");
                    ui.horizontal(|ui| {
                        ui.label("TLE Line 0 (Name)");
                        ui.text_edit_singleline(&mut self.tle_line0);
                    });
                    let mut need_parse = false;
                    ui.horizontal(|ui| {
                        ui.label("TLE Line 1");
                        let before = self.tle_line1.clone();
                        if ui.text_edit_singleline(&mut self.tle_line1).changed()
                            && self.tle_line1 != before
                        {
                            need_parse = true;
                        }
                    });
                    ui.horizontal(|ui| {
                        ui.label("TLE Line 2");
                        let before = self.tle_line2.clone();
                        if ui.text_edit_singleline(&mut self.tle_line2).changed()
                            && self.tle_line2 != before
                        {
                            need_parse = true;
                        }
                    });
                    if need_parse {
                        self.try_parse_tle();
                    }

                    ui.add_space(8.0);
                    ui.separator();

                    // ------------------------------
                    // Orbital Parameters
                    // ------------------------------
                    ui.heading("Orbital Parameters");
                    for field in OrbitalField::iter() {
                        let label = field.display_label();
                        let val = self
                            .input_fields
                            .orbital_params
                            .get(&field)
                            .cloned()
                            .unwrap_or_default();
                        let mut val_mut = val.clone();
                        ui.horizontal(|ui| {
                            ui.label(label); // .min_size.(egui::vec2(180.0, 0.0));
                            if ui.text_edit_singleline(&mut val_mut).changed() {
                                self.input_fields
                                    .orbital_params
                                    .insert(field.clone(), val_mut.clone());
                                self.update_tle_from_fields();
                            }
                        });
                    }

                    ui.add_space(8.0);
                    ui.separator();

                    // ------------------------------
                    // Ground Station
                    // ------------------------------
                    ui.heading("Ground Station");
                    for f in GroundStationField::iter() {
                        let label = f.label();
                        let val = self
                            .input_fields
                            .ground_station_inputs
                            .get(&f)
                            .cloned()
                            .unwrap_or_default();
                        let mut val_mut = val.clone();
                        ui.horizontal(|ui| {
                            ui.label(label); //.min_size(egui::vec2(180.0, 0.0));
                            if ui.text_edit_singleline(&mut val_mut).changed() {
                                self.input_fields
                                    .ground_station_inputs
                                    .insert(f.clone(), val_mut.clone());
                            }
                        });
                    }

                    ui.add_space(8.0);
                    ui.separator();

                    // ------------------------------
                    // Satellite
                    // ------------------------------
                    ui.heading("Satellite");
                    for f in SatelliteField::iter() {
                        let label = f.label();
                        let val = self
                            .input_fields
                            .satellite_inputs
                            .get(&f)
                            .cloned()
                            .unwrap_or_default();
                        let mut val_mut = val.clone();
                        ui.horizontal(|ui| {
                            ui.label(label); // .min_size(egui::vec2(180.0, 0.0));
                            if ui.text_edit_singleline(&mut val_mut).changed() {
                                self.input_fields
                                    .satellite_inputs
                                    .insert(f.clone(), val_mut.clone());
                            }
                        });
                    }

                    ui.add_space(8.0);
                    ui.separator();

                    // ------------------------------
                    // Simulation Settings
                    // ------------------------------
                    ui.heading("Simulation Settings");
                    for f in SimulationField::iter() {
                        let label = f.label();
                        let val = self
                            .input_fields
                            .simulation_inputs
                            .get(&f)
                            .cloned()
                            .unwrap_or_default();
                        let mut val_mut = val.clone();
                        ui.horizontal(|ui| {
                            ui.label(label); // .min_size(egui::vec2(180.0, 0.0));
                            if ui.text_edit_singleline(&mut val_mut).changed() {
                                self.input_fields
                                    .simulation_inputs
                                    .insert(f.clone(), val_mut.clone());
                            }
                        });
                    }
                    for f in SimulationBoolField::iter() {
                        let label = f.label();
                        let value = *self.input_fields.simulation_bools.get(&f).unwrap_or(&false);
                        let mut value_mut = value;
                        ui.horizontal(|ui| {
                            ui.label(label); // .min_size(egui::vec2(180.0, 0.0));
                            if ui.checkbox(&mut value_mut, "").changed() {
                                self.input_fields
                                    .simulation_bools
                                    .insert(f.clone(), value_mut);
                            }
                        });
                    }

                    ui.add_space(8.0);
                    ui.separator();

                    // ------------------------------
                    // Inputs JSON I/O
                    // ------------------------------
                    ui.heading("Import/Export Simulation Configuration as JSON");
                    ui.horizontal(|ui| {
                        if ui.button("Export Inputs").clicked() {
                            self.on_export_inputs_json();
                        }
                        if ui.button("Import Inputs").clicked() {
                            self.on_import_inputs_json();
                        }
                    });
                    egui::ScrollArea::vertical()
                        .max_height(200.0)
                        .show(ui, |ui| {
                            ui.add(
                                egui::TextEdit::multiline(&mut self.inputs_json_buffer)
                                    .font(FontId::monospace(14.0))
                                    .hint_text("Paste or edit inputs JSON here…"),
                            );
                        });

                    ui.add_space(8.0);
                    ui.separator();

                    // ------------------------------
                    // Run bar
                    // ------------------------------
                    ui.horizontal(|ui| {
                        if ui
                            .add_enabled(!self.is_running, egui::Button::new("Run"))
                            .clicked()
                        {
                            self.on_button_pressed_run(ctx);
                        }
                        ui.label(&self.run_status);
                    });

                    ui.add_space(8.0);
                    ui.separator();

                    // ------------------------------
                    // Telemetry
                    // ------------------------------
                    ui.heading("Latest Telemetry");
                    match &self.latest_telemetry {
                        Some(t) => {
                            let angles_preview = if t.elevation_angles_degrees.is_empty() {
                                "[]".to_string()
                            } else {
                                let shown = t
                                    .elevation_angles_degrees
                                    .iter()
                                    .take(5)
                                    .map(|v| format!("{:.2}", v))
                                    .collect::<Vec<_>>()
                                    .join(", ");
                                if t.elevation_angles_degrees.len() > 5 {
                                    format!(
                                        "[{}, …] ({} total)",
                                        shown,
                                        t.elevation_angles_degrees.len()
                                    )
                                } else {
                                    format!("[{}]", shown)
                                }
                            };

                            grid_kv(ui, "Data Point Timestamp", &t.time.as_iso8601());
                            grid_kv(
                                ui,
                                "Time since epoch",
                                &format!(
                                    "{:.3} hours = {:.3} days",
                                    t.hours_since_epoch,
                                    t.hours_since_epoch / 24.0
                                ),
                            );
                            grid_kv(ui, "ITRF position", &format!("{:?}", t.position_itrf));
                            grid_kv(ui, "ITRF velocity", &format!("{:?}", t.velocity_itrf));
                            grid_kv(ui, "Speed (m/s)", &format!("{:.3}", t.speed_m_per_s));
                            grid_kv(ui, "Elevation (km)", &format!("{:.3}", t.elevation_km));
                            grid_kv(ui, "Elevation angles (deg)", &angles_preview);
                            grid_kv(ui, "Drag power (W)", &format!("{:.3}", t.drag_power_watts));
                            grid_kv(
                                ui,
                                "Irradiance approx (W/m²)",
                                &format!("{:.1}", t.irradiance_approx_w_per_m2),
                            );
                            grid_kv(
                                ui,
                                "Irradiance (W/m²)",
                                &format!("{:.1}", t.irradiance_w_per_m2),
                            );
                            grid_kv(ui, "Local time (h)", &format!("{:.3}", t.local_time_hours));
                            grid_kv(ui, "Deorbited?", if t.is_deorbited { "yes" } else { "no" });
                        }
                        None => {
                            ui.label("No telemetry yet. Press Run to start.");
                        }
                    }
                });
        });
    }
}

fn grid_kv(ui: &mut egui::Ui, key: &str, val: &str) {
    ui.horizontal(|ui| {
        ui.label(key); // .min_size(egui::vec2(180.0, 0.0));
        ui.label(val);
    });
}

// -------------------------------------
// eframe entry point
// -------------------------------------
pub fn main() -> eframe::Result<()> {
    let native_options = eframe::NativeOptions::default();
    eframe::run_native(
        "Squid Orbit Simulator",
        native_options,
        Box::new(|_cc| Ok(Box::new(MyApp::new()))),
    )
}
