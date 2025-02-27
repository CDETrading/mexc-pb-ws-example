// #![recursion_limit = "128"]

use futures_util::{SinkExt, StreamExt};
use tokio_tungstenite::connect_async;
use tungstenite::Message;
use url::Url;
use bytes::BytesMut;
use prost::Message as ProstMessage;

include!(concat!(env!("OUT_DIR"), "/mexc_proto_build.rs"));

use mexc_proto::*;
// use mexc_proto::PublicAggreBookTickerV3Api;

#[tokio::main]
async fn main() {
    // Connect to the secure WebSocket endpoint.
    let ws_url = Url::parse("wss://wbs-api.mexc.com/ws").expect("Invalid URL");
    let (mut ws_stream, _) = connect_async(ws_url.as_str())
        .await
        .expect("Failed to connect to WebSocket");

    println!("Connected to WebSocket");

    // Prepare the subscription message.
    let subscribe_message = r#"{
        "method": "SUBSCRIPTION",
        "params": ["spot@public.aggre.bookTicker.v3.api.pb@10ms@BTCUSDT"]
    }"#;

    ws_stream
        .send(Message::Text(subscribe_message.to_string().into()))
        .await
        .expect("Failed to send subscription message");

    println!("Subscription message sent, listening for updates...");

    // Listen and process incoming messages.
    while let Some(msg) = ws_stream.next().await {
        match msg {
            Ok(Message::Binary(bin)) => {
                let mut buf = BytesMut::from(&bin[..]);
                match PushDataV3ApiWrapper::decode(&mut buf) {
                    Ok(response) => {
                        println!("Deserialized Response: {:?}", response);
                    },
                    Err(e) => {
                        eprintln!("Failed to decode protobuf message: {}", e);
                    }
                }
            },
            Ok(Message::Text(text)) => {
                println!("Received text message: {}", text);
            },
            Ok(Message::Ping(data)) => {
                ws_stream.send(Message::Pong(data))
                    .await
                    .expect("Failed to send pong");
            },
            Ok(Message::Close(frame)) => {
                println!("Connection closed: {:?}", frame);
                break;
            },
            Err(e) => {
                eprintln!("WebSocket error: {}", e);
                break;
            },
            _ => {}
        }
    }
    println!("WebSocket connection closed.");
}
