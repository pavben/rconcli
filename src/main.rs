mod servers;

use clap::Parser;
use std::path::PathBuf;
use futures::{
    select,
    SinkExt,
    StreamExt,
};
use tokio::io::{AsyncBufReadExt};
use tokio_tungstenite::{connect_async, tungstenite::protocol::Message};

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// YAML file containing the server list
    #[arg(short, long, default_value = "servers.yaml")]
    file: PathBuf,
    
    /// ID prefix to search for
    id_prefix: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    //println!("Loading servers from: {}", args.file.display());

    let servers = match servers::load_servers(&args.file).await {
        Ok(servers) => servers,
        Err(e) => {
            eprintln!("Error loading servers: {}", e);
            std::process::exit(1);
        }
    };
    //println!("Loaded {} servers", servers.len());
    
    match servers::find_server_by_prefix(&servers, &args.id_prefix) {
        Ok(server) => {
            connect_and_interact(&server).await;
        }
        Err(e) => {
            eprintln!("Server lookup error: {}", e);
            std::process::exit(1);
        }
    }

    Ok(())
}

async fn connect_and_interact(server: &servers::Server) {
    let address = format!("{}:{}", server.host, server.port);
    println!("Connecting to {address}...");
    let url = format!("ws://{}/{}", address, server.password);
    let (ws_stream, _) = connect_async(url).await.expect("Failed to connect");
    let (mut ws_write, ws_read) = ws_stream.split();

    println!("Connected");

/*
    let auth_message = r#"{"Identifier": 1, "Message": "status", "Name": "WebRcon" }"#;
    write.send(Message::Text(auth_message.to_string())).await.expect("Failed to send auth message");
*/

    let (stdin_write, stdin_read) = futures_channel::mpsc::unbounded();
    tokio::spawn(read_stdin(stdin_write));

    let mut ws_read = ws_read.fuse();
    let mut stdin_read = stdin_read.fuse();

    loop {
        select! {
            read_result = ws_read.next() => {
                match read_result {
                    Some(Ok(Message::Text(text))) => {
                        //println!("{text}");
                        match serde_json::from_str(&text) {
                            Ok(serde_json::Value::Object(obj)) => {
                                if let Some(serde_json::Value::String(message)) = obj.get("Message") {
                                    let message = message.replace("\\n", "\n");
                                    println!("{message}");

                                } else {
                                    println!("JSON message does not have a 'Message' field or it is not a String");
                                }
                                /*
                                match obj.get("Message") {
                                    Some(serde_json::Value::String(message)) => {
                                    }
                                    Some(_) => {
                                        println!("JSON 'Message' field is not a String");
                                    }
                                    None => {
                                    }
                                }
                                */
                            }
                            Ok(v) => {
                                eprintln!("Unhandled JSON from_str type: {v:?}");
                            }
                            Err(e) => {
                                eprintln!("Failed to parse JSON: {}", e);
                            }
                        }
                    }
                    Some(Ok(Message::Close(_))) => {
                        println!("Connection closed with a Close message.");
                        break;
                    }
                    None => {
                        println!("WebSocket stream ended");
                        break;
                    }
                    _ => {
                        println!("Catchall case for WebSocket");
                        break;
                    }
                }
            }
            msg = stdin_read.next() => {
                match msg {
                    Some(text) => {
                        let text = text.replace("\"", "\\\"");
                        let message_json = format!(r#"{{"Identifier": 1, "Message": "{text}", "Name": "WebRcon" }}"#);
                        ws_write.send(Message::Text(message_json)).await.expect("Failed to send message");
                    },
                    None => {
                        break;
                    }
                }
            }
        }
    }
}

async fn read_stdin(tx: futures_channel::mpsc::UnboundedSender<String>) {
    let stdin = tokio::io::stdin();
    let mut reader = tokio::io::BufReader::new(stdin);
    loop {
        let mut buf = String::new();
        let n = reader.read_line(&mut buf).await.expect("stdin reader.read_line fail");
        if n == 0 {
            println!("EOF");
            break; // EOF reached
        }
        buf = buf.trim_end().to_string(); // Remove trailing newline
        tx.unbounded_send(buf).expect("unbounded_send failed");
    }
}
