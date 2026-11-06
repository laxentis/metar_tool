//! Sends METAR reports to configured HTTP endpoints.

use crate::i18n::{Language, localized};
use std::thread;
use std::time::Duration;

const REQUEST_TIMEOUT: Duration = Duration::from_secs(15);

#[derive(Debug, serde::Deserialize)]
struct ServerResponse {
    #[serde(default)]
    ok: Option<bool>,
    #[serde(default)]
    status: Option<u16>,
    #[serde(default)]
    icao: Option<String>,
    #[serde(default)]
    report_time: Option<String>,
    #[serde(default)]
    replaced: Option<bool>,
    #[serde(default)]
    error: Option<String>,
    #[serde(default)]
    message: Option<String>,
}

fn format_response(
    server: &str,
    status: reqwest::StatusCode,
    body: &str,
    language: Language,
) -> String {
    let parsed = serde_json::from_str::<ServerResponse>(body).ok();

    match status.as_u16() {
        200 => {
            if let Some(response) = parsed {
                let destination = response
                    .icao
                    .as_deref()
                    .unwrap_or(language.text("unknown ICAO", "nieznany kod ICAO"));
                let report_time = response
                    .report_time
                    .as_deref()
                    .map(|time| {
                        localized!(language, ", report time {time}", ", termin depeszy {time}")
                    })
                    .unwrap_or_default();
                let replacement = match response.replaced {
                    Some(true) => language.text(
                        " (replaced existing report)",
                        " (zastąpiono istniejącą depeszę)",
                    ),
                    Some(false) => language.text(" (new report)", " (nowa depesza)"),
                    None => "",
                };
                let ok = response.ok.unwrap_or(true);
                let response_status = response
                    .status
                    .map(|value| {
                        localized!(
                            language,
                            ", response status {value}",
                            ", status odpowiedzi {value}"
                        )
                    })
                    .unwrap_or_default();
                localized!(
                    language,
                    "{server}: accepted for {destination}{report_time}{replacement} (HTTP 200{response_status}, ok={ok})",
                    "{server}: przyjęto dla {destination}{report_time}{replacement} (HTTP 200{response_status}, ok={ok})"
                )
            } else {
                localized!(
                    language,
                    "{server}: accepted (HTTP 200), but returned invalid JSON",
                    "{server}: przyjęto (HTTP 200), ale zwrócono niepoprawny JSON"
                )
            }
        }
        400 => {
            if let Some(response) = parsed {
                let reason = response.error.as_deref().unwrap_or("invalid_report");
                let message = response
                    .message
                    .as_deref()
                    .unwrap_or(language.text("invalid METAR", "niepoprawna depesza METAR"));
                localized!(
                    language,
                    "{server}: rejected (HTTP 400, {reason}): {message}",
                    "{server}: odrzucono (HTTP 400, {reason}): {message}"
                )
            } else {
                localized!(
                    language,
                    "{server}: rejected (HTTP 400): {}",
                    "{server}: odrzucono (HTTP 400): {}",
                    response_body_or_default(
                        body,
                        language.text("invalid METAR", "niepoprawna depesza METAR")
                    )
                )
            }
        }
        401 => localized!(
            language,
            "{server}: rejected (HTTP 401): invalid API key",
            "{server}: odrzucono (HTTP 401): niepoprawny klucz API"
        ),
        404 => localized!(
            language,
            "{server}: not found (HTTP 404): check the server URL",
            "{server}: nie znaleziono (HTTP 404): sprawdź adres URL serwera"
        ),
        500..=599 => localized!(
            language,
            "{server}: server error (HTTP {}): try again shortly",
            "{server}: błąd serwera (HTTP {}): spróbuj ponownie za chwilę",
            status.as_u16()
        ),
        _ => localized!(
            language,
            "{server}: request failed (HTTP {}): {}",
            "{server}: żądanie nie powiodło się (HTTP {}): {}",
            status.as_u16(),
            response_body_or_default(
                body,
                language.text(
                    "unexpected server response",
                    "nieoczekiwana odpowiedź serwera"
                )
            )
        ),
    }
}

fn response_body_or_default(body: &str, default: &str) -> String {
    let body = body.trim();
    if body.is_empty() {
        default.to_owned()
    } else {
        body.to_owned()
    }
}

