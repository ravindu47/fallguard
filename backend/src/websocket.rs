use crate::logic::{calculate_g_force, is_fall};
use crate::model::{AccelerometerData, ClientCommand};
use actix::{Actor, AsyncContext, Handler, Message, StreamHandler};
use actix_web_actors::ws;
use chrono::Utc;
use sqlx::PgPool;
use tokio::sync::broadcast;
use uuid::Uuid;

#[derive(Message)]
#[rtype(result = "()")]
struct BroadcastMessage(String);

pub struct SensorSession {
    pub db_pool: PgPool,
    pub broadcaster: broadcast::Sender<String>,
}

impl Actor for SensorSession {
    type Context = ws::WebsocketContext<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        // Subscribe to the broadcast channel to receive messages from other users/sensors
        let mut rx = self.broadcaster.subscribe();
        let addr = ctx.address();
        tokio::spawn(async move {
            while let Ok(msg) = rx.recv().await {
                addr.do_send(BroadcastMessage(msg));
            }
        });
    }
}

impl Handler<BroadcastMessage> for SensorSession {
    type Result = ();
    fn handle(&mut self, msg: BroadcastMessage, ctx: &mut Self::Context) {
        // Send the message to this specific client's browser
        ctx.text(msg.0);
    }
}

impl StreamHandler<Result<ws::Message, ws::ProtocolError>> for SensorSession {
    fn handle(&mut self, msg: Result<ws::Message, ws::ProtocolError>, _ctx: &mut Self::Context) {
        match msg {
            Ok(ws::Message::Ping(msg)) => _ctx.pong(&msg),
            Ok(ws::Message::Text(text)) => {
                // --- 1. CHECK FOR COMMANDS (From Dashboard Buttons) ---
                // We try to parse the message as a COMMAND first.
                if let Ok(cmd) = serde_json::from_str::<ClientCommand>(&text) {
                    // A. FALSE ALARM (Nurse clicked "Mark False Alarm")
                    if cmd.action == "CANCEL_ALERT" {
                        println!("🛡️ NURSE OVERRIDE: False Alarm.");

                        // Broadcast "CANCEL" to all screens (turns status Green)
                        let _ = self.broadcaster.send("CANCEL_ALERT".to_string());

                        // Log to DB
                        let pool = self.db_pool.clone();
                        let now = Utc::now();
                        actix::spawn(async move {
                            let _ = sqlx::query!(
                                "INSERT INTO events (id, detected_at, g_force_value, severity, is_false_alarm) VALUES ($1, $2, $3, $4, $5)",
                                Uuid::new_v4(), now, 0.0, "False Alarm", true
                            ).execute(&pool).await;
                        });
                    }
                    // B. REAL FALL (Nurse clicked "Dispatch Team")
                    else if cmd.action == "CONFIRM_FALL" {
                        println!("🚑 CRITICAL ACTION: Team Dispatched!");

                        // Broadcast "CONFIRMED" to all screens (turns status Blue)
                        let _ = self.broadcaster.send("CONFIRMED".to_string());

                        // Log to DB
                        let pool = self.db_pool.clone();
                        let now = Utc::now();
                        actix::spawn(async move {
                            let _ = sqlx::query!(
                                "INSERT INTO events (id, detected_at, g_force_value, severity, is_false_alarm) VALUES ($1, $2, $3, $4, $5)",
                                Uuid::new_v4(), now, 0.0, "Assistance Sent", false
                            ).execute(&pool).await;
                        });
                    }
                    // C. RESET SYSTEM (Nurse clicked "Reset to Stable")
                    else if cmd.action == "RESET_SYSTEM" {
                        println!("♻️ SYSTEM RESET: Patient is stable.");

                        // Broadcast "RESET_COMPLETE" to all screens (turns status Green, hides button)
                        let _ = self.broadcaster.send("RESET_COMPLETE".to_string());

                        // Log to DB
                        let pool = self.db_pool.clone();
                        let now = Utc::now();
                        actix::spawn(async move {
                            let _ = sqlx::query!(
                                "INSERT INTO events (id, detected_at, g_force_value, severity, is_false_alarm) VALUES ($1, $2, $3, $4, $5)",
                                Uuid::new_v4(), now, 0.0, "Resolved", false
                            ).execute(&pool).await;
                        });
                    }

                    return; // Stop processing, it was a command
                }

                // --- 2. CHECK FOR SENSOR DATA (From Pi/Simulator) ---
                // Forward raw data to graphs immediately
                let _ = self.broadcaster.send(text.to_string());

                if let Ok(data) = serde_json::from_str::<AccelerometerData>(&text) {
                    let g_force = calculate_g_force(data.x, data.y, data.z);

                    if is_fall(g_force) {
                        println!("⚠️ FALL DETECTED! G={:.2}", g_force);

                        let alert_msg = format!("ALERT: Fall Detected! G={:.2}", g_force);
                        let _ = self.broadcaster.send(alert_msg);

                        // Save "Critical" event to DB
                        let pool = self.db_pool.clone();
                        let now = Utc::now();
                        actix::spawn(async move {
                            let _ = sqlx::query!(
                                "INSERT INTO events (id, detected_at, g_force_value, severity, is_false_alarm) VALUES ($1, $2, $3, $4, $5)",
                                Uuid::new_v4(), now, g_force, "Critical", false
                            ).execute(&pool).await;
                        });
                    }
                }
            }
            _ => (),
        }
    }
}
