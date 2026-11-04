//! Data types for constructing and formatting METAR and SPECI reports.
//!
//! The types in this module represent report groups and their coded values.
//! Their [`fmt::Display`] implementations serialize those values in the
//! conventional METAR token format. The report model currently covers the
//! report header, visibility and weather groups, cloud layers, and pressure.

use std::fmt;
use std::fmt::Display;

/// Runway designation suffix identifying a parallel runway.
#[allow(dead_code)]
#[derive(serde::Serialize, serde::Deserialize)]
enum RunwaySuffix {
    L,
    C,
    R,
}

impl fmt::Display for RunwaySuffix {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            RunwaySuffix::L => write!(f, "L"),
            RunwaySuffix::C => write!(f, "C"),
            RunwaySuffix::R => write!(f, "R"),
        }
    }
}

/// Runway number with an optional parallel-runway suffix.
#[derive(serde::Serialize, serde::Deserialize)]
pub struct Runway {
    identifier: u32,
    suffix: Option<RunwaySuffix>,
}

impl fmt::Display for Runway {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if let Some(suffix) = &self.suffix {
            write!(f, "{:02}{}", self.identifier, suffix)
        } else {
            write!(f, "{:02}", self.identifier)
        }
    }
}

#[allow(dead_code)]
/// Kind of aviation observation report, including corrected reports.
#[derive(serde::Serialize, serde::Deserialize)]
pub enum MessageType {
    Metar,
    MetarCor,
    Speci,
    SpeciCor,
}

impl fmt::Display for MessageType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            MessageType::Metar => {
                write!(f, "METAR")
            }
            MessageType::MetarCor => {
                write!(f, "METAR COR")
            }
            MessageType::Speci => {
                write!(f, "SPECI")
            }
            MessageType::SpeciCor => {
                write!(f, "SPECI COR")
            }
        }
    }
}

/// Unit used to encode wind speed in a report.
#[allow(dead_code)]
#[derive(serde::Serialize, serde::Deserialize)]
enum SpeedUnits {
    Knots,
    MetersPerSecond,
}

impl fmt::Display for SpeedUnits {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            SpeedUnits::Knots => {
                write!(f, "KT")
            }
            SpeedUnits::MetersPerSecond => {
                write!(f, "MPS")
            }
        }
    }
}

/// One of the eight principal compass directions.
#[allow(dead_code)]
#[derive(serde::Serialize, serde::Deserialize)]
enum Direction {
    N,
    E,
    S,
    W,
    NE,
    NW,
    SE,
    SW,
}

impl fmt::Display for Direction {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Direction::N => write!(f, "N"),
            Direction::E => write!(f, "E"),
            Direction::S => write!(f, "S"),
            Direction::W => write!(f, "W"),
            Direction::NE => write!(f, "NE"),
            Direction::NW => write!(f, "NW"),
            Direction::SE => write!(f, "SE"),
            Direction::SW => write!(f, "SW"),
        }
    }
}

/// The endpoints of a variable wind direction range, in degrees.
#[derive(serde::Serialize, serde::Deserialize)]
pub struct WindVariance(pub i32, pub i32);

impl fmt::Display for WindVariance {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:03}V{:03}", self.0, self.1)
    }
}

/// Wind direction, speed, gust, units, and optional direction variation.
#[derive(serde::Serialize, serde::Deserialize)]
pub struct WindData {
    pub(crate) direction: Option<i32>,
    pub speed: i32,
    pub gust_speed: Option<i32>,
    speed_units: SpeedUnits,
    pub(crate) variance: Option<WindVariance>,
}

impl fmt::Display for WindData {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut w = String::new();
        if self.direction.is_some() {
            let dir = self.direction.unwrap() as f64 / 10.0;
            let dir = dir.round() * 10.0;
            let dir = dir as i32;
            w.push_str(format!("{:03}", dir).as_str());
        } else {
            w.push_str("VRB");
        }
        w.push_str(format!("{:02}", self.speed).as_str());
        if self.gust_speed.is_some() {
            w.push_str(format!("G{:02}", self.gust_speed.unwrap()).as_str());
        }
        w.push_str(self.speed_units.to_string().as_str());
        if self.variance.is_some() {
            w.push_str(format!(" {}", self.variance.as_ref().unwrap()).as_str());
        }
        write!(f, "{}", w)
    }
}

