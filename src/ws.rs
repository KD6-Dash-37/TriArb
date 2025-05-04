// src/ws.rs

use std::{collections::HashSet, future::Future, sync::Arc};
use anyhow::Result;
use bytes::Bytes;
use fastwebsockets::{FragmentCollector, Frame, OpCode, Payload};
use http_body_util::Empty;
use hyper::{
    header::{CONNECTION, UPGRADE},
    upgrade::Upgraded,
    Request,
};
use hyper_util::rt::TokioIo;
use tokio::{net::TcpStream, sync::mpsc::Sender};
use tokio_rustls::{
    rustls::{ClientConfig, OwnedTrustAnchor},
    TlsConnector,
};
use crate::price_path::PricingPath;

/// Starts a WebSocket connection and streams raw frames into the `tx` channel.
///
/// - Connects to either Binance (`wss://data-stream.binance.com`) or a local mock feed (`ws://localhost:9001`)
/// - Subscribes to `@bookTicker` channels for all symbols derived from the pricing paths
/// - Forwards raw WebSocket frames into the async channel for downstream parsing
///
/// # Parameters
/// - `price_paths`: The arbitrage pricing paths to extract symbols from
/// - `tx`: The receiving end of the stream pipeline
/// - `use_mock`: If `true`, connect to local mock server instead of Binance
pub async fn start_ws_listener(
    price_paths: Vec<PricingPath>,
    tx: Sender<Bytes>,
    local_domain: Option<bool>,
) -> Result<()> {

    let mut ws = if  local_domain.is_some() {
        tracing::info!("🔌 Connecting to local mock WebSocket feed at ws://localhost:9001...");
        connect_local().await?
    } else {
        let domain = "data-stream.binance.com";
        tracing::info!("🌐 Connecting to Binance at wss://{domain}:9443...");
        connect_exchange(domain).await?
    };

    let symbols = extract_symbols_from_paths(&price_paths);
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

/// Basic executor required by hyper handshake for spawning background tasks.
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

/// Configures the TLS connector using the system trust roots.
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


/// Connects to Binance using TLS and returns a WebSocket frame reader.
///
/// This establishes a secure `wss://` connection to Binance and completes
/// the WebSocket upgrade handshake.
async fn connect_exchange(domain: &str) -> Result<FragmentCollector<TokioIo<Upgraded>>> {
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


/// Connects to a local mock WebSocket server over plain TCP.
///
/// This simulates a Binance-like feed without TLS and performs a standard
/// WebSocket handshake with the local test server.
async fn connect_local() -> Result<FragmentCollector<TokioIo<Upgraded>>> {
    let addr = "localhost:9001";
    let stream = TcpStream::connect(addr).await?;
    tracing::info!("🧪 Local TCP connection established to {addr}");
    let req = Request::builder()
        .method("GET")
        .uri(format!("http://{addr}"))
        .header("Host", "localhost:9001")
        .header(UPGRADE, "websocket")
        .header(CONNECTION, "upgrade")
        .header(
        "Sec-WebSocket-Key",
        fastwebsockets::handshake::generate_key(),
        )
        .header("Sec-WebSocket-Version", "13")
        .body(Empty::<Bytes>::new())?;

    let (ws, _) =
        fastwebsockets::handshake::client(&SpawnExecutor, req, stream).await?;
    Ok(FragmentCollector::new(ws))
}

/// Subscribes to Binance's `@bookTicker` stream for the given symbols.
async fn subscribe_symbols(
    ws: &mut FragmentCollector<TokioIo<Upgraded>>,
    symbols: Vec<String>,
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

/// Extracts a de-duplicated list of symbols from the pricing paths.
///
/// Useful for determining which WebSocket channels to subscribe to.
pub fn extract_symbols_from_paths(price_paths: &[PricingPath]) -> Vec<String> {
    let mut symbols = HashSet::new();
    for path in price_paths {
        for s in path.symbols() {
            symbols.insert(s);
        }
    }
    symbols.into_iter().collect()
}
