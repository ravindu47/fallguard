
# 🛡️ FallGuard: IoT Critical Fall Detection System

[![Status](https://img.shields.io/badge/Status-Active_Development-success?style=flat-square)]()
[![Backend](https://img.shields.io/badge/Backend-Rust_%2F_Actix-orange?style=flat-square)]()
[![Edge](https://img.shields.io/badge/Edge_Compute-Python_%2F_Raspberry_Pi-blue?style=flat-square)]()
[![Standard](https://img.shields.io/badge/Interoperability-HL7_FHIR_R4-firebrick?style=flat-square)]()
[![License](https://img.shields.io/badge/License-MIT-green?style=flat-square)]()

**FallGuard** is a high-performance, privacy-centric IoT ecosystem engineered to detect critical fall events in real-time. By bridging edge telemetry with hospital-grade interoperability standards (**HL7 FHIR R4**), FallGuard provides a seamless integration path between home monitoring devices and Electronic Health Record (EHR) systems.

---

## 📑 Table of Contents
- [Overview](#-overview)
- [System Architecture](#-system-architecture)
- [Technical Implementation](#-technical-implementation)
- [Getting Started](#-getting-started)
- [API Reference](#-api-reference)
- [Project Team](#-project-team)

---

## 📖 Overview

Falls are a leading cause of injury among high-risk demographics. Traditional solutions often suffer from high latency or lack clinical integration. FallGuard solves this by combining **Rust's memory safety and concurrency** for backend processing with a lightweight **Python edge client**, ensuring millisecond-level detection and immediate clinical logging.

### Key Capabilities
* **Real-Time Telemetry:** Stream synchronized 3-axis accelerometer data (X, Y, Z, G-Force) at 20Hz via Secure WebSockets (`wss://`).
* **Clinical Interoperability:** Automated serialization of events into **FHIR R4 Observation** resources for EHR compatibility.
* **Intelligent False Alarm Mitigation:** Multi-stage state machine filters "Near Miss" events versus critical impacts.
* **Interactive ICU Command Center:** Live D3.js data visualization for remote patient monitoring.

---

## 🏗 System Architecture

The solution follows a distributed event-driven architecture:

| Component | Technology Stack | Description |
| :--- | :--- | :--- |
| **Edge Node** | Raspberry Pi Zero 2 W + Python | Interfaces with MPU6050 sensors via I2C; performs raw vector normalization and transmission. |
| **Core Backend** | Rust (Actix-Web) | Handles high-concurrency WebSocket connections, manages state, and processes detection logic. |
| **Persistence** | PostgreSQL (Docker) | Relational storage for event logs, telemetry history, and FHIR resources. |
| **Dashboard** | HTML5 / D3.js | Low-latency frontend for real-time visualization and alert management. |

---

## 📐 Technical Implementation

### 1. Vector Magnitude & Impact Analysis
To ensure orientation-independent detection, the system calculates the Euclidean Norm of the acceleration vector in real-time.

$$G = \frac{\sqrt{a_x^2 + a_y^2 + a_z^2}}{g}$$

* **$a_{x,y,z}$**: Raw acceleration inputs ($m/s^2$)
* **$g$**: Standard gravity constant ($9.81 m/s^2$)
* **Threshold:** Events where $G > 2.5$ trigger the state validation engine.

### 2. Detection State Machine
To minimize alert fatigue, the backend implements a strict finite state machine (FSM):

1.  **MONITORING:** Idle state; continuous telemetry ingestion.
2.  **VALIDATING:** Threshold breached. System enters a 2-second "recovery window" to analyze post-impact movement.
3.  **CRITICAL_FALL:** No recovery movement detected. Severity escalated; immediate alert dispatched to UI.
4.  **RESOLVED:** Manual intervention or "False Alarm" signal received from the dashboard.

---

## 🚀 Getting Started

### Prerequisites
* **Docker Desktop** (Containerization)
* **Rust Toolchain** (Latest Stable)
* **Python 3.9+** (Edge Client)

### 🖥️ Backend Setup (Host)

1.  **Clone the Repository**
    ```bash
    git clone [https://github.com/yourusername/fallguard.git](https://github.com/yourusername/fallguard.git)
    cd fallguard/backend
    ```

2.  **Initialize Infrastructure**
    Launch the PostgreSQL container using Docker Compose:
    ```bash
    docker compose up -d
    ```

3.  **Apply Database Migrations**
    Initialize the schema using `sqlx`:
    ```bash
    export DATABASE_URL=postgres://postgres:password@localhost:5432/fallguard
    sqlx migrate run
    ```

4.  **Launch the Server**
    ```bash
    cargo run --bin backend
    ```
    *> Server listening on `0.0.0.0:8080`*

### 🥧 Edge Node Setup (Raspberry Pi)

1.  **Hardware Configuration (MPU6050)**
    * **VCC:** Pin 1 (3.3V) | **GND:** Pin 6
    * **SDA:** Pin 3 (GPIO 2) | **SCL:** Pin 5 (GPIO 3)

2.  **Environment Setup**
    ```bash
    # Install system dependencies
    sudo apt-get install -y i2c-tools python3-smbus

    # Setup Python Virtual Environment
    python3 -m venv venv
    source venv/bin/activate
    pip install websocket-client smbus2
    ```

3.  **Deploy Agent**
    Update `fallguard.py` with your backend endpoint and start the service:
    ```bash
    python fallguard.py
    ```

---

## 🩺 API Reference

### WebSocket Endpoint: `/ws`
**Protocol:** `ws://` or `wss://`
* **Ingress (Sensor → Server):**
    ```json
    {
      "x": 0.12, "y": -0.05, "z": 9.81,
      "t": 1705928355,
      "wifi": 98, "temp": 36.5
    }
    ```
* **Egress (Server → Client):** `CRITICAL_FALL`, `VALIDATING`, `NEAR_MISS`, `CONFIRMED`

### Clinical API: `/api/fhir/history`
Returns clinical observations compliant with **HL7 FHIR Release 4**.

**Response Example:**
```json
[
  {
    "resourceType": "Observation",
    "status": "final",
    "code": {
      "coding": [{ "system": "[http://loinc.org](http://loinc.org)", "code": "89020-2", "display": "Fall risk" }]
    },
    "valueString": "Patient Stable",
    "effectiveDateTime": "2026-01-22T02:30:12+00:00"
  }
]

```

---

## 👥 Project Team

| Name / ID | Role | Key Contributions |
| :--- | :--- | :--- |
| **Chameesha Ravindu Wijewickrama Kankanamalage**<br>*(22404656)* | **Lead Architect & Frontend Engineer** | • **System Architecture & Leadership:** Designed the overall project roadmap and solved critical integration problems between the hardware, backend, and frontend layers.<br>• **Clinical Dashboard:** Developed the real-time ICU Command Center using **D3.js**, enabling high-frequency visualization of G-force telemetry.<br>• **Integration Logic:** Served as the bridge between raw hardware data and clinical metrics, translating LSB sensor values into human-readable health data.<br>• **Bi-Directional Control:** Engineered the client-side control logic allowing operators to send "Dispatch" or "Reset" commands back to the Rust core. |
| **Vidanapathiranage Ruwan Chamara**<br>*(22403812)* | **Edge & Embedded Systems Engineer** | • **Hardware Integration:** Handled physical wiring and I2C protocol configuration for the MPU-6050 and Raspberry Pi Zero 2 W.<br>• **Embedded Software:** Wrote the Python ingestion script (`fallguard.py`) to serialize raw sensor data and transmit it via Secure WebSockets.<br>• **OS Management:** Configured the "headless" Linux environment, ensuring autonomous boot and network connectivity without peripherals. |
| **Sajana Ransika Abeyrathna**<br>*(22404659)* | **Backend & Infrastructure Engineer** | • **Core Server:** Built the high-concurrency Rust (Actix-Web) backend with the Actor model for real-time streaming.<br>• **Infrastructure:** Managed PostgreSQL via Docker and utilized SQLx for type-safe database interactions.<br>• **Compliance:** Implemented **HL7 FHIR R4** standards for medical interoperability and mapped internal models to LOINC/SNOMED CT codes.<br>• **QA:** Wrote extensive unit tests to mathematically verify the physics logic of the fall detection algorithm. |

> This project is for educational and research purposes.

```

```
