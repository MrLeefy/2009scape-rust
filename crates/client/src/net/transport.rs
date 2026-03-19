//! TCP transport layer for native and future WebSocket support.

use tokio::net::TcpStream;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use anyhow::Result;
use log::info;

/// Network transport abstraction.
/// On native: direct TCP. On WASM: would use WebSocket (via proxy).
pub struct Transport {
    stream: TcpStream,
}

impl Transport {
    /// Connect to a game server.
    pub async fn connect(host: &str, port: u16) -> Result<Self> {
        let addr = format!("{}:{}", host, port);
        info!("Connecting to {}...", addr);
        let stream = TcpStream::connect(&addr).await?;
        stream.set_nodelay(true)?;
        info!("Connected to {}", addr);
        Ok(Transport { stream })
    }

    /// Send raw bytes.
    pub async fn write(&mut self, data: &[u8]) -> Result<()> {
        self.stream.write_all(data).await?;
        self.stream.flush().await?;
        Ok(())
    }

    /// Read exactly `n` bytes.
    pub async fn read_exact(&mut self, buf: &mut [u8]) -> Result<()> {
        self.stream.read_exact(buf).await?;
        Ok(())
    }

    /// Read up to `buf.len()` bytes, returning how many were read.
    pub async fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
        let n = self.stream.read(buf).await?;
        Ok(n)
    }

    /// Read a single byte.
    pub async fn read_byte(&mut self) -> Result<u8> {
        let mut buf = [0u8; 1];
        self.read_exact(&mut buf).await?;
        Ok(buf[0])
    }
}
