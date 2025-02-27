use futures_util::{SinkExt, StreamExt};
use tokio_tungstenite::connect_async;
use tungstenite::Message;
use url::Url;
use bytes::BytesMut;
use prost::Message as ProstMessage;

pub mod mexc {
    include!(concat!(env!("OUT_DIR"), "/mexc.rs"));
}
// Assume the generated protobuf code is in the `mexc` module.
use mexc::BatchBookTickerResponse;

#[tokio::main]
async fn main() {
    // Connect to the secure WebSocket endpoint.
    let ws_url = Url::parse("ws://wbs-api.mexc.com/ws").expect("Invalid URL");
    let (mut ws_stream, _) = connect_async(ws_url.as_str())
        .await
        .expect("Failed to connect to WebSocket");

    println!("Connected to WebSocket");

    // Prepare the subscription message for the batch bookticker channel.
    // Note: The channel string must exactly match what the server expects.
    let subscribe_message = r#"{
        "method": "SUBSCRIPTION",
        "params": ["spot@public.bookTicker.batch.v3.api.pb@BTCUSDT"]
    }"#;

    ws_stream
        .send(Message::Text(subscribe_message.to_string().into()))
        .await
        .expect("Failed to send subscription message");

    println!("Subscription message sent, listening for updates...");

    // Listen for incoming messages.
    while let Some(msg) = ws_stream.next().await {
        match msg {
            Ok(Message::Binary(bin)) => {
                println!("Received binary message: {:?}", bin);
                // Use BytesMut for prost decoding.
                let mut buf = BytesMut::from(&bin[..]);
                match BatchBookTickerResponse::decode(&mut buf) {
                    Ok(response) => {
                        println!("Deserialized Response: {:?}", response);
                        // Iterate through each ticker item.
                        for item in response.public_book_ticker_batch.unwrap().items {
                            println!(
                                "Symbol: {}, Bid: {} (qty: {}), Ask: {} (qty: {})",
                                response.symbol,
                                item.bid_price,
                                item.bid_quantity,
                                item.ask_price,
                                item.ask_quantity
                            );
                        }
                    }
                    Err(e) => {
                        eprintln!("Failed to decode protobuf message: {}", e);
                    }
                }
            }
            Ok(Message::Text(text)) => {
                println!("Received text message: {}", text);
            }
            Ok(Message::Ping(data)) => {
                // Respond to keep the connection alive.
                ws_stream
                    .send(Message::Pong(data))
                    .await
                    .expect("Failed to send pong");
            }
            Ok(Message::Pong(data)) => {
                println!("Received pong: {:?}", data);
            }
            Ok(Message::Close(frame)) => {
                println!("Connection closed: {:?}", frame);
                break;
            }
            Err(e) => {
                eprintln!("WebSocket error: {}", e);
                break;
            }
            _ => {}
        }
    }

    println!("WebSocket connection closed.");
}
