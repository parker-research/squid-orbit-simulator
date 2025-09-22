use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use strum_macros::EnumIter;

use crate::initial_state_model::TleData;

#[derive(Debug, Clone, Eq, Hash, PartialEq, EnumIter, Serialize, Deserialize)]
pub enum TleParameterField {
    // Name,
    IntlDesig,
    SatNum,
    DesigYear,
    DesigLaunch,
    DesigPiece,
    Epoch,
    MeanMotionDot,
    MeanMotionDotDot,
    BStar,
    EphemType,
    ElementNum,
    Inclination,
    Raan,
    Eccentricity,
    ArgOfPerigee,
    MeanAnomaly,
    MeanMotion,
    RevNum,
}

impl TleParameterField {
    pub fn display_label(&self) -> &'static str {
        match self {
            // TleParameterField::Name => "Satellite Name",
            TleParameterField::IntlDesig => "International Designator",
            TleParameterField::SatNum => "Satellite NORAD Number",
            TleParameterField::DesigYear => "Launch Year",
            TleParameterField::DesigLaunch => "Launch Number of Year",
            TleParameterField::DesigPiece => "Launch Piece",
            TleParameterField::Epoch => "Epoch",
            TleParameterField::MeanMotionDot => "1st Derivative of Mean Motion (rev/day²)",
            TleParameterField::MeanMotionDotDot => "2nd Derivative of Mean Motion (rev/day³)",
            TleParameterField::BStar => "B* (1/Earth radii)",
            TleParameterField::EphemType => "Ephemeris Type",
            TleParameterField::ElementNum => "Element Number",
            TleParameterField::Inclination => "Inclination (deg)",
            TleParameterField::Raan => "RAAN (deg)",
            TleParameterField::Eccentricity => "Eccentricity",
            TleParameterField::ArgOfPerigee => "Argument of Perigee (deg)",
            TleParameterField::MeanAnomaly => "Mean Anomaly (deg)",
            TleParameterField::MeanMotion => "Mean Motion (rev/day)",
            TleParameterField::RevNum => "Revolution Number",
        }
    }

    /// Stringify the corresponding value from `TleData` for UI inputs.
    /// Adjust formatting/precision as needed for your app.
    pub fn format_value(&self, t: &TleData) -> String {
        match self {
            // TleParameterField::Name => t.name.clone(),
            TleParameterField::IntlDesig => t.intl_desig.clone(),
            TleParameterField::SatNum => format!("{}", t.sat_num),
            TleParameterField::DesigYear => format!("{}", t.desig_year),
            TleParameterField::DesigLaunch => format!("{}", t.desig_launch),
            TleParameterField::DesigPiece => t.desig_piece.clone(),
            TleParameterField::Epoch => t.epoch.as_iso8601(),
            TleParameterField::MeanMotionDot => format!("{}", t.mean_motion_dot),
            TleParameterField::MeanMotionDotDot => format!("{}", t.mean_motion_dot_dot),
            TleParameterField::BStar => format!("{}", t.bstar),
            TleParameterField::EphemType => format!("{}", t.ephem_type),
            TleParameterField::ElementNum => format!("{}", t.element_num),
            TleParameterField::Inclination => format!("{}", t.inclination),
            TleParameterField::Raan => format!("{}", t.raan),
            TleParameterField::Eccentricity => format!("{}", t.eccen),
            TleParameterField::ArgOfPerigee => format!("{}", t.arg_of_perigee),
            TleParameterField::MeanAnomaly => format!("{}", t.mean_anomaly),
            TleParameterField::MeanMotion => format!("{}", t.mean_motion),
            TleParameterField::RevNum => format!("{}", t.rev_num),
        }
    }
}

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

    pub tle_parameter_inputs: HashMap<TleParameterField, String>,
}
