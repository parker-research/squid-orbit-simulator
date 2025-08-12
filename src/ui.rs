use std::collections::HashMap;

use iced::{
    Element, Renderer,
    widget::{button, checkbox, column, horizontal_rule, row, scrollable, text, text_input},
};
use satkit::TLE;
use strum::IntoEnumIterator;
use strum_macros::EnumIter;

// -------------------------------------
// Your domain structs
// -------------------------------------
pub struct GroundStation {
    pub name: String,
    pub latitude_deg: f64,
    pub longitude_deg: f64,
    pub elevation_m: Option<f64>,
    pub altitude_m: f64,
    pub min_elevation_deg: f64,
}

pub struct Satellite {
    pub name: String,
    /// Unitless drag coefficient of the satellite (C_d) for atmospheric drag calculations.
    pub drag_coefficient: f64,
    /// Average cross-sectional area of the satellite (A) for atmospheric drag calculations.
    pub drag_area_m2: f64,
}

pub struct SimulationSettings {
    pub max_days: f64,
    pub step_interval_hours: f64,
    pub drag_power_enable_space_weather: bool,
}

// -------------------------------------
// App messages
// -------------------------------------
#[derive(Debug, Clone)]
pub enum Message {
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
}

#[derive(Debug, Clone, Eq, Hash, PartialEq, EnumIter)]
pub enum OrbitalField {
    Inclination,
    Raan,
    Eccentricity,
    ArgOfPerigee,
    MeanAnomaly,
    MeanMotion,
    Epoch,
}

impl OrbitalField {
    pub fn display_label(&self) -> &'static str {
        match self {
            OrbitalField::Inclination => "Inclination (deg)",
            OrbitalField::Raan => "RAAN (deg)",
            OrbitalField::Eccentricity => "Eccentricity",
            OrbitalField::ArgOfPerigee => "Argument of Perigee (deg)",
            OrbitalField::MeanAnomaly => "Mean Anomaly (deg)",
            OrbitalField::MeanMotion => "Mean Motion (rev/day)",
            OrbitalField::Epoch => "Epoch",
        }
    }
}

// -------------------------------------
// field enums for the forms
// -------------------------------------
#[derive(Debug, Clone, Eq, Hash, PartialEq, EnumIter)]
pub enum GroundStationField {
    Name,
    LatitudeDeg,
    LongitudeDeg,
    ElevationM, // Option<f64> (empty = None)
    AltitudeM,
    MinElevationDeg,
}
impl GroundStationField {
    pub fn label(&self) -> &'static str {
        match self {
            GroundStationField::Name => "Name",
            GroundStationField::LatitudeDeg => "Latitude (deg)",
            GroundStationField::LongitudeDeg => "Longitude (deg)",
            GroundStationField::ElevationM => "Elevation MSL (m) (optional)",
            GroundStationField::AltitudeM => "Altitude AGL (m)",
            GroundStationField::MinElevationDeg => "Min Elevation (deg)",
        }
    }
}

#[derive(Debug, Clone, Eq, Hash, PartialEq, EnumIter)]
pub enum SatelliteField {
    Name,
    DragCoefficient,
    DragAreaM2,
}
impl SatelliteField {
    pub fn label(&self) -> &'static str {
        match self {
            SatelliteField::Name => "Name",
            SatelliteField::DragCoefficient => "Drag Coefficient (C_d)",
            SatelliteField::DragAreaM2 => "Drag Area (m²)",
        }
    }
}

#[derive(Debug, Clone, Eq, Hash, PartialEq, EnumIter)]
pub enum SimulationField {
    MaxDays,
    StepIntervalHours,
}
impl SimulationField {
    pub fn label(&self) -> &'static str {
        match self {
            SimulationField::MaxDays => "Max Days",
            SimulationField::StepIntervalHours => "Step Interval (hours)",
        }
    }
}

