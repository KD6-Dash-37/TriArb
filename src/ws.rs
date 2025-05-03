// src/ws.rs
use std::future::Future;
use std::sync::Arc;

use anyhow::Result;
use bytes::Bytes;
use fastwebsockets::FragmentCollector;
use fastwebsockets::Frame;
use fastwebsockets::OpCode;
use fastwebsockets::Payload;
use http_body_util::Empty;
use hyper::header::CONNECTION;
use hyper::header::UPGRADE;
use hyper::upgrade::Upgraded;
use hyper::Request;
use hyper_util::rt::TokioIo;
use tokio::net::TcpStream;
use tokio_rustls::rustls::ClientConfig;
use tokio_rustls::rustls::OwnedTrustAnchor;
use tokio_rustls::TlsConnector;
use tokio::sync::mpsc::Sender;

struct SpawnExecutor;

impl <Fut> hyper::rt::Executor<Fut> for SpawnExecutor
    where 
        Fut: Future + Send + 'static,
        Fut::Output: Send + 'static,
    {
        fn execute(&self, fut: Fut) {
            tokio::task::spawn(fut);
        }
    }

fn tls_connector() -> Result<TlsConnector> {
    let mut root_store = tokio_rustls::rustls::RootCertStore::empty();
    
    root_store.add_trust_anchors(webpki_roots::TLS_SERVER_ROOTS.0.iter().map(
        |ta| {
          OwnedTrustAnchor::from_subject_spki_name_constraints(
            ta.subject,
            ta.spki,
            ta.name_constraints,
          )
        },
      ));

    let config = ClientConfig::builder()
        .with_safe_defaults()
        .with_root_certificates(root_store)
        .with_no_client_auth();

    Ok(TlsConnector::from(Arc::new(config)))
}

async fn connect(domain: &str) -> Result<FragmentCollector<TokioIo<Upgraded>>> {
    let mut addr = String::from(domain);
    addr.push_str(":9443");

    let tcp_stream = TcpStream::connect(&addr).await?;
    let tls_connector = tls_connector().unwrap();
    let domain = 
        tokio_rustls::rustls::ServerName::try_from(domain).map_err(|_| {
            std::io::Error::new(std::io::ErrorKind::InvalidInput, "invalid dns name")
        })?;
    
    let tls_stream = tls_connector.connect(domain, tcp_stream).await?;

    let req = Request::builder()
        .method("GET")
        .uri(format!("wss://{}/ws", &addr))
        .header("Host", &addr)
        .header(UPGRADE, "websocket")
        .header(CONNECTION, "upgrade")
        .header(
            "Sec-WebSocket-Key",
            fastwebsockets::handshake::generate_key(),  
        )
        .header("Sec-WebSocket-Version", "13")
        .body(Empty::<Bytes>::new())?;

    let (ws, _) = 
        fastwebsockets::handshake::client(&SpawnExecutor, req, tls_stream).await?;
    
    Ok(FragmentCollector::new(ws))
}

async fn subscribe_symbols(
    ws: &mut FragmentCollector<TokioIo<Upgraded>>,
    symbols: Vec<&str>,
) -> Result<()> {
    let params: Vec<String> = symbols.iter()
        .map(|s| format!("{}@bookTicker", s.to_lowercase()))
        .collect();

    let subscribe_message = serde_json::json!({
        "method": "SUBSCRIBE",
        "params": params,
    });

    let subscribe_payload = serde_json::to_string(&subscribe_message)?;
    ws.write_frame(Frame::text(subscribe_payload.into_bytes().into())).await?;
    Ok(())
}

pub async fn start_ws_listener(
    symbols: Vec<&str>,
    tx: Sender<Bytes>
) -> Result<()> {
    
    let domain = "data-stream.binance.com";
    let mut ws = connect(domain).await?;
    
    subscribe_symbols(&mut ws, symbols).await?;
    
    loop {
        let frame = match ws.read_frame().await {
            Ok(frame) => frame,
            Err(e) => {
                eprintln!("Websocket error: {e}");
                ws.write_frame(Frame::close_raw(vec![].into())).await?;
                break;
            }
        };

        match frame.opcode {
            OpCode::Text | OpCode::Binary => {
                match frame.payload {
                    Payload::Bytes(data) => {
                        tx.send(data.into()).await?;
                    }
                    Payload::Borrowed(data) => {
                        tx.send(Bytes::copy_from_slice(data)).await?;
                    }
                    Payload::BorrowedMut(data) => {
                        tx.send(Bytes::copy_from_slice(&*data)).await?;
                    }
                    Payload::Owned(data) => {
                        tx.send(data.into()).await?;
                    }
                }
            }
            OpCode::Close => {
                println!("WebSocket Close frame received");
                break;
            }
            _ => {
                // Ignore test
            }
        }
    }
    Ok::<_, anyhow::Error>(())
}