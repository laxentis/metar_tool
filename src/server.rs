//! Sends METAR reports to configured HTTP endpoints.

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

fn format_response(server: &str, status: reqwest::StatusCode, body: &str) -> String {
    let parsed = serde_json::from_str::<ServerResponse>(body).ok();

    match status.as_u16() {
        200 => {
            if let Some(response) = parsed {
                let destination = response.icao.as_deref().unwrap_or("unknown ICAO");
                let report_time = response
                    .report_time
                    .as_deref()
                    .map(|time| format!(", report time {time}"))
                    .unwrap_or_default();
                let replacement = match response.replaced {
                    Some(true) => " (replaced existing report)",
                    Some(false) => " (new report)",
                    None => "",
                };
                let ok = response.ok.unwrap_or(true);
                let response_status = response
                    .status
                    .map(|value| format!(", response status {value}"))
                    .unwrap_or_default();
                format!(
                    "{server}: accepted for {destination}{report_time}{replacement} (HTTP 200{response_status}, ok={ok})"
                )
            } else {
                format!("{server}: accepted (HTTP 200), but returned invalid JSON")
            }
        }
        400 => {
            if let Some(response) = parsed {
                let reason = response.error.as_deref().unwrap_or("invalid_report");
                let message = response.message.as_deref().unwrap_or("invalid METAR");
                format!("{server}: rejected (HTTP 400, {reason}): {message}")
            } else {
                format!(
                    "{server}: rejected (HTTP 400): {}",
                    response_body_or_default(body, "invalid METAR")
                )
            }
        }
        401 => format!("{server}: rejected (HTTP 401): invalid API key"),
        404 => format!("{server}: not found (HTTP 404): check the server URL"),
        500..=599 => format!(
            "{server}: server error (HTTP {}): try again shortly",
            status.as_u16()
        ),
        _ => format!(
            "{server}: request failed (HTTP {}): {}",
            status.as_u16(),
            response_body_or_default(body, "unexpected server response")
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
    on_result: impl Fn(String) + Send + 'static,
) {
    thread::spawn(move || {
        let runtime = match tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
        {
            Ok(runtime) => runtime,
            Err(error) => {
                on_result(format!("Could not start sender: {error}"));
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
                    on_result(format!("Could not create HTTP client: {error}"));
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
                            Ok(body) => format_response(&server, status, &body),
                            Err(error) => format!(
                                "{server}: received HTTP {status}, but could not read response: {}", error.without_url()
                            ),
                        }
                    }
                    Err(error) => format!(
                        "{server}: no response ({}); try again shortly", error.without_url()
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

    #[test]
    fn accepted_new_report_is_described() {
        let result = format_response(
            "https://example.test/metar",
            reqwest::StatusCode::OK,
            r#"{"ok":true,"status":200,"icao":"EPBC","report_time":"2026-11-05T12:00:00Z","replaced":false}"#,
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
        );

        assert!(result.contains("missing_z"));
        assert!(result.contains("time group must end in Z"));
    }

    #[test]
    fn server_errors_suggest_retrying() {
        let result = format_response("server", reqwest::StatusCode::INTERNAL_SERVER_ERROR, "");

        assert!(result.contains("try again shortly"));
    }

    #[test]
    fn not_found_response_suggests_checking_the_url() {
        let result = format_response("server", reqwest::StatusCode::NOT_FOUND, "");

        assert!(result.contains("not found (HTTP 404)"));
        assert!(result.contains("check the server URL"));
    }
}
