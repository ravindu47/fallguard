use serde::{Deserialize, Serialize};
use sqlx::FromRow;

// 1. INPUT: Sensor Data (Now with Real Temp support)
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct SensorData {
    pub x: f64,
    pub y: f64,
    pub z: f64,
    #[serde(rename = "t")]
    pub timestamp: f64,
    #[serde(default)]
    pub wifi: i32,
    #[serde(default)]
    pub temp: f64, // Real temperature from MPU6050
}

// 2. OUTPUT: Enriched Data (Live Stream)
#[derive(Debug, Serialize, Clone)]
pub struct EnrichedData {
    pub raw: SensorData,
    pub g_force: f64,
    pub alert: bool,
    pub diagnosis: String,
}

// 3. DATABASE: Fall Log
#[derive(Debug, Serialize, FromRow)]
pub struct FallLog {
    pub id: i32,
    pub detected_at: chrono::DateTime<chrono::Utc>,
    pub severity: String,
    pub g_force_value: f64,
    pub is_false_alarm: bool,
}

// 4. STATS: Risk Report (Upgrade 3)
// "How many Forward falls vs Side falls?"
#[derive(Debug, Serialize, FromRow)]
pub struct RiskReport {
    pub severity: String,   // e.g., "Forward Fall"
    pub count: Option<i64>, // Postgres counts can be null, Option handles that
}

// 5. COMPLIANCE: FHIR Observation (Upgrade 1)
// The standard format hospitals use (HL7 FHIR R4)
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FhirObservation {
    pub resource_type: String,
    pub id: String,
    pub status: String,
    pub code: serde_json::Value,
    pub subject: serde_json::Value,
    pub value_string: String,
    pub effective_date_time: String,
}

// 6. INPUT: Client Command (Frontend Buttons)
#[derive(Debug, Deserialize)]
pub struct ClientCommand {
    pub action: String,
}

impl FallLog {
    pub fn to_fhir(&self) -> FhirObservation {
        use serde_json::json;

        // Map internal "Severity" to FHIR "ValueString" & "Status"
        let (status, value) = match self.severity.as_str() {
            "Critical" => ("final", "High Risk - Fall Detected"),
            "False Alarm" => ("entered-in-error", "Low Risk - False Alarm"),
            "Assistance Sent" => ("final", "Assessment in Progress"),
            "Resolved" => ("final", "Patient Stable"),
            "Near Miss" => ("final", "Near Miss - Movement Detected"), // Matches new status
            _ => ("preliminary", "Unknown Status"),
        };

        FhirObservation {
            resource_type: "Observation".to_string(),
            id: self.id.to_string(),
            status: status.to_string(),
            code: json!({
                "coding": [{
                    "system": "http://loinc.org",
                    "code": "89020-2", // LOINC Fall risk assessment
                    "display": "Fall risk assessment"
                }],
                "text": "Fall Detection Event"
            }),
            subject: json!({
                "reference": "Patient/ICU-04",
                "display": "John Doe"
            }),
            value_string: value.to_string(),
            effective_date_time: self.detected_at.to_rfc3339(),
        }
    }
}
