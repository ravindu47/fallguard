use futures_util::{SinkExt, StreamExt};
use serde_json::json;
use tokio::time::{sleep, Duration};
use tokio_tungstenite::{connect_async, tungstenite::protocol::Message};
use url::Url;

#[tokio::main]
async fn main() {
    // --- UPDATED URL WITH SECURITY KEY ---
    let url = Url::parse("ws://127.0.0.1:8080/ws?key=admin123").unwrap();

    println!("🔌 Connecting to FallGuard Server...");

    // 1. Connect
    let (ws_stream, _) = connect_async(url.to_string())
        .await
        .expect("Failed to connect");

    let (mut write, _read) = ws_stream.split();

    println!("✅ Connected! Starting Data Stream...");

    // 2. Loop 1: Simulate Normal Walking
    for i in 1..=10 {
        let packet = json!({
            "x": 0.1,
            "y": 0.9,
            "z": 0.2,
            "timestamp": 170000000 + i
        });

        // Convert JSON to String, then into WebSocket Message
        write
            .send(Message::Text(packet.to_string().into()))
            .await
            .unwrap();

        println!("🚶 Sending Normal Data... ({}/10)", i);
        sleep(Duration::from_millis(500)).await;
    }

    // 3. Loop 2: Simulate Fall
    println!("⚠️ SIMULATING FALL EVENT!");
    let fall_packet = json!({
        "x": 24.5, // High G-Force (~2.5G * 9.8)
        "y": 2.0,
        "z": 1.0,
        "timestamp": 170000100
    });

    write
        .send(Message::Text(fall_packet.to_string().into()))
        .await
        .unwrap();

    // Wait a bit to let the user see the alert on the screen
    sleep(Duration::from_secs(5)).await;
    println!("🛑 Simulation Finished.");
}
