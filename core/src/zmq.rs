use std::time::Duration;
use log::{error, info};
use tokio::time::sleep;
use zeromq::{Socket, SocketRecv, SubSocket};

use crate::error::VoltaTestError;

/// ZMQ base configuration
pub struct ZeromqConfig {
    /// Connection port
    pub port: u16,

    /// Connection host
    pub addr: String,

    /// Connection protocol
    pub protocol: String,
}

impl ZeromqConfig {
    /// Convert a configuration into an endpoint for connect and bind
    pub fn url(&self) -> String {
        format!("{}://{}:{}", self.protocol, self.addr, self.port)
    }
}

/// Prefered configuration
impl Default for ZeromqConfig {
    fn default() -> Self {
        Self {
            port: 9000,
            addr: "127.0.0.1".to_string(),
            protocol: "tcp".to_string(),
        }
    }
}

pub struct ZeromqClient {
    socket: SubSocket,
    url: String,
}

impl ZeromqClient {

    /// Connect client to ZeroMQ based on provided configuration
    pub async fn connect(config: ZeromqConfig) -> Result<Self, VoltaTestError> {
        let url = config.url();
        let socket = Self::make_socket(&url).await?;
        Ok(Self { socket, url })
    }

    async fn make_socket(url: &str) -> Result<SubSocket, VoltaTestError> {
        let mut socket = SubSocket::new();
        socket
            .connect(url)
            .await
            .map_err(|e| VoltaTestError::alerting_error(format!("connect: {e}")))?;
        socket
            .subscribe("")
            .await
            .map_err(|e| VoltaTestError::alerting_error(format!("subscribe: {e}")))?;
        Ok(socket)
    }

    /// Receive next message from ZeroMQ
    pub async fn recv(&mut self) -> Vec<u8> {
        loop {
            match self.socket.recv().await {
                Ok(msg) => return msg.get(0).map(|f| f.to_vec()).unwrap_or_default(),
                Err(e) => {
                    // Try reconnection
                    error!("ZMQ recv error: {e}, reconnecting in 2s...");
                    sleep(Duration::from_secs(2)).await;
                    match Self::make_socket(&self.url).await {
                        Ok(socket) => {
                            info!("ZMQ reconnected to {}", self.url);
                            self.socket = socket;
                        }
                        Err(e) => error!("ZMQ reconnect failed: {e}"),
                    }
                }
            }
        }
    }
}
