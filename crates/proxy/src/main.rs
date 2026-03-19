//! WebSocket-to-TCP proxy for browser clients.
//!
//! Accepts WebSocket connections and forwards raw bytes to a 2009Scape game server
//! over TCP. The game server is completely unaware a proxy exists.

use anyhow::Result;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tokio_tungstenite::accept_async;
use futures_util::{SinkExt, StreamExt};
use log::{info, error};

const DEFAULT_WS_PORT: u16 = 8081;
const DEFAULT_GAME_HOST: &str = "test.2009scape.org";
const DEFAULT_GAME_PORT: u16 = 43594;

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::init();

    let ws_addr = format!("0.0.0.0:{}", DEFAULT_WS_PORT);
    let game_addr = format!("{}:{}", DEFAULT_GAME_HOST, DEFAULT_GAME_PORT);

    info!("RS2 WebSocket Proxy starting...");
    info!("  WebSocket: ws://{}", ws_addr);
    info!("  Game server: {}", game_addr);

    let listener = TcpListener::bind(&ws_addr).await?;
    info!("Listening for WebSocket connections...");

    while let Ok((stream, addr)) = listener.accept().await {
        let game_addr = game_addr.clone();
        tokio::spawn(async move {
            info!("New connection from {}", addr);
            if let Err(e) = handle_connection(stream, &game_addr).await {
                error!("Connection error from {}: {}", addr, e);
            }
            info!("Connection closed: {}", addr);
        });
    }

    Ok(())
}

async fn handle_connection(ws_stream: TcpStream, game_addr: &str) -> Result<()> {
    let ws = accept_async(ws_stream).await?;
    let (mut ws_write, mut ws_read) = ws.split();

    let tcp = TcpStream::connect(game_addr).await?;
    let (mut tcp_read, mut tcp_write) = tcp.into_split();

    // WebSocket → TCP
    let ws_to_tcp = tokio::spawn(async move {
        while let Some(msg) = ws_read.next().await {
            match msg {
                Ok(msg) if msg.is_binary() => {
                    if tcp_write.write_all(&msg.into_data()).await.is_err() {
                        break;
                    }
                }
                Ok(msg) if msg.is_close() => break,
                Err(_) => break,
                _ => {}
            }
        }
    });

    // TCP → WebSocket
    let tcp_to_ws = tokio::spawn(async move {
        let mut buf = vec![0u8; 4096];
        loop {
            match tcp_read.read(&mut buf).await {
                Ok(0) => break,
                Ok(n) => {
                    let msg = tokio_tungstenite::tungstenite::Message::Binary(buf[..n].to_vec().into());
                    if ws_write.send(msg).await.is_err() {
                        break;
                    }
                }
                Err(_) => break,
            }
        }
    });

    tokio::select! {
        _ = ws_to_tcp => {},
        _ = tcp_to_ws => {},
    }

    Ok(())
}
