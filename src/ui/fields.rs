use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use strum_macros::EnumIter;

#[derive(Debug, Clone, Eq, Hash, PartialEq, EnumIter, Serialize, Deserialize)]
pub enum TleParameterField {
    Inclination,
    Raan,
    Eccentricity,
    ArgOfPerigee,
    MeanAnomaly,
    MeanMotion,
    Epoch,
}

impl TleParameterField {
    pub fn display_label(&self) -> &'static str {
        match self {
            TleParameterField::Inclination => "Inclination (deg)",
            TleParameterField::Raan => "RAAN (deg)",
            TleParameterField::Eccentricity => "Eccentricity",
            TleParameterField::ArgOfPerigee => "Argument of Perigee (deg)",
            TleParameterField::MeanAnomaly => "Mean Anomaly (deg)",
            TleParameterField::MeanMotion => "Mean Motion (rev/day)",
            TleParameterField::Epoch => "Epoch",
        }
    }
}

// -------------------------------------
// field enums for the forms
// -------------------------------------
#[derive(Debug, Clone, Eq, Hash, PartialEq, EnumIter, Serialize, Deserialize)]
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

#[derive(Debug, Clone, Eq, Hash, PartialEq, EnumIter, Serialize, Deserialize)]
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

#[derive(Debug, Clone, Eq, Hash, PartialEq, EnumIter, Serialize, Deserialize)]
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

#[derive(Debug, Clone, Eq, Hash, PartialEq, EnumIter, Serialize, Deserialize)]
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

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct MyAppInputFields {
    pub ground_station_inputs: HashMap<GroundStationField, String>,
    pub satellite_inputs: HashMap<SatelliteField, String>,
    pub simulation_inputs: HashMap<SimulationField, String>,
    pub simulation_bools: HashMap<SimulationBoolField, bool>,

    pub orbital_params: HashMap<TleParameterField, String>,
}
