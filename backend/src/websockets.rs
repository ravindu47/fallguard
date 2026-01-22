use crate::logic::{DetectionEvent, FallDetector};
use crate::model::{ClientCommand, SensorData};
use crate::AppState;
use actix_web::{web, HttpRequest, Responder};
use actix_ws::Message;
use chrono::Utc;
use futures_util::StreamExt;
use rand;

pub async fn ws_handler(
    req: HttpRequest,
    stream: web::Payload,
    data: web::Data<AppState>,
) -> Result<impl Responder, actix_web::Error> {
    let (res, mut session, mut msg_stream) = actix_ws::handle(&req, stream)?;
    let mut rx = data.tx.subscribe();
    let tx = data.tx.clone();

    // Each connection has its own stateful detector
    let mut detector = FallDetector::new();

    actix_rt::spawn(async move {
        loop {
            tokio::select! {
                val = msg_stream.next() => {
                    match val {
                        Some(Ok(Message::Text(text))) => {
                            // 1. Try Command
                            if let Ok(cmd) = serde_json::from_str::<ClientCommand>(&text) {
                                let pool = data.db.clone();
                                let tx_clone = tx.clone();

                                if cmd.action == "CANCEL_ALERT" {
                                    let _ = tx_clone.send("CANCEL_ALERT".to_string());
                                    actix_rt::spawn(async move {
                                        let _ = sqlx::query!(
                                            "INSERT INTO events (id, detected_at, g_force_value, severity, is_false_alarm) VALUES ($1, $2, $3, $4, $5)",
                                            rand::random::<i32>(), Utc::now(), 0.0, "Refused", true
                                        ).execute(&pool).await;
                                    });
                                } else if cmd.action == "CONFIRM_FALL" {
                                    let _ = tx_clone.send("CONFIRMED".to_string());
                                    actix_rt::spawn(async move {
                                        let _ = sqlx::query!(
                                            "INSERT INTO events (id, detected_at, g_force_value, severity, is_false_alarm) VALUES ($1, $2, $3, $4, $5)",
                                            rand::random::<i32>(), Utc::now(), 0.0, "Assistance Sent", false
                                        ).execute(&pool).await;
                                    });
                                } else if cmd.action == "RESET_SYSTEM" {
                                    let _ = tx_clone.send("RESET_COMPLETE".to_string());
                                    actix_rt::spawn(async move {
                                        let _ = sqlx::query!(
                                            "INSERT INTO events (id, detected_at, g_force_value, severity, is_false_alarm) VALUES ($1, $2, $3, $4, $5)",
                                            rand::random::<i32>(), Utc::now(), 0.0, "Resolved", false
                                        ).execute(&pool).await;
                                    });
                                }
                            }
                            // 2. Try Sensor Data
                            else if let Ok(mut sensor_data) = serde_json::from_str::<SensorData>(&text) {
                                // Feed into Logic
                                if let Some(event) = detector.process(sensor_data.clone()) {
                                    match event {
                                        DetectionEvent::Validating => {
                                            println!("🟡 State: VALIDATING (Buffer Started)");
                                            let _ = tx.send("VALIDATING".to_string());
                                        }
                                        DetectionEvent::CriticalFall { g_force } => {
                                            println!("🔴 State: CRITICAL FALL CONFIRMED! (G: {:.2})", g_force);
                                            // Send alert with G-Force
                                            let alert_msg = serde_json::json!({
                                                "type": "CRITICAL_FALL",
                                                "g_force": g_force
                                            }).to_string();
                                            let _ = tx.send(alert_msg);
                                        }
                                        DetectionEvent::NearMiss => {
                                             println!("⚪ State: NEAR MISS (Movement Detected)");
                                             let _ = tx.send("NEAR_MISS".to_string());
                                        }
                                    }
                                }

                                // Broadcast raw data for charts
                                let _ = tx.send(text.to_string());
                            } else {
                                // Debug: Print if JSON is invalid
                                println!("⚠️ Received Unknown format: {}", text);
                            }
                        }
                        Some(Ok(Message::Close(_))) => break,
                        None => break,
                        _ => {}
                    }
                }
                val = rx.recv() => {
                    if let Ok(msg) = val {
                        let _ = session.text(msg).await;
                    }
                }
            }
        }
    });

    Ok(res)
}
