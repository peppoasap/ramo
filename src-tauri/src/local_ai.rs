//! Cancellable loopback HTTP. Dropping the request also closes its local transport.
use crate::cancellation::Cancellation;
use std::time::Duration;
const LIMIT: usize = 256 * 1024;
pub(crate) fn generate(
    prompt: &str,
    model: &str,
    cancellation: &Cancellation,
) -> Result<serde_json::Value, String> {
    request(
        "http://127.0.0.1:11434/api/chat",
        serde_json::json!({"model":model,"stream":false,"messages":[{"role":"user","content":prompt}],"options":{"num_predict":1200}}),
        cancellation,
    )
}
fn request(
    url: &str,
    body: serde_json::Value,
    cancellation: &Cancellation,
) -> Result<serde_json::Value, String> {
    cancellation.check()?;
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .map_err(|_| "Cannot initialize local AI transport")?;
    runtime.block_on(async {
        let mut pending = Box::pin(async {
            let client = reqwest::Client::builder()
                .no_proxy()
                .redirect(reqwest::redirect::Policy::none())
                .pool_max_idle_per_host(0)
                .timeout(Duration::from_secs(120))
                .build()
                .map_err(|_| "Cannot initialize the local AI connection")?;
            let mut response = client
                .post(url)
                .json(&body)
                .send()
                .await
                .map_err(|e| format!("Cannot reach local Ollama: {e}"))?;
            if !response.status().is_success() {
                return Err(format!(
                    "Local Ollama returned {}. Redirects are not followed.",
                    response.status()
                ));
            }
            let mut bytes = Vec::new();
            while let Some(chunk) = response
                .chunk()
                .await
                .map_err(|_| "Cannot read the local Ollama response")?
            {
                if bytes.len() + chunk.len() > LIMIT {
                    return Err("Local Ollama response exceeded the 256 KiB limit".into());
                }
                bytes.extend_from_slice(&chunk);
            }
            serde_json::from_slice(&bytes).map_err(|_| "Invalid local Ollama response".into())
        });
        loop {
            cancellation.check()?;
            match tokio::time::timeout(Duration::from_millis(50), pending.as_mut()).await {
                Ok(result) => {
                    cancellation.check()?;
                    return result;
                }
                Err(_) => continue,
            }
        }
    })
}
#[cfg(test)]
mod tests {
    use super::*;
    use std::{
        io::{Read, Write},
        net::TcpListener,
        sync::mpsc,
        thread,
        time::Instant,
    };
    #[test]
    fn cancellation_closes_connections_waiting_for_headers_or_more_body() {
        for mid_body in [false, true] {
            let listener = TcpListener::bind("127.0.0.1:0").unwrap();
            let url = format!("http://{}", listener.local_addr().unwrap());
            let (ready_tx, ready_rx) = mpsc::channel::<()>();
            let (closed_tx, closed_rx) = mpsc::channel();
            let server = thread::spawn(move || {
                let (mut stream, _) = listener.accept().unwrap();
                stream
                    .set_read_timeout(Some(Duration::from_secs(4)))
                    .unwrap();
                let mut data = [0; 4096];
                stream.read(&mut data).unwrap();
                if mid_body {
                    stream
                        .write_all(b"HTTP/1.1 200 OK\r\nContent-Length: 1000\r\n\r\n{")
                        .unwrap();
                }
                ready_tx.send(()).unwrap();
                loop {
                    match stream.read(&mut data) {
                        Ok(0) => {
                            closed_tx.send(()).unwrap();
                            break;
                        }
                        Ok(_) => {}
                        Err(e) => panic!("client connection did not close: {e}"),
                    }
                }
            });
            let token = Cancellation::default();
            let worker_token = token.clone();
            let worker = thread::spawn(move || {
                request(&url, serde_json::json!({"synthetic":true}), &worker_token)
            });
            ready_rx.recv_timeout(Duration::from_secs(3)).unwrap();
            let start = Instant::now();
            token.cancel();
            assert!(worker.join().unwrap().unwrap_err().contains("cancelled"));
            assert!(start.elapsed() < Duration::from_secs(2));
            closed_rx.recv_timeout(Duration::from_secs(3)).unwrap();
            server.join().unwrap();
        }
    }
    fn serve(response: Vec<u8>) -> (String, thread::JoinHandle<()>) {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let url = format!("http://{}", listener.local_addr().unwrap());
        let worker = thread::spawn(move || {
            let (mut stream, _) = listener.accept().unwrap();
            stream
                .set_read_timeout(Some(Duration::from_secs(3)))
                .unwrap();
            let mut data = [0; 4096];
            stream.read(&mut data).unwrap();
            let _ = stream.write_all(&response);
        });
        (url, worker)
    }
    #[test]
    fn cancellable_http_retains_response_limits_redirect_refusal_and_valid_json() {
        for (body, valid) in [
            (b"{\"message\":{\"content\":\"synthetic\"}}".to_vec(), true),
            (vec![b'x'; LIMIT + 1], false),
        ] {
            let mut response = format!(
                "HTTP/1.1 200 OK\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
                body.len()
            )
            .into_bytes();
            response.extend(body);
            let (url, worker) = serve(response);
            let result = request(&url, serde_json::json!({}), &Cancellation::default());
            worker.join().unwrap();
            if valid {
                assert_eq!(result.unwrap()["message"]["content"], "synthetic");
            } else {
                assert!(result.unwrap_err().contains("256 KiB"));
            }
        }
        let destination = TcpListener::bind("127.0.0.1:0").unwrap();
        destination.set_nonblocking(true).unwrap();
        let(url,worker)=serve(format!("HTTP/1.1 307 Temporary Redirect\r\nLocation: http://{}/leak\r\nContent-Length: 0\r\nConnection: close\r\n\r\n",destination.local_addr().unwrap()).into_bytes());
        assert!(
            request(&url, serde_json::json!({}), &Cancellation::default())
                .unwrap_err()
                .contains("307")
        );
        worker.join().unwrap();
        assert_eq!(
            destination.accept().unwrap_err().kind(),
            std::io::ErrorKind::WouldBlock
        );
    }
    #[test]
    fn cancelled_http_never_connects() {
        let server = TcpListener::bind("127.0.0.1:0").unwrap();
        server.set_nonblocking(true).unwrap();
        let token = Cancellation::default();
        token.cancel();
        assert!(request(
            &format!("http://{}", server.local_addr().unwrap()),
            serde_json::json!({}),
            &token
        )
        .unwrap_err()
        .contains("cancelled"));
        assert_eq!(
            server.accept().unwrap_err().kind(),
            std::io::ErrorKind::WouldBlock
        );
    }
}
#[cfg(test)]
mod proxy_tests {
    use super::*;
    use std::{
        io::{Read, Write},
        net::TcpListener,
        process::Command,
        thread,
    };
    #[test]
    fn proxy_probe() {
        let Ok(url) = std::env::var("RAMO_CANCEL_PROXY_PROBE") else {
            return;
        };
        let value = request(
            &url,
            serde_json::json!({"synthetic":true}),
            &Cancellation::default(),
        )
        .unwrap();
        assert_eq!(value["message"]["content"], "local fixture");
    }
    #[test]
    fn cancellable_transport_bypasses_environment_proxies() {
        let proxy = TcpListener::bind("127.0.0.1:0").unwrap();
        proxy.set_nonblocking(true).unwrap();
        let server = TcpListener::bind("127.0.0.1:0").unwrap();
        let url = format!("http://{}", server.local_addr().unwrap());
        let worker = thread::spawn(move || {
            let (mut stream, _) = server.accept().unwrap();
            stream
                .set_read_timeout(Some(Duration::from_secs(3)))
                .unwrap();
            let mut bytes = [0; 4096];
            stream.read(&mut bytes).unwrap();
            let body = r#"{"message":{"content":"local fixture"}}"#;
            let _ = write!(
                stream,
                "HTTP/1.1 200 OK\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                body.len(),
                body
            );
        });
        let proxy_url = format!("http://{}", proxy.local_addr().unwrap());
        let mut child = Command::new(std::env::current_exe().unwrap());
        child
            .args([
                "--exact",
                "local_ai::proxy_tests::proxy_probe",
                "--nocapture",
            ])
            .env("RAMO_CANCEL_PROXY_PROBE", url);
        for key in [
            "HTTP_PROXY",
            "http_proxy",
            "HTTPS_PROXY",
            "https_proxy",
            "ALL_PROXY",
            "all_proxy",
        ] {
            child.env(key, &proxy_url);
        }
        child.env("NO_PROXY", "").env("no_proxy", "");
        assert!(child.status().unwrap().success());
        worker.join().unwrap();
        assert_eq!(
            proxy.accept().unwrap_err().kind(),
            std::io::ErrorKind::WouldBlock
        );
    }
}
