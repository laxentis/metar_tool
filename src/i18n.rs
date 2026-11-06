//! Interface language. METAR codes and server-provided content remain unchanged.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum Language {
    #[default]
    English,
    Polish,
}
impl Language {
    pub fn text<'a>(self, english: &'a str, polish: &'a str) -> &'a str {
        match self {
            Self::English => english,
            Self::Polish => polish,
        }
    }
}
macro_rules! localized {
    ($language:expr, $english:literal, $polish:literal $(, $arg:expr)* $(,)?) => {
        match $language {
            $crate::i18n::Language::English => format!($english $(, $arg)*),
            $crate::i18n::Language::Polish => format!($polish $(, $arg)*),
        }
    };
}
pub(crate) use localized;

#[cfg(test)]
mod tests {
    use super::Language;

    #[test]
    fn language_round_trips_through_saved_settings() {
        let saved = serde_json::to_string(&Language::Polish).unwrap();
        assert_eq!(
            serde_json::from_str::<Language>(&saved).unwrap(),
            Language::Polish
        );
        assert_eq!(Language::default(), Language::English);
    }
}
