import smbus2 as smbus
import math
import time
import json
import threading
import websocket
import ssl
import os

# [IoT Sensor Logic]
# Author: Vidanapathiranage Ruwan Chamara (22403812)
# Driver for MPU6050 Accelerometer
# Handles I2C communication and raw vector normalization.

# --- CONFIGURATION ---
# TODO: REPLACE THIS URL WITH YOUR CODESPACE URL
# Example: "wss://glowing-guide-49w...-8080.app.github.dev/ws"
SERVER_URL = "wss://YOUR-CODESPACE-NAME-8080.app.github.dev/ws"

# MPU6050 Registers
PWR_MGMT_1 = 0x6B
SMPLRT_DIV = 0x19
CONFIG = 0x1A
GYRO_CONFIG = 0x1B
INT_ENABLE = 0x38
ACCEL_XOUT_H = 0x3B
ACCEL_YOUT_H = 0x3D
ACCEL_ZOUT_H = 0x3F
GYRO_XOUT_H = 0x43
GYRO_YOUT_H = 0x45
GYRO_ZOUT_H = 0x47

# Initialize I2C
bus = smbus.SMBus(1)
DEVICE_ADDRESS = 0x68

def MPU_Init():
    try:
        # Write to sample rate register
        bus.write_byte_data(DEVICE_ADDRESS, SMPLRT_DIV, 7)
        # Write to power management register
        bus.write_byte_data(DEVICE_ADDRESS, PWR_MGMT_1, 1)
        # Write to Configuration register
        bus.write_byte_data(DEVICE_ADDRESS, CONFIG, 0)
        # Write to Gyro configuration register
        bus.write_byte_data(DEVICE_ADDRESS, GYRO_CONFIG, 24)
        # Write to interrupt enable register
        bus.write_byte_data(DEVICE_ADDRESS, INT_ENABLE, 1)
        print("✅ MPU6050 Initialized Successfully")
    except Exception as e:
        print(f"❌ Failed to init MPU6050: {e}")

def read_raw_data(addr):
    # Reads 16-bit values (High byte + Low byte)
    try:
        high = bus.read_byte_data(DEVICE_ADDRESS, addr)
        low = bus.read_byte_data(DEVICE_ADDRESS, addr + 1)
        value = ((high << 8) | low)
        if (value > 32768):
            value = value - 65536
        return value
    except:
        return 0

def on_message(ws, message):
    print(f"📩 Received from Server: {message}")

def on_error(ws, error):
    print(f"⚠️ Error: {error}")

def on_close(ws, close_status_code, close_msg):
    print("🔌 Disconnected from Server")

def on_open(ws):
    print("✅ Connected to Codespaces!")
    
    # Start the sensor loop in a separate thread so it doesn't block
    def run_sensor():
        print("🚀 Starting Telemetry Stream...")
        while True:
            try:
                # Read Accelerometer raw value
                acc_x = read_raw_data(ACCEL_XOUT_H)
                acc_y = read_raw_data(ACCEL_YOUT_H)
                acc_z = read_raw_data(ACCEL_ZOUT_H)
                
                # Convert to G-Force (approximate sensitivity for +/- 2g)
                Ax = acc_x / 16384.0
                Ay = acc_y / 16384.0
                Az = acc_z / 16384.0

                # Read Temperature (Just for extra data)
                temp_raw = read_raw_data(0x41)
                temp = (temp_raw / 340.0) + 36.53

                # Prepare Payload
                payload = {
                    "x": round(Ax, 2),
                    "y": round(Ay, 2),
                    "z": round(Az, 2),
                    "temp": round(temp, 1),
                    "wifi": 100, # Mock WiFi signal strength
                    "t": int(time.time())
                }

                # Send to Server
                ws.send(json.dumps(payload))
                
                # 20Hz Update Rate (0.05s delay)
                time.sleep(0.05)

            except Exception as e:
                print(f"Sensor Error: {e}")
                time.sleep(1)

    # Run the loop in background
    threading.Thread(target=run_sensor, daemon=True).start()

if __name__ == "__main__":
    MPU_Init()
    
    # Enable Trace for debugging
    # websocket.enableTrace(True)
    
    while True:
        try:
            print(f"🔗 Connecting to {SERVER_URL}...")
            ws = websocket.WebSocketApp(SERVER_URL,
                                        on_open=on_open,
                                        on_message=on_message,
                                        on_error=on_error,
                                        on_close=on_close)
            
            # Run forever with SSL verification disabled (needed for some dev environments)
            ws.run_forever(sslopt={"cert_reqs": ssl.CERT_NONE})
            
            print("Reconnecting in 3 seconds...")
            time.sleep(3)
            
        except KeyboardInterrupt:
            print("Exiting...")
            break
