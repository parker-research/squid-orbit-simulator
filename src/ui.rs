use std::collections::HashMap;

use iced::{
    Element, Renderer,
    widget::{button, column, row, text, text_input},
};
use satkit::TLE;
use strum::IntoEnumIterator;
use strum_macros::EnumIter;

#[derive(Debug, Clone)]
pub enum Message {
    TleLine0Changed(String),
    TleLine1Changed(String),
    TleLine2Changed(String),
    OrbitalParamChanged(OrbitalField, String),
    ButtonPressedRun,
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

#[derive(Debug, Default)]
pub struct MyApp {
    tle_line0: String,
    tle_line1: String,
    tle_line2: String,
    tle: Option<TLE>,
    orbital_params: HashMap<OrbitalField, String>,

    /// Status message to display the result of the last run.
    run_status: String,
}

impl MyApp {
    pub fn update(&mut self, message: Message) {
        match message {
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
            // Repeat for other fields...
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

    fn on_button_pressed_run(&mut self) {
        // TODO(Parker): Replace this with your actual simulation / propagation call.
        if let Some(tle) = &self.tle {
            self.run_status = format!(
                "Ran with TLE '{}': i={:.6}°, RAAN={:.6}°, e={:.7}, ω={:.6}°, M={:.6}°, n={:.8} rev/day",
                self.tle_line0.trim(),
                tle.inclination,
                tle.raan,
                tle.eccen,
                tle.arg_of_perigee,
                tle.mean_anomaly,
                tle.mean_motion
            );
        } else {
            self.run_status = "Nothing to run - please enter a valid TLE.".to_string();
        }
    }

    fn view(&self) -> Element<'_, Message> {
        let tle_inputs = vec![
            row![
                text("TLE Line 0 (Name)").width(100),
                text_input::<Message, iced::Theme, Renderer>("TLE Line 0", &self.tle_line0)
                    .on_input(Message::TleLine0Changed),
            ]
            .into(),
            row![
                text("TLE Line 1").width(100),
                text_input::<Message, iced::Theme, Renderer>("TLE Line 1", &self.tle_line1)
                    .on_input(Message::TleLine1Changed),
            ]
            .into(),
            row![
                text("TLE Line 2").width(100),
                text_input::<Message, iced::Theme, Renderer>("TLE Line 2", &self.tle_line2)
                    .on_input(Message::TleLine2Changed),
            ]
            .into(),
        ];

        let param_inputs = OrbitalField::iter().map(|field| {
            let label = field.display_label();
            let value = self.orbital_params.get(&field).cloned().unwrap_or_default();
            row![
                text(label).width(150),
                text_input::<Message, iced::Theme, Renderer>(label, &value)
                    .on_input(move |val| Message::OrbitalParamChanged(field.clone(), val))
            ]
            .into()
        });

        // Bottom bar with Run button + status.
        let run_bar = row![
            button::<Message, iced::Theme, Renderer>(text("Run"))
                // disable until TLE is valid/parsed
                .on_press_maybe(self.tle.as_ref().map(|_| Message::ButtonPressedRun)),
            text(&self.run_status),
        ]
        .spacing(12);

        column![
            column(tle_inputs).spacing(10),
            column(param_inputs.collect::<Vec<Element<'_, Message>>>()).spacing(10),
            run_bar
        ]
        .spacing(16)
        .into()
    }
}

pub fn main() -> iced::Result {
    iced::run("Squid Orbit Simulator", MyApp::update, MyApp::view)
}
