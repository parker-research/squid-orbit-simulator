use std::collections::HashMap;

use iced::{
    Element, Renderer,
    widget::{column, row, text, text_input},
};
use satkit::TLE;
use strum::IntoEnumIterator;
use strum_macros::EnumIter;

#[derive(Debug, Clone)]
pub enum Message {
    TleLine1Changed(String),
    TleLine2Changed(String),
    OrbitalParamChanged(OrbitalField, String),
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

#[derive(Debug, Default)]
pub struct MyApp {
    tle_line1: String,
    tle_line2: String,
    tle: Option<TLE>,
    orbital_params: HashMap<OrbitalField, String>,
}

impl MyApp {
    pub fn update(&mut self, message: Message) {
        match message {
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
        }
    }

    fn view(&self) -> Element<'_, Message> {
        let tle_inputs = column![
            text("TLE Line 1"),
            text_input("TLE Line 1", &self.tle_line1).on_input(Message::TleLine1Changed),
            text("TLE Line 2"),
            text_input("TLE Line 2", &self.tle_line2).on_input(Message::TleLine2Changed),
        ];

        let param_inputs = OrbitalField::iter().map(|field| {
            let label = format!("{:?}", field);
            let value = self.orbital_params.get(&field).cloned().unwrap_or_default();
            row![
                text_input::<Message, iced::Theme, Renderer>(&label, &value)
                    .on_input(move |val| Message::OrbitalParamChanged(field.clone(), val))
            ]
            .into()
        });

        column![
            tle_inputs,
            column(param_inputs.collect::<Vec<Element<'_, Message>>>()).spacing(10)
        ]
        .into()
    }
}

pub fn main() -> iced::Result {
    iced::run("Squid Orbit Simulator", MyApp::update, MyApp::view)
}
