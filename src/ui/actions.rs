use std::collections::HashMap;

use crate::ui::fields::{
    GroundStationField, OrbitalField, SatelliteField, SimulationBoolField, SimulationField,
};
use iced::{
    Element, Event, Renderer, Subscription, Task, event,
    keyboard::{self, key},
    widget::{self, button, checkbox, column, horizontal_rule, row, scrollable, text, text_input},
};
use satkit::TLE;
use strum::IntoEnumIterator;

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
    pub ground_station_inputs: HashMap<GroundStationField, String>,
    pub satellite_inputs: HashMap<SatelliteField, String>,
    pub simulation_inputs: HashMap<SimulationField, String>,
    pub simulation_bools: HashMap<SimulationBoolField, bool>,

    /// Status message to display the result of the last run.
    run_status: String,
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
                self.on_button_pressed_run();
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
    fn on_button_pressed_run(&mut self) {
        let gs_dom = self.read_ground_station();
        let sat_dom = self.read_satellite();
        let sim_dom = self.read_simulation_settings();

        // Build status (or show first error encountered)
        match (gs_dom, sat_dom, sim_dom, &self.tle) {
            (Err(e), _, _, _) => self.run_status = e,
            (_, Err(e), _, _) => self.run_status = e,
            (_, _, Err(e), _) => self.run_status = e,
            (_, _, _, None) => {
                self.run_status = "Nothing to run - please enter a valid TLE.".to_string();
            }
            (Ok(gs_dom), Ok(sat_dom), Ok(sim_dom), Some(tle)) => {
                let ground_station_name: String = gs_dom.name.clone();
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
                            ground_station_name,
                            sat_dom.name,
                            sim_dom.step_interval_hours,
                            sim_dom.max_days,
                            sim_dom.drag_power_enable_space_weather
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
    iced::application("Squid Orbit Simulator", MyApp::update, MyApp::view)
        .subscription(MyApp::subscription)
        .run()
}
