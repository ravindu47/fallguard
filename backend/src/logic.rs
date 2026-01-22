use crate::model::SensorData;
use std::collections::VecDeque;

const IMPACT_THRESHOLD_G: f64 = 1.6;
const STILLNESS_THRESHOLD_VARIANCE: f64 = 3.5; // Relaxed to allow post-fall movement
const BUFFER_DURATION_MS: i64 = 2000; // 2 seconds

#[derive(Debug, Clone)]
pub enum DetectionEvent {
    Validating,
    CriticalFall { g_force: f64 },
    NearMiss,
}

enum State {
    Monitoring,
    PreAlert {
        start_time: i64,
        buffer: VecDeque<SensorData>,
        max_g: f64,
    },
}

pub struct FallDetector {
    state: State,
}

impl FallDetector {
    pub fn new() -> Self {
        Self {
            state: State::Monitoring,
        }
    }

    pub fn process(&mut self, data: SensorData) -> Option<DetectionEvent> {
        // Calculate G-Force
        let g_force = (data.x.powi(2) + data.y.powi(2) + data.z.powi(2)).sqrt() / 9.8;

        if g_force > 1.2 {
            println!(
                "📊 G-Force: {:.2} (Threshold: {:.2})",
                g_force, IMPACT_THRESHOLD_G
            );
        }

        let now = chrono::Utc::now().timestamp_millis();

        match &mut self.state {
            State::Monitoring => {
                if g_force > IMPACT_THRESHOLD_G {
                    // Transition to PreAlert
                    self.state = State::PreAlert {
                        start_time: now,
                        buffer: VecDeque::new(),
                        max_g: g_force,
                    };
                    return Some(DetectionEvent::Validating);
                }
            }
            State::PreAlert {
                start_time,
                buffer,
                max_g,
            } => {
                // Keep track of max impact during buffer
                if g_force > *max_g {
                    *max_g = g_force;
                }

                buffer.push_back(data);

                // Check time duration
                if now - *start_time >= BUFFER_DURATION_MS {
                    // 2.0s passed. Analyze buffer for stillness.
                    let variance = calculate_variance(buffer);

                    let result = if variance < STILLNESS_THRESHOLD_VARIANCE {
                        Some(DetectionEvent::CriticalFall { g_force: *max_g })
                    } else {
                        Some(DetectionEvent::NearMiss)
                    };

                    // Reset to Monitoring
                    self.state = State::Monitoring;
                    return result;
                }
            }
        }
        None
    }
}

fn calculate_variance(buffer: &VecDeque<SensorData>) -> f64 {
    if buffer.is_empty() {
        return 0.0;
    }

    let count = buffer.len() as f64;
    let sum_x: f64 = buffer.iter().map(|d| d.x).sum();
    let sum_y: f64 = buffer.iter().map(|d| d.y).sum();
    let sum_z: f64 = buffer.iter().map(|d| d.z).sum();

    let mean_x = sum_x / count;
    let mean_y = sum_y / count;
    let mean_z = sum_z / count;

    let var_x: f64 = buffer.iter().map(|d| (d.x - mean_x).powi(2)).sum();
    let var_y: f64 = buffer.iter().map(|d| (d.y - mean_y).powi(2)).sum();
    let var_z: f64 = buffer.iter().map(|d| (d.z - mean_z).powi(2)).sum();

    // Total variance magnitude
    (var_x + var_y + var_z) / count
}