/// Visibility reported for a particular compass direction.
#[derive(serde::Serialize, serde::Deserialize)]
pub struct DirectionalVisibility(u32, Direction);

impl fmt::Display for DirectionalVisibility {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:04}{}", self.0, self.1.to_string())
    }
}

/// Prevailing visibility and optional directional visibility information.
#[derive(serde::Serialize, serde::Deserialize)]
pub struct Visibility {
    pub visibility: u32,
    pub directional: Option<DirectionalVisibility>,
}

impl fmt::Display for Visibility {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut w = String::new();
        if self.visibility >= 9999 {
            w.push_str("9999");
        } else if self.visibility >= 5000 {
            w.push_str(format!("{:04}", self.visibility / 1000 * 1000).as_str());
        } else {
            w.push_str(format!("{:04}", self.visibility).as_str());
        }

        if self.directional.is_some() {
            w.push_str(format!(" {}", self.directional.as_ref().unwrap()).as_ref());
        }
        write!(f, "{}", w)
    }
}

/// Qualifier indicating a runway visual range below or above its stated value.
#[derive(serde::Serialize, serde::Deserialize)]
enum RunwayVisualRangePrefix {
    M,
    P,
}

impl fmt::Display for RunwayVisualRangePrefix {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            RunwayVisualRangePrefix::M => write!(f, "M"),
            RunwayVisualRangePrefix::P => write!(f, "P"),
        }
    }
}

/// Trend of runway visual range: upward, unchanged, or downward.
#[derive(serde::Serialize, serde::Deserialize)]
enum RunwayVisualRangeSuffix {
    U,
    N,
    D,
}

impl fmt::Display for RunwayVisualRangeSuffix {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            RunwayVisualRangeSuffix::U => write!(f, "U"),
            RunwayVisualRangeSuffix::N => write!(f, "N"),
            RunwayVisualRangeSuffix::D => write!(f, "D"),
        }
    }
}

/// Runway visual range, with optional limit and trend indicators.
#[derive(serde::Serialize, serde::Deserialize)]
pub struct RunwayVisualRange {
    runway: Runway,
    prefix: Option<RunwayVisualRangePrefix>,
    range: u32,
    suffix: Option<RunwayVisualRangeSuffix>,
}

impl fmt::Display for RunwayVisualRange {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut w = String::from(format!("R{}/", self.runway.to_string()));
        if let Some(prefix) = &self.prefix {
            w.push_str(&prefix.to_string());
        }
        w.push_str(format!("{:04}", self.range).as_ref());
        if let Some(suffix) = &self.suffix {
            w.push_str(&suffix.to_string());
        }
        write!(f, "{}", w)
    }
}

/// Intensity qualifier for a present-weather phenomenon.
#[derive(serde::Serialize, serde::Deserialize)]
pub enum PhenomenonIntensity {
    Plus,
    Minus,
}

impl fmt::Display for PhenomenonIntensity {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            PhenomenonIntensity::Plus => write!(f, "+"),
            PhenomenonIntensity::Minus => write!(f, "-"),
        }
    }
}

/// WMO present-weather code used in a METAR report.
#[derive(serde::Serialize, serde::Deserialize)]
pub enum PhenomenonCode {
    DZ,
    RA,
    SN,
    SG,
    PL,
    GR,
    GS,
    UP,
    BR,
    FG,
    FU,
    VA,
    DU,
    SA,
    HZ,
    PO,
    SQ,
    FC,
    SS,
    DS,
    TS,
    NSW,
}

