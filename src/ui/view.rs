use crate::ui::{
    actions::Message,
    fields::{
        GroundStationField, OrbitalField, SatelliteField, SimulationBoolField, SimulationField,
    },
};
use iced::{
    Element, Renderer,
    widget::{button, checkbox, column, horizontal_rule, row, scrollable, text, text_input},
};
use strum::IntoEnumIterator;

use crate::ui::actions::MyApp;

impl MyApp {
    pub fn view(&self) -> Element<'_, Message> {
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
