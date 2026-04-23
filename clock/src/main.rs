use clap::{Parser, ValueEnum};
use std::time::{Duration, SystemTime};
use tokio::{
    io::{AsyncBufReadExt, BufReader},
    time::sleep,
};
use voltaalert_core::zmq::{ZeromqClient, ZeromqConfig};
use zeromq::{PubSocket, Socket, SocketSend, ZmqMessage};

#[derive(ValueEnum, Clone)]
enum Mode {
    /// Sends the current Unix timestamp every cycle
    Period,
    /// Sends each line typed by the user (ignores --duration)
    Manual,
    /// Reads a file line by line and loops back when EOF
    File,
}

#[derive(Parser)]
#[clap(author, version, about, long_about = None)]
struct Cli {
    #[clap(short, long, default_value = "9000")]
    port: u16,

    #[clap(short, long, default_value = "127.0.0.1")]
    addr: String,

    #[clap(long, default_value = "tcp")]
    protocol: String,

    #[clap(short, long)]
    subscribe: bool,

    #[clap(short, long, value_enum, default_value = "period")]
    mode: Mode,

    /// File path (required for --mode file)
    #[clap(short, long)]
    file: Option<String>,
}

async fn subscriber_process(config: ZeromqConfig) {
    let url = config.url();
    let mut client = ZeromqClient::connect(config).await.expect("Failed to connect");
    println!("Subscribed on: {}", url);
    loop {
        let msg = client.recv().await;
        println!("Received: {}", String::from_utf8_lossy(&msg));
    }
}

async fn publisher_process(config: ZeromqConfig, mode: Mode, cycle: Duration, file: Option<String>) {
    let addr = config.url();

    let mut socket = PubSocket::new();
    socket.bind(&addr).await.expect("Failed to bind");
    let mode_label = match mode {
        Mode::Period => "period",
        Mode::Manual => "manual",
        Mode::File   => "file",
    };
    println!("Publishing on: {} (mode: {})", addr, mode_label);

    match mode {
        Mode::Period => loop {
            let ts = SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .expect("Time went backwards")
                .as_secs()
                .to_string();
            socket.send(ZmqMessage::from(ts.as_str())).await.expect("Failed to send");
            sleep(cycle).await;
        },

        Mode::Manual => {
            let mut lines = BufReader::new(tokio::io::stdin()).lines();
            while let Ok(Some(line)) = lines.next_line().await {
                socket.send(ZmqMessage::from(line.as_str())).await.expect("Failed to send");
            }
        },

        Mode::File => {
            let path = file.expect("--file <PATH> is required for --mode file");
            loop {
                let f = tokio::fs::File::open(&path).await.expect("Failed to open file");
                let mut lines = BufReader::new(f).lines();
                while let Ok(Some(line)) = lines.next_line().await {
                    socket.send(ZmqMessage::from(line.as_str())).await.expect("Failed to send");
                    sleep(cycle).await;
                }
            }
        },
    }
}

#[tokio::main]
async fn main() {
    let args = Cli::parse();

    let config = ZeromqConfig {
        port: args.port,
        addr: args.addr,
        protocol: args.protocol,
    };

    if args.subscribe {
        subscriber_process(config).await;
    } else {
        publisher_process(config, args.mode, Duration::from_secs(1), args.file).await;
    }
}