impl fmt::Display for PhenomenonCode {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            PhenomenonCode::DZ => write!(f, "DZ"),
            PhenomenonCode::RA => write!(f, "RA"),
            PhenomenonCode::SN => write!(f, "SN"),
            PhenomenonCode::SG => write!(f, "SG"),
            PhenomenonCode::PL => write!(f, "PL"),
            PhenomenonCode::GR => write!(f, "GR"),
            PhenomenonCode::GS => write!(f, "GS"),
            PhenomenonCode::UP => write!(f, "UP"),
            PhenomenonCode::BR => write!(f, "BR"),
            PhenomenonCode::FG => write!(f, "FG"),
            PhenomenonCode::FU => write!(f, "FU"),
            PhenomenonCode::VA => write!(f, "VA"),
            PhenomenonCode::DU => write!(f, "DU"),
            PhenomenonCode::SA => write!(f, "SA"),
            PhenomenonCode::HZ => write!(f, "HZ"),
            PhenomenonCode::PO => write!(f, "PO"),
            PhenomenonCode::SQ => write!(f, "SQ"),
            PhenomenonCode::FC => write!(f, "FC"),
            PhenomenonCode::SS => write!(f, "SS"),
            PhenomenonCode::DS => write!(f, "DS"),
            PhenomenonCode::TS => write!(f, "TS"),
            PhenomenonCode::NSW => write!(f, "NSW"),
        }
    }
}

/// Descriptor that modifies a present-weather phenomenon code.
#[derive(serde::Serialize, serde::Deserialize)]
pub enum PhenomenonModifier {
    MI,
    BC,
    PR,
    DR,
    BL,
    SH,
    TS,
    FZ,
    VC,
}

impl fmt::Display for PhenomenonModifier {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            PhenomenonModifier::MI => write!(f, "MI"),
            PhenomenonModifier::BC => write!(f, "BC"),
            PhenomenonModifier::PR => write!(f, "PR"),
            PhenomenonModifier::DR => write!(f, "DR"),
            PhenomenonModifier::BL => write!(f, "BL"),
            PhenomenonModifier::SH => write!(f, "SH"),
            PhenomenonModifier::TS => write!(f, "TS"),
            PhenomenonModifier::FZ => write!(f, "FZ"),
            PhenomenonModifier::VC => write!(f, "VC"),
        }
    }
}

/// A present-weather group with optional intensity and descriptors.
#[derive(serde::Serialize, serde::Deserialize)]
pub struct Phenomenon {
    pub intensity: Option<PhenomenonIntensity>,
    pub modifiers: Option<Vec<PhenomenonModifier>>,
    pub code: PhenomenonCode,
}

impl fmt::Display for Phenomenon {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut w = String::new();
        if let Some(intensity) = &self.intensity {
            w.push_str(&intensity.to_string());
        }
        if let Some(modifiers) = &self.modifiers {
            for modifier in modifiers {
                w.push_str(&modifier.to_string());
            }
        }
        w.push_str(self.code.to_string().as_ref());
        write!(f, "{}", w)
    }
}

/// Sky coverage category for a reported cloud layer.
#[derive(serde::Serialize, serde::Deserialize)]
pub enum CloudCoverage {
    FEW,
    SCT,
    BKN,
    OVC,
}

impl fmt::Display for CloudCoverage {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            CloudCoverage::FEW => write!(f, "FEW"),
            CloudCoverage::SCT => write!(f, "SCT"),
            CloudCoverage::BKN => write!(f, "BKN"),
            CloudCoverage::OVC => write!(f, "OVC"),
        }
    }
}

/// Convective cloud type that may be appended to a cloud layer.
#[derive(serde::Serialize, serde::Deserialize)]
pub enum CloudType {
    CB,
    TCU,
}

impl fmt::Display for CloudType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            CloudType::CB => write!(f, "CB"),
            CloudType::TCU => write!(f, "TCU"),
        }
    }
}

/// Cloud coverage, base height in feet, and optional cloud type.
#[derive(serde::Serialize, serde::Deserialize)]
pub struct CloudLayer {
    pub coverage: CloudCoverage,
    pub height: u32,
    pub cloud_type: Option<CloudType>,
}

impl fmt::Display for CloudLayer {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut w = String::from(format!("{}{:03}", self.coverage, self.height / 100));
        if let Some(cloud_type) = &self.cloud_type {
            w.push_str(&cloud_type.to_string());
        }
        write!(f, "{}", w)
    }
}

