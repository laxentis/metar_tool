use std::fmt;

enum MessageType {
    Metar,
    MetarCor,
    Speci,
    SpeciCor,
}

impl fmt::Display for MessageType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            MessageType::Metar => {write!(f, "METAR")}
            MessageType::MetarCor => {write!(f, "METAR COR")}
            MessageType::Speci => {write!(f, "SPECI")}
            MessageType::SpeciCor => {write!(f, "SPECI COR")}
        }
    }
}

enum SpeedUnits {
    Knots,
    MetersPerSecond,
}

impl fmt::Display for SpeedUnits {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            SpeedUnits::Knots => {write!(f, "KT")}
            SpeedUnits::MetersPerSecond => {write!(f, "MPS")}
        }
    }
}

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

struct WindVariance(i32, i32);

impl fmt::Display for WindVariance {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}V{}", self.0, self.1)
    }
}

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

struct DirectionalVisibility (u32, Direction);

impl fmt::Display for DirectionalVisibility {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self.1 {
            Direction::N => {write!(f, "{:04}N", self.0)},
            Direction::E => {write!(f, "{:04}E", self.0)},
            Direction::S => {write!(f, "{:04}S", self.0)},
            Direction::W => {write!(f, "{:04}W", self.0)},
            Direction::NE => {write!(f, "{:04}NE", self.0)},
            Direction::NW => {write!(f, "{:04}NW", self.0)},
            Direction::SE => {write!(f, "{:04}SE", self.0)},
            Direction::SW => {write!(f, "{:04}SW", self.0)},
        }
    }
}

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

struct ObservationMessage {
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
    use crate::SpeedUnits::{Knots, MetersPerSecond};
    use super::*;

    #[test]
    fn simple_wind(){
        let speed = WindData { direction: Some(20), speed: 10, gust_speed: None, speed_units: Knots, variance: None };
        assert_eq!(speed.to_string(), "02010KT");
    }

    #[test]
    fn mps_wind(){
        let speed = WindData { direction: Some(20), speed: 10, gust_speed: None, speed_units: MetersPerSecond, variance: None };
        assert_eq!(speed.to_string(), "02010MPS");
    }

    #[test]
    fn wind_variable(){
        let wind = WindData { direction: None, speed: 2, gust_speed: None, speed_units: Knots, variance: None };
        assert_eq!(wind.to_string(), "VRB02KT");
    }

    #[test]
    fn wind_with_gusts(){
        let wind = WindData { direction: Some(320), speed: 10, gust_speed: Some(20), speed_units: Knots, variance: None };
        assert_eq!(wind.to_string(), "32010G20KT");
    }

    #[test]
    fn wind_varying_between(){
        let wind = WindData { direction: Some(320), speed: 10, gust_speed: None, speed_units: Knots, variance: Some(WindVariance(200, 340)) };
        assert_eq!(wind.to_string(), "32010KT 200V340");
    }

    #[test]
    fn simple_visibility() {
        let vis = Visibility { visibility: 5000, directional: None };
        assert_eq!(vis.to_string(), "5000");
    }

    #[test]
    fn visibility_with_directional(){
        let vis = Visibility { visibility: 1500, directional: Some(DirectionalVisibility(2000, Direction::E)) };
        assert_eq!(vis.to_string(), "1500 2000E");
    }
}