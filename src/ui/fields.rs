use std::collections::HashMap;

use strum_macros::EnumIter;

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

#[derive(Debug, Clone, Default)]
pub struct MyAppInputFields {
    pub ground_station_inputs: HashMap<GroundStationField, String>,
    pub satellite_inputs: HashMap<SatelliteField, String>,
    pub simulation_inputs: HashMap<SimulationField, String>,
    pub simulation_bools: HashMap<SimulationBoolField, bool>,

    pub orbital_params: HashMap<OrbitalField, String>,
}