/// Visibility, runway visual ranges, weather phenomena, and cloud information.
#[derive(serde::Serialize, serde::Deserialize)]
pub struct VisibilityPhenomenaCloudsGroup {
    pub visibility: Visibility,
    pub runway_visual_ranges: Option<Vec<RunwayVisualRange>>,
    pub phenomena: Option<Vec<Phenomenon>>,
    pub cloud_layers: Option<Vec<CloudLayer>>,
}

impl VisibilityPhenomenaCloudsGroup {
    /// Returns whether the available visibility and cloud data permit CAVOK.
    fn is_cavok(&self) -> bool {
        self.visibility.visibility > 9999
            && (self.cloud_layers.is_none() || self.cloud_layers.as_ref().unwrap().is_empty()
            && (self.phenomena.is_none() || self.phenomena.as_ref().unwrap().is_empty())
        )
    }
}

impl fmt::Display for VisibilityPhenomenaCloudsGroup {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if self.is_cavok() {
            write!(f, "CAVOK")
        } else {
            let mut w = String::new();
            w.push_str(self.visibility.to_string().as_ref());
            if let Some(runway_visual_ranges) = &self.runway_visual_ranges {
                for rvr in runway_visual_ranges {
                    w.push_str(format!(" {}", rvr).as_str());
                }
            }
            if let Some(phenomena) = &self.phenomena {
                for phenomenon in phenomena {
                    w.push_str(format!(" {}", phenomenon).as_str());
                }
            }
            if let Some(cloud_layers) = &self.cloud_layers {
                if cloud_layers.is_empty() {
                    w.push_str(" NSC");
                } else {
                    for layer in cloud_layers {
                        w.push_str(format!(" {}", layer).as_str());
                    }
                }
            } else {
                w.push_str(" NSC");
            }
            write!(f, "{}", w)
        }
    }
}

/// Unit system used for the coded pressure setting.
#[derive(serde::Serialize, serde::Deserialize)]
pub enum PressureUnits {
    Hectopascals,
    InchesOfMercury,
}

/// Altimeter or QNH pressure value and its reporting unit.
#[derive(serde::Serialize, serde::Deserialize)]
pub struct PressureSetting {
    pub value: f32,
    pub units: PressureUnits,
}

impl Display for PressureSetting {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.units {
            PressureUnits::Hectopascals => write!(f, "Q{:04}", self.value as u32),
            PressureUnits::InchesOfMercury => {
                let fst = self.value.trunc();
                let snd = (self.value - fst) * 100_f32;
                write!(f, "A{:02}{}", fst as u32, snd.trunc() as u32)
            }
        }
    }
}

/// A METAR or SPECI observation message and its formatted report groups.
///
/// Formatting this value produces a report ending in `=`. The observation
/// time is represented as the day of the month and UTC time in `HHMM` form.
#[derive(serde::Serialize, serde::Deserialize)]
pub struct ObservationMessage {
    message_type: MessageType,
    pub location: String,
    pub observation_day: u32,
    pub observation_time: u32,
    is_auto: bool,
    pub(crate) wind: WindData,
    pub visibility_phenomena_clouds_group: VisibilityPhenomenaCloudsGroup,
    pub temperature: i32,
    pub dew_point: i32,
    pub pressure: PressureSetting,
}

impl ObservationMessage {
    pub(crate) fn default() -> Self {
        Self {
            message_type: MessageType::Metar,
            location: "EPBC".to_string(),
            observation_day: 1,
            observation_time: 600,
            is_auto: false,
            wind: WindData {
                direction: Some(280),
                speed: 5,
                gust_speed: None,
                speed_units: SpeedUnits::Knots,
                variance: None,
            },
            visibility_phenomena_clouds_group: VisibilityPhenomenaCloudsGroup {
                visibility: Visibility {
                    visibility: 10000,
                    directional: None,
                },
                runway_visual_ranges: None,
                phenomena: None,
                cloud_layers: None,
            },
            temperature: 15,
            dew_point: 15,
            pressure: PressureSetting {
                value: 1013.25,
                units: PressureUnits::Hectopascals,
            },
        }
    }
}