#[derive(Debug, Clone, Eq, Hash, PartialEq, EnumIter)]
pub enum SimulationBoolField {
    DragPowerEnableSpaceWeather,
}
impl SimulationBoolField {
    pub fn label(&self) -> &'static str {
        match self {
            SimulationBoolField::DragPowerEnableSpaceWeather => {
                "Enable Space Weather for Drag Power"
            }
        }
    }
}

// -------------------------------------
// State
// -------------------------------------
#[derive(Debug, Default)]
pub struct MyApp {
    // Existing
    tle_line0: String,
    tle_line1: String,
    tle_line2: String,
    tle: Option<TLE>,
    orbital_params: HashMap<OrbitalField, String>,

    // raw input states (strings for numeric fields, so we can validate lazily)
    ground_station_inputs: HashMap<GroundStationField, String>,
    satellite_inputs: HashMap<SatelliteField, String>,
    simulation_inputs: HashMap<SimulationField, String>,
    simulation_bools: HashMap<SimulationBoolField, bool>,

    /// Status message to display the result of the last run.
    run_status: String,
}

impl MyApp {
    pub fn update(&mut self, message: Message) {
        match message {
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
                self.on_button_pressed_run();
            }
        }
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

    // Helpers to parse numeric fields on-demand
    fn parse_f64(label: &str, s: &str) -> Result<f64, String> {
        s.trim()
            .parse::<f64>()
            .map_err(|_| format!("Invalid number for '{}'", label))
    }

    fn parse_f64_opt(s: &str) -> Option<f64> {
        let t = s.trim();
        if t.is_empty() {
            None
        } else {
            t.parse::<f64>().ok()
        }
    }

