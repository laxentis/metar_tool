use std::fmt;

/// Runway designation suffix (Left, Center, Right)
#[allow(dead_code)]
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

/// Runway identification with optional suffix
struct Runway {
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
enum MessageType {
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

/// Wind speed units (KT or MPS)
#[allow(dead_code)]
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

/// Cardinal direction
#[allow(dead_code)]
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

/// Wind varying between two directions
struct WindVariance(i32, i32);

impl fmt::Display for WindVariance {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}V{}", self.0, self.1)
    }
}

/// METAR wind data
struct WindData {
    direction: Option<i32>,
    speed: i32,
    gust_speed: Option<i32>,
    speed_units: SpeedUnits,
    variance: Option<WindVariance>,
}

impl fmt::Display for WindData {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut w = String::new();
        if self.direction.is_some() {
            w.push_str(format!("{:03}", self.direction.as_ref().unwrap()).as_ref());
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

/// Visibility in one of 8 cardinal directions
struct DirectionalVisibility(u32, Direction);

impl fmt::Display for DirectionalVisibility {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:04}{}", self.0, self.1.to_string())
    }
}

/// Raw visibility group
struct Visibility {
    visibility: u32,
    directional: Option<DirectionalVisibility>,
}

impl fmt::Display for Visibility {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut w = String::from(format!("{:04}", self.visibility));
        if self.directional.is_some() {
            w.push_str(format!(" {}", self.directional.as_ref().unwrap()).as_ref());
        }
        write!(f, "{}", w)
    }
}

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

enum RunwayVisualRangeSuffix {
    U,
    N,
    D
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

struct RunwayVisualRange {
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

enum PhenomenonIntensity{
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

enum PhenomenonCode {
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
    NSW
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
            PhenomenonCode::NSW => write!(f, "NSW")
        }
    }
}

enum PhenomenonModifier{
    MI,
    BC,
    PR,
    DR,
    BL,
    SH,
    TS,
    FZ,
    VC
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

struct Phenomenon {
    intensity: Option<PhenomenonIntensity>,
    modifiers: Option<Vec<PhenomenonModifier>>,
    code: PhenomenonCode,
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

/// A METAR or SPECI observation message
pub struct ObservationMessage {
    message_type: MessageType,
    location: String,
    observation_time: u32,
    is_auto: bool,
}

fn main() {
    println!("Hello, world!");
}

#[cfg(test)]
mod tests {
    use crate::PhenomenonIntensity::{Minus, Plus};
    use crate::PhenomenonModifier::SH;
    use super::*;
    use crate::RunwayVisualRangePrefix::P;
    use crate::RunwayVisualRangeSuffix::U;
    use crate::SpeedUnits::{Knots, MetersPerSecond};

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
        let rvr = RunwayVisualRange{
            runway: Runway { identifier: 28, suffix: Some(RunwaySuffix::L) },
            prefix: Some(P),
            range: 2000,
            suffix: None,
        };
        assert_eq!(rvr.to_string(), "R28L/P2000");
    }

    #[test]
    fn runway_visual_range_0125u() {
        let rvr = RunwayVisualRange{
            runway: Runway { identifier: 26, suffix: None },
            prefix: None,
            range: 125,
            suffix: Some(U),
        };
        assert_eq!(rvr.to_string(), "R26/0125U");
    }

    #[test]
    fn phenomenon_light_drizzle() {
        let phenomenon = Phenomenon{
            intensity: Some(Minus),
            modifiers: None,
            code: PhenomenonCode::DZ,
        };
        assert_eq!(phenomenon.to_string(), "-DZ");
    }

    #[test]
    fn phenomenon_heavy_showers_rain() {
        let phenomenon = Phenomenon{
            intensity: Some(Plus),
            modifiers: Some(vec![SH]),
            code: PhenomenonCode::RA,
        };
        assert_eq!(phenomenon.to_string(), "+SHRA");
    }

    #[test]
    fn phenomenon_vctsra() {
        let phenomenon = Phenomenon{
            intensity: None,
            modifiers: Some(vec![PhenomenonModifier::VC, PhenomenonModifier::TS]),
            code: PhenomenonCode::RA,
        };
        assert_eq!(phenomenon.to_string(), "VCTSRA");
    }
}
