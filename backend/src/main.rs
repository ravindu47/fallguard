use actix_web::{web, App, HttpRequest, HttpResponse, HttpServer, Responder};
use actix_ws::Message;
use dotenv::dotenv;
use futures_util::StreamExt;
use sqlx::postgres::PgPoolOptions;
use sqlx::PgPool;
use std::env;
use tokio::sync::broadcast;

// Internal modules
mod logic;
mod model;
mod websockets;

use crate::model::{ClientCommand, FallLog};
use crate::websockets::ws_handler;
use chrono::Utc; // Added for timestamping
use rand;
use uuid::Uuid; // Added for generating IDs // For random IDs

/// **Global Application State**
///
/// This struct holds the resources that are shared across all connected clients.
/// - `db`: Connection pool to the PostgreSQL database for history logs.
/// - `tx`: The "Radio Station" (Broadcast Channel) used to send real-time sensor data to the frontend.
struct AppState {
    db: PgPool,
    tx: broadcast::Sender<String>,
}

/// **GET /api/history**
///
/// Retrieves the last 20 detected fall events from the database.
/// This is used by the frontend to populate the "Event Log" panel on startup.
async fn get_history(data: web::Data<AppState>) -> impl Responder {
    // Execute SQL query to fetch recent logs
    let result = sqlx::query_as!(
        FallLog,
        r#"
        SELECT 
            id,
            detected_at as "detected_at!", 
            severity as "severity!", 
            g_force_value as "g_force_value!", 
            is_false_alarm as "is_false_alarm!" 
        FROM events 
        ORDER BY detected_at DESC 
        LIMIT 20
        "#
    )
    .fetch_all(&data.db)
    .await;

    // Return JSON or Error
    match result {
        Ok(logs) => HttpResponse::Ok().json(logs),
        Err(e) => {
            eprintln!("❌ Database Error: {:?}", e);
            HttpResponse::InternalServerError().body("Error fetching logs")
        }
    }
}

/// **GET /api/fhir/history**
///
/// Retrieves fall events and converts them into clinical FHIR R4 "Observation" resources.
/// Code: LOINC 89020-2 (Fall risk assessment)
async fn get_fhir_history(data: web::Data<AppState>) -> impl Responder {
    // 1. Fetch from DB
    let result = sqlx::query_as!(
        FallLog,
        r#"
        SELECT 
            id,
            detected_at as "detected_at!", 
            severity as "severity!", 
            g_force_value as "g_force_value!", 
            is_false_alarm as "is_false_alarm!" 
        FROM events 
        ORDER BY detected_at DESC 
        LIMIT 20
        "#
    )
    .fetch_all(&data.db)
    .await;

    match result {
        Ok(logs) => {
            // 2. Transform to FHIR using model method
            use crate::model::FhirObservation;

            let fhir_bundle: Vec<FhirObservation> =
                logs.into_iter().map(|log| log.to_fhir()).collect();

            HttpResponse::Ok().json(fhir_bundle)
        }
        Err(e) => {
            eprintln!("❌ FHIR API Error: {:?}", e);
            HttpResponse::InternalServerError().body("Error generating FHIR data")
        }
    }
}

// ws_handler moved to websockets.rs

/// **Application Entry Point**
///
/// Initializes the Database, the Broadcast System, and starts the HTTP Server.
#[tokio::main]
async fn main() -> std::io::Result<()> {
    // 1. Load environment variables from .env file
    dotenv().ok();

    // 2. Database Setup (Connection Pool)
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .acquire_timeout(std::time::Duration::from_secs(3)) // Add this
        .connect(&database_url)
        .await
        .expect("Failed to connect to Postgres.");

    // 3. Broadcast System Setup
    // Capacity = 100 messages (Drop oldest if system gets overwhelmed)
    let (tx, _rx) = broadcast::channel(100);

    // 4. Initialize Global State
    let app_state = web::Data::new(AppState { db: pool, tx: tx });

    println!("🚀 SYSTEM HEALTH: Server started at http://0.0.0.0:8080");

    // 5. Start the HTTP Server
    HttpServer::new(move || {
        let cors = actix_cors::Cors::permissive();

        App::new()
            .wrap(cors) // Enable CORS
            .app_data(app_state.clone()) // Inject State
            .route("/api/history", web::get().to(get_history)) // REST API
            .route("/api/fhir/history", web::get().to(get_fhir_history)) // FHIR API
            .route("/ws", web::get().to(ws_handler)) // WebSocket API
    })
    .bind(("0.0.0.0", 8080))?
    .run()
    .await
}