impl Display for ObservationMessage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut w = String::from(format!(
            "{} {} {:02}{:04}",
            self.message_type, self.location, self.observation_day, self.observation_time
        ));
        if self.is_auto {
            w.push_str(" AUTO");
        }
        w.push_str(format!(" {}", self.wind).as_str());
        w.push_str(
            format!(
                " {} {:02}/{:02} {}=",
                self.visibility_phenomena_clouds_group.to_string(),
                self.temperature,
                self.dew_point,
                self.pressure.to_string()
            )
            .as_str(),
        );
        write!(f, "{}", w)
    }
}

#[cfg(test)]
mod tests {
    use crate::metar::PhenomenonIntensity::*;
    use crate::metar::PhenomenonModifier::*;
    use crate::metar::RunwayVisualRangePrefix::*;
    use crate::metar::RunwayVisualRangeSuffix::*;
    use crate::metar::SpeedUnits::*;
    use crate::metar::*;

    #[test]
    fn simple_runway() {
        let rwy = Runway {
            identifier: 08,
            suffix: None,
        };
        assert_eq!(rwy.to_string(), "08");
    }

    #[test]
    fn suffixed_runway() {
        let rwy = Runway {
            identifier: 08,
            suffix: Some(RunwaySuffix::L),
        };
        assert_eq!(rwy.to_string(), "08L");
    }

    #[test]
    fn simple_wind() {
        let speed = WindData {
            direction: Some(20),
            speed: 10,
            gust_speed: None,
            speed_units: Knots,
            variance: None,
        };
        assert_eq!(speed.to_string(), "02010KT");
    }

    #[test]
    fn mps_wind() {
        let speed = WindData {
            direction: Some(20),
            speed: 10,
            gust_speed: None,
            speed_units: MetersPerSecond,
            variance: None,
        };
        assert_eq!(speed.to_string(), "02010MPS");
    }

    #[test]
    fn wind_variable() {
        let wind = WindData {
            direction: None,
            speed: 2,
            gust_speed: None,
            speed_units: Knots,
            variance: None,
        };
        assert_eq!(wind.to_string(), "VRB02KT");
    }

    #[test]
    fn wind_with_gusts() {
        let wind = WindData {
            direction: Some(320),
            speed: 10,
            gust_speed: Some(20),
            speed_units: Knots,
            variance: None,
        };
        assert_eq!(wind.to_string(), "32010G20KT");
    }

    #[test]
    fn wind_varying_between() {
        let wind = WindData {
            direction: Some(320),
            speed: 10,
            gust_speed: None,
            speed_units: Knots,
            variance: Some(WindVariance(200, 340)),
        };
        assert_eq!(wind.to_string(), "32010KT 200V340");
    }

    #[test]
    fn simple_visibility() {
        let vis = Visibility {
            visibility: 5000,
            directional: None,
        };
        assert_eq!(vis.to_string(), "5000");
    }

    #[test]
    fn visibility_with_directional() {
        let vis = Visibility {
            visibility: 1500,
            directional: Some(DirectionalVisibility(2000, Direction::E)),
        };
        assert_eq!(vis.to_string(), "1500 2000E");
    }

    #[test]
    fn runway_visual_range_p2000() {
        let rvr = RunwayVisualRange {
            runway: Runway {
                identifier: 28,
                suffix: Some(RunwaySuffix::L),
            },
            prefix: Some(P),
            range: 2000,
            suffix: None,
        };
        assert_eq!(rvr.to_string(), "R28L/P2000");
    }

    #[test]
    fn runway_visual_range_0125u() {
        let rvr = RunwayVisualRange {
            runway: Runway {
                identifier: 26,
                suffix: None,
            },
            prefix: None,
            range: 125,
            suffix: Some(U),
        };
        assert_eq!(rvr.to_string(), "R26/0125U");
    }

    #[test]
    fn phenomenon_light_drizzle() {
        let phenomenon = Phenomenon {
            intensity: Some(Minus),
            modifiers: None,
            code: PhenomenonCode::DZ,
        };
        assert_eq!(phenomenon.to_string(), "-DZ");
    }