    fn on_button_pressed_run(&mut self) {
        // Parse GroundStation
        let gs = (|| -> Result<GroundStation, String> {
            let name = self
                .ground_station_inputs
                .get(&GroundStationField::Name)
                .cloned()
                .unwrap_or_default();

            let lat = Self::parse_f64(
                GroundStationField::LatitudeDeg.label(),
                self.ground_station_inputs
                    .get(&GroundStationField::LatitudeDeg)
                    .map(String::as_str)
                    .unwrap_or(""),
            )?;
            let lon = Self::parse_f64(
                GroundStationField::LongitudeDeg.label(),
                self.ground_station_inputs
                    .get(&GroundStationField::LongitudeDeg)
                    .map(String::as_str)
                    .unwrap_or(""),
            )?;
            let elev_opt = self
                .ground_station_inputs
                .get(&GroundStationField::ElevationM)
                .map(String::as_str)
                .and_then(Self::parse_f64_opt);

            let alt = Self::parse_f64(
                GroundStationField::AltitudeM.label(),
                self.ground_station_inputs
                    .get(&GroundStationField::AltitudeM)
                    .map(String::as_str)
                    .unwrap_or(""),
            )?;
            let min_el = Self::parse_f64(
                GroundStationField::MinElevationDeg.label(),
                self.ground_station_inputs
                    .get(&GroundStationField::MinElevationDeg)
                    .map(String::as_str)
                    .unwrap_or(""),
            )?;

            Ok(GroundStation {
                name,
                latitude_deg: lat,
                longitude_deg: lon,
                elevation_m: elev_opt,
                altitude_m: alt,
                min_elevation_deg: min_el,
            })
        })();

        // Parse Satellite
        let sat = (|| -> Result<Satellite, String> {
            let name = self
                .satellite_inputs
                .get(&SatelliteField::Name)
                .cloned()
                .unwrap_or_default();

            let cd = Self::parse_f64(
                SatelliteField::DragCoefficient.label(),
                self.satellite_inputs
                    .get(&SatelliteField::DragCoefficient)
                    .map(String::as_str)
                    .unwrap_or(""),
            )?;
            let area = Self::parse_f64(
                SatelliteField::DragAreaM2.label(),
                self.satellite_inputs
                    .get(&SatelliteField::DragAreaM2)
                    .map(String::as_str)
                    .unwrap_or(""),
            )?;

            Ok(Satellite {
                name,
                drag_coefficient: cd,
                drag_area_m2: area,
            })
        })();

        // Parse SimulationSettings
        let sim = (|| -> Result<SimulationSettings, String> {
            let max_days = Self::parse_f64(
                SimulationField::MaxDays.label(),
                self.simulation_inputs
                    .get(&SimulationField::MaxDays)
                    .map(String::as_str)
                    .unwrap_or(""),
            )?;
            let step_hours = Self::parse_f64(
                SimulationField::StepIntervalHours.label(),
                self.simulation_inputs
                    .get(&SimulationField::StepIntervalHours)
                    .map(String::as_str)
                    .unwrap_or(""),
            )?;
            let enable_sw = *self
                .simulation_bools
                .get(&SimulationBoolField::DragPowerEnableSpaceWeather)
                .unwrap_or(&false);

            if max_days <= 0.0 {
                return Err("Max Days must be > 0".into());
            }
            if step_hours <= 0.0 {
                return Err("Step Interval (hours) must be > 0".into());
            }

            Ok(SimulationSettings {
                max_days,
                step_interval_hours: step_hours,
                drag_power_enable_space_weather: enable_sw,
            })
        })();

        // Build status (or show first error encountered)
        match (gs, sat, sim, &self.tle) {
            (Err(e), _, _, _) => self.run_status = e,
            (_, Err(e), _, _) => self.run_status = e,
            (_, _, Err(e), _) => self.run_status = e,
            (_, _, _, None) => {
                self.run_status = "Nothing to run - please enter a valid TLE.".to_string();
            }
            (Ok(gs), Ok(sat), Ok(sim), Some(tle)) => {
                // Convert UI structs to your domain types and run the real simulator.
                let gs_dom = crate::initial_state_model::GroundStation::new(
                    gs.name.clone(),
                    gs.latitude_deg,
                    gs.longitude_deg,
                    gs.elevation_m,
                    gs.altitude_m,
                    gs.min_elevation_deg,
                );

                let sat_dom = crate::initial_state_model::Satellite {
                    name: sat.name.clone(),
                    drag_coefficient: sat.drag_coefficient,
                    drag_area_m2: sat.drag_area_m2,
                };

                let sim_dom = crate::initial_state_model::SimulationSettings {
                    max_days: sim.max_days,
                    step_interval_hours: sim.step_interval_hours,
                    drag_power_enable_space_weather: sim.drag_power_enable_space_weather,
                };

                let ground_stations = [gs_dom];

                match crate::satellite_state::propagate_to_deorbit(
                    &sim_dom,
                    &sat_dom,
                    tle,
                    &ground_stations,
                ) {
                    Ok(days_to_deorbit) => {
                        self.run_status = format!(
                            "Simulation complete: deorbit in {:.3} days.\n\
                         GS: {} | SAT: {} | step={:.4} h | max_days={:.1} | space_weather={}",
                            days_to_deorbit,
                            gs.name,
                            sat.name,
                            sim.step_interval_hours,
                            sim.max_days,
                            sim.drag_power_enable_space_weather
                        );
                    }
                    Err(err) => {
                        self.run_status = format!("Simulation failed: {err}");
                    }
                }
            }
        }
    }

    fn view(&self) -> Element<'_, Message> {
        // ------------------------------
        // TLE inputs (existing)
        // ------------------------------
        let tle_inputs = vec![
            row![
                text("TLE Line 0 (Name)").width(180),
                text_input::<Message, iced::Theme, Renderer>("TLE Line 0", &self.tle_line0)
                    .on_input(Message::TleLine0Changed),
            ]
            .into(),
            row![
                text("TLE Line 1").width(180),
                text_input::<Message, iced::Theme, Renderer>("TLE Line 1", &self.tle_line1)
                    .on_input(Message::TleLine1Changed),
            ]
            .into(),
            row![
                text("TLE Line 2").width(180),
                text_input::<Message, iced::Theme, Renderer>("TLE Line 2", &self.tle_line2)
                    .on_input(Message::TleLine2Changed),
            ]
            .into(),
        ];

