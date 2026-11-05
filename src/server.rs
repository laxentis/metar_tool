//! Sends METAR reports to configured HTTP endpoints.

use std::thread;

/// POST `report` as plain text to each configured URL, reporting each result
/// through `on_result` from the worker thread.
pub fn send_to_servers(
    servers: Vec<String>,
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
            let client = reqwest::Client::new();
            for server in servers {
                let result = match client
                    .post(&server)
                    .header(reqwest::header::CONTENT_TYPE, "text/plain; charset=utf-8")
                    .body(report.clone())
                    .send()
                    .await
                {
                    Ok(response) if response.status().is_success() => {
                        format!("{server}: sent (HTTP {})", response.status())
                    }
                    Ok(response) => format!("{server}: rejected (HTTP {})", response.status()),
                    Err(error) => format!("{server}: failed ({error})"),
                };
                on_result(result);
            }
        });
    });
}