    #[test]
    fn phenomenon_heavy_showers_rain() {
        let phenomenon = Phenomenon {
            intensity: Some(Plus),
            modifiers: Some(vec![SH]),
            code: PhenomenonCode::RA,
        };
        assert_eq!(phenomenon.to_string(), "+SHRA");
    }

    #[test]
    fn phenomenon_vc_ts_ra() {
        let phenomenon = Phenomenon {
            intensity: None,
            modifiers: Some(vec![PhenomenonModifier::VC, PhenomenonModifier::TS]),
            code: PhenomenonCode::RA,
        };
        assert_eq!(phenomenon.to_string(), "VCTSRA");
    }

    #[test]
    fn cloud_layer() {
        let layer = CloudLayer {
            coverage: CloudCoverage::BKN,
            height: 2000,
            cloud_type: Some(CloudType::CB),
        };
        assert_eq!(layer.to_string(), "BKN020CB");
    }

    #[test]
    fn vis_group() {
        let vg = VisibilityPhenomenaCloudsGroup {
            visibility: Visibility {
                visibility: 5000,
                directional: Some(DirectionalVisibility(2000, Direction::E)),
            },
            runway_visual_ranges: Some(vec![RunwayVisualRange {
                runway: Runway {
                    identifier: 26,
                    suffix: None,
                },
                prefix: Some(crate::metar::RunwayVisualRangePrefix::P),
                range: 2000,
                suffix: None,
            }]),
            phenomena: Some(vec![Phenomenon {
                intensity: None,
                modifiers: None,
                code: PhenomenonCode::DZ,
            }]),
            cloud_layers: Some(vec![CloudLayer {
                coverage: CloudCoverage::FEW,
                height: 1500,
                cloud_type: None,
            }]),
        };
        assert_eq!(vg.to_string(), "5000 2000E R26/P2000 DZ FEW015");
    }

    #[test]
    fn vis_group_cavok() {
        let vg = VisibilityPhenomenaCloudsGroup {
            visibility: Visibility {
                visibility: 12000,
                directional: None,
            },
            runway_visual_ranges: None,
            phenomena: None,
            cloud_layers: None,
        };
        assert_eq!(vg.to_string(), "CAVOK");
    }

    #[test]
    fn vis_group_9000_nsc() {
        let vg = VisibilityPhenomenaCloudsGroup {
            visibility: Visibility {
                visibility: 9000,
                directional: None,
            },
            runway_visual_ranges: None,
            phenomena: None,
            cloud_layers: None,
        };
        assert_eq!(vg.to_string(), "9000 NSC");
    }

    #[test]
    fn qnh() {
        let ps = PressureSetting {
            units: PressureUnits::Hectopascals,
            value: 1013.25,
        };
        assert_eq!(ps.to_string(), "Q1013");
    }

    #[test]
    fn qnh_in_hg() {
        let ps = PressureSetting {
            units: PressureUnits::InchesOfMercury,
            value: 29.92,
        };
        assert_eq!(ps.to_string(), "A2992");
    }

    #[test]
    fn metar() {
        let observation = ObservationMessage {
            message_type: MessageType::Metar,
            location: "EPBC".to_string(),
            observation_day: 1,
            observation_time: 600,
            is_auto: true,
            wind: WindData {
                direction: Some(280),
                speed: 5,
                gust_speed: None,
                speed_units: SpeedUnits::Knots,
                variance: None,
            },
            visibility_phenomena_clouds_group: VisibilityPhenomenaCloudsGroup {
                visibility: Visibility {
                    visibility: 5000,
                    directional: None,
                },
                runway_visual_ranges: None,
                phenomena: Some(vec![Phenomenon {
                    intensity: None,
                    modifiers: None,
                    code: PhenomenonCode::DZ,
                }]),
                cloud_layers: Some(vec![CloudLayer {
                    coverage: CloudCoverage::FEW,
                    height: 1200,
                    cloud_type: None,
                }]),
            },
            temperature: 15,
            dew_point: 5,
            pressure: PressureSetting {
                value: 1013.25,
                units: PressureUnits::Hectopascals,
            },
        };
        assert_eq!(
            observation.to_string(),
            "METAR EPBC 010600 AUTO 28005KT 5000 DZ FEW012 15/05 Q1013="
        );
    }
}