/// POST `report` as plain text to each configured URL, reporting each result
/// through `on_result` from the worker thread.
pub fn send_to_servers(
    servers: Vec<(String, String, String)>,
    report: String,
    language: Language,
    on_result: impl Fn(String) + Send + 'static,
) {
    thread::spawn(move || {
        let runtime = match tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
        {
            Ok(runtime) => runtime,
            Err(error) => {
                on_result(localized!(
                    language,
                    "Could not start sender: {error}",
                    "Nie można uruchomić wysyłania: {error}"
                ));
                return;
            }
        };

        runtime.block_on(async move {
            let client = match reqwest::Client::builder()
                .timeout(REQUEST_TIMEOUT)
                .build()
            {
                Ok(client) => client,
                Err(error) => {
                    on_result(localized!(language, "Could not create HTTP client: {error}", "Nie można utworzyć klienta HTTP: {error}"));
                    return;
                }
            };
            for (server, url, api_key) in servers {
                let mut request = client
                    .post(&url)
                    .header(reqwest::header::CONTENT_TYPE, "text/plain; charset=utf-8")
                    .body(report.clone());
                if !api_key.is_empty() {
                    request = request.header("X-API-Key", &api_key);
                }
                let result = match request.send().await {
                    Ok(response) => {
                        let status = response.status();
                        match response.text().await {
                            Ok(body) => format_response(&server, status, &body, language),
                            Err(error) => localized!(language, "{server}: received HTTP {status}, but could not read response: {}", "{server}: otrzymano HTTP {status}, ale nie można odczytać odpowiedzi: {}", error.without_url()
                            ),
                        }
                    }
                    Err(error) => localized!(language, "{server}: no response ({}); try again shortly", "{server}: brak odpowiedzi ({}); spróbuj ponownie za chwilę", error.without_url()
                    ),
                };
                on_result(result);
            }
        });
    });
}

#[cfg(test)]
mod tests {
    use super::format_response;
    use crate::i18n::Language;

    #[test]
    fn polish_response_preserves_server_content() {
        let result = format_response(
            "Serwer główny",
            reqwest::StatusCode::OK,
            r#"{"ok":true,"icao":"EPBC","report_time":"2026-11-05T12:00:00Z","replaced":true}"#,
            Language::Polish,
        );
        assert!(result.contains("przyjęto dla EPBC"));
        assert!(result.contains("zastąpiono istniejącą depeszę"));
        assert!(result.contains("2026-11-05T12:00:00Z"));

        let result = format_response(
            "Serwer główny",
            reqwest::StatusCode::BAD_REQUEST,
            r#"{"error":"missing_z","message":"time group must end in Z"}"#,
            Language::Polish,
        );
        assert!(result.contains("odrzucono"));
        assert!(result.contains("missing_z"));
        assert!(result.contains("time group must end in Z"));
    }

    #[test]
    fn accepted_new_report_is_described() {
        let result = format_response(
            "https://example.test/metar",
            reqwest::StatusCode::OK,
            r#"{"ok":true,"status":200,"icao":"EPBC","report_time":"2026-11-05T12:00:00Z","replaced":false}"#,
            Language::English,
        );

        assert!(result.contains("accepted for EPBC"));
        assert!(result.contains("new report"));
    }

    #[test]
    fn validation_error_includes_reason_and_message() {
        let result = format_response(
            "server",
            reqwest::StatusCode::BAD_REQUEST,
            r#"{"ok":false,"status":400,"error":"missing_z","message":"time group must end in Z"}"#,
            Language::English,
        );

        assert!(result.contains("missing_z"));
        assert!(result.contains("time group must end in Z"));
    }

    #[test]
    fn server_errors_suggest_retrying() {
        let result = format_response(
            "server",
            reqwest::StatusCode::INTERNAL_SERVER_ERROR,
            "",
            Language::English,
        );

        assert!(result.contains("try again shortly"));
    }

    #[test]
    fn not_found_response_suggests_checking_the_url() {
        let result = format_response(
            "server",
            reqwest::StatusCode::NOT_FOUND,
            "",
            Language::English,
        );

        assert!(result.contains("not found (HTTP 404)"));
        assert!(result.contains("check the server URL"));
    }
}