        // ------------------------------
        // Orbital param inputs
        // ------------------------------
        let param_inputs = OrbitalField::iter().map(|field| {
            let label = field.display_label();
            let value = self.orbital_params.get(&field).cloned().unwrap_or_default();
            row![
                text(label).width(180),
                text_input::<Message, iced::Theme, Renderer>(label, &value)
                    .on_input(move |val| Message::OrbitalParamChanged(field.clone(), val))
            ]
            .into()
        });

        // ------------------------------
        // Ground Station inputs
        // ------------------------------
        let gs_inputs = GroundStationField::iter().map(|f| {
            let label = f.label();
            let value = self
                .ground_station_inputs
                .get(&f)
                .cloned()
                .unwrap_or_default();
            row![
                text(label).width(180),
                text_input::<Message, iced::Theme, Renderer>(label, &value)
                    .on_input(move |val| Message::GroundStationChanged(f.clone(), val))
            ]
            .into()
        });

        // ------------------------------
        // Satellite inputs
        // ------------------------------
        let sat_inputs = SatelliteField::iter().map(|f| {
            let label = f.label();
            let value = self.satellite_inputs.get(&f).cloned().unwrap_or_default();
            row![
                text(label).width(180),
                text_input::<Message, iced::Theme, Renderer>(label, &value)
                    .on_input(move |val| Message::SatelliteChanged(f.clone(), val))
            ]
            .into()
        });

        // ------------------------------
        // Simulation Settings inputs
        // ------------------------------
        let sim_number_inputs = SimulationField::iter().map(|f| {
            let label = f.label();
            let value = self.simulation_inputs.get(&f).cloned().unwrap_or_default();
            row![
                text(label).width(180),
                text_input::<Message, iced::Theme, Renderer>(label, &value)
                    .on_input(move |val| Message::SimulationChanged(f.clone(), val))
            ]
            .into()
        });

        let sim_bool_row = SimulationBoolField::iter().map(|f| {
            let label = f.label();
            let value = self.simulation_bools.get(&f).cloned().unwrap_or_default();
            row![
                text(label).width(180),
                checkbox::<Message, iced::Theme, Renderer>(label, value)
                    .on_toggle(move |val| Message::SimulationBoolToggled(f.clone(), val))
            ]
            .into()
        });

        // Bottom bar with Run button + status.
        let run_bar = row![
            button::<Message, iced::Theme, Renderer>(text("Run"))
                .on_press(Message::ButtonPressedRun),
            text(&self.run_status),
        ]
        .spacing(12);

        // Layout.
        scrollable(
            column![
                // TLE + Orbital
                text("TLE").size(22),
                column(tle_inputs).spacing(8),
                horizontal_rule(1),
                text("Orbital Parameters").size(22),
                column(param_inputs.collect::<Vec<Element<'_, Message>>>()).spacing(8),
                horizontal_rule(1),
                // Ground Station
                text("Ground Station").size(22),
                column(gs_inputs.collect::<Vec<Element<'_, Message>>>()).spacing(8),
                horizontal_rule(1),
                // Satellite
                text("Satellite").size(22),
                column(sat_inputs.collect::<Vec<Element<'_, Message>>>()).spacing(8),
                horizontal_rule(1),
                // Simulation Settings
                text("Simulation Settings").size(22),
                column(sim_number_inputs.collect::<Vec<Element<'_, Message>>>()).spacing(8),
                column(sim_bool_row.collect::<Vec<Element<'_, Message>>>()).spacing(8),
                horizontal_rule(1),
                // Run
                run_bar
            ]
            .spacing(16)
            .padding(16),
        )
        .into()
    }
}

pub fn main() -> iced::Result {
    iced::run("Squid Orbit Simulator", MyApp::update, MyApp::view)
}
