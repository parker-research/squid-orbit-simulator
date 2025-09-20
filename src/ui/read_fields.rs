use crate::ui::actions::MyApp;
use crate::ui::fields::{GroundStationField, SatelliteField, SimulationBoolField, SimulationField};

fn parse_required_f64(label: &str, s: &str) -> Result<f64, String> {
    let trimmed = s.trim();
    if trimmed.is_empty() {
        return Err(format!("'{}' is required", label));
    }
    trimmed
        .parse::<f64>()
        .map_err(|_| format!("Invalid number for '{}'", label))
}

fn parse_optional_f64(s: &str) -> Option<f64> {
    let t = s.trim();
    if t.is_empty() {
        None
    } else {
        t.parse::<f64>().ok()
    }
}

impl MyApp {
    pub fn read_ground_station(&self) -> Result<crate::initial_state_model::GroundStation, String> {
        let name = self
            .ground_station_inputs
            .get(&GroundStationField::Name)
            .cloned()
            .unwrap_or_default();

        let lat = parse_required_f64(
            GroundStationField::LatitudeDeg.label(),
            self.ground_station_inputs
                .get(&GroundStationField::LatitudeDeg)
                .map(String::as_str)
                .unwrap_or(""),
        )?;
        let lon = parse_required_f64(
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
            .and_then(parse_optional_f64);

        let alt = parse_required_f64(
            GroundStationField::AltitudeM.label(),
            self.ground_station_inputs
                .get(&GroundStationField::AltitudeM)
                .map(String::as_str)
                .unwrap_or(""),
        )?;
        let min_el = parse_required_f64(
            GroundStationField::MinElevationDeg.label(),
            self.ground_station_inputs
                .get(&GroundStationField::MinElevationDeg)
                .map(String::as_str)
                .unwrap_or(""),
        )?;

        Ok(crate::initial_state_model::GroundStation::new(
            name, lat, lon, elev_opt, alt, min_el,
        ))
    }

    pub fn read_satellite(&self) -> Result<crate::initial_state_model::Satellite, String> {
        let name = self
            .satellite_inputs
            .get(&SatelliteField::Name)
            .cloned()
            .unwrap_or_default();

        let cd = parse_required_f64(
            SatelliteField::DragCoefficient.label(),
            self.satellite_inputs
                .get(&SatelliteField::DragCoefficient)
                .map(String::as_str)
                .unwrap_or(""),
        )?;
        let area = parse_required_f64(
            SatelliteField::DragAreaM2.label(),
            self.satellite_inputs
                .get(&SatelliteField::DragAreaM2)
                .map(String::as_str)
                .unwrap_or(""),
        )?;

        Ok(crate::initial_state_model::Satellite {
            name,
            drag_coefficient: cd,
            drag_area_m2: area,
        })
    }

    pub fn read_simulation_settings(
        &self,
    ) -> Result<crate::initial_state_model::SimulationSettings, String> {
        let max_days = parse_required_f64(
            SimulationField::MaxDays.label(),
            self.simulation_inputs
                .get(&SimulationField::MaxDays)
                .map(String::as_str)
                .unwrap_or(""),
        )?;
        let step_hours = parse_required_f64(
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

        Ok(crate::initial_state_model::SimulationSettings {
            max_days,
            step_interval_hours: step_hours,
            drag_power_enable_space_weather: enable_sw,
        })
    }
}
