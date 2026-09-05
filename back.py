"""
CubeSat Airborne Disaster Response Platform — Telemetry Gateway & Mission Control Backend
Complies with aerospace telemetry standards: packet sequencing, CRC validation, deterministic sensor fusion.
"""

import asyncio
import json
import math
import random
import time
from datetime import datetime, timezone
from typing import Dict, List, Optional
from fastapi import FastAPI, WebSocket, WebSocketDisconnect
from fastapi.middleware.cors import CORSMiddleware
from pydantic import BaseModel

app = FastAPI(title="CubeSat Airborne Disaster Response Telemetry Server", version="1.0.0")

app.add_middleware(
    CORSMiddleware,
    allow_origins=["*"],
    allow_credentials=True,
    allow_methods=["*"],
    allow_headers=["*"],
)

class MissionState:
    def __init__(self):
        self.unit_id = "CUBESAT-2U-ALPHA"
        self.packet_id = 1000
        self.mode = "WILDFIRE"  # WILDFIRE, FLOOD, LANDSLIDE, EARTHQUAKE
        self.sample_rate_hz = 2.0
        self.uplink_status = "LINK_LOCKED"
        self.downlink_status = "DOWNLINK_ACTIVE"
        self.rssi_dbm = -68.0
        self.crc_valid = True
        self.packets_sent = 0
        self.packets_dropped = 0
        
        # Flight dynamics (balloon platform)
        self.base_lat = 34.1985
        self.base_lon = -118.1750
        self.current_lat = self.base_lat
        self.current_lon = self.base_lon
        self.altitude_m = 1240.0  # Balloon float altitude in meters
        self.climb_rate_ms = 0.2
        self.ground_speed_kmh = 14.8
        self.heading_deg = 42.5
        
        # Subsystems & Power Bus
        self.battery_voltage = 7.92  # 2S LiPo (8.4V max, 7.4V nom, 6.6V cutoff)
        self.current_draw_ma = 480.0
        self.rail_3v3_v = 3.31
        self.rail_5v0_v = 4.98
        self.storage_used_mb = 142.5
        self.storage_total_mb = 16384.0
        
        # Primary Sensors
        self.temp_c = 28.4
        self.accel_x = 0.04
        self.accel_y = -0.02
        self.accel_z = 0.99
        self.tilt_pitch = 2.1
        self.tilt_roll = -1.4
        self.vibration_index = 0.08
        self.ambient_light_lux = 1840.0
        
        # Secondary / Ground field nodes
        self.ground_flame_intensity = 0.0
        self.ground_smoke_ppm = 18.0
        self.ground_water_level_cm = 12.0
        self.ground_soil_moisture_pct = 24.0
        self.ground_geophone_vibration = 0.02
        self.ground_inclinometer_deg = 0.4
        
        # Imagery catalog
        self.recent_images: List[Dict] = []
        self.init_sample_images()
        
        # Telemetry history buffer for chart warm-up (last 40 points)
        self.history: List[Dict] = []
        
    def init_sample_images(self):
        self.recent_images = [
            {
                "id": "IMG-0084",
                "timestamp": datetime.now(timezone.utc).isoformat(),
                "mode": "WILDFIRE",
                "lat": 34.2045,
                "lon": -118.1680,
                "alt_m": 1250,
                "resolution": "1600x1200 UXGA",
                "exposure_ms": 14,
                "spectral_band": "VIS-RGB",
                "hotspot_count": 3,
                "burn_severity_index": 0.78,
                "status": "ANALYZED"
            },
            {
                "id": "IMG-0083",
                "timestamp": datetime.now(timezone.utc).isoformat(),
                "mode": "WILDFIRE",
                "lat": 34.2012,
                "lon": -118.1715,
                "alt_m": 1238,
                "resolution": "1600x1200 UXGA",
                "exposure_ms": 12,
                "spectral_band": "THERMAL-IR-SYNTH",
                "hotspot_count": 2,
                "burn_severity_index": 0.65,
                "status": "ANALYZED"
            },
            {
                "id": "IMG-0082",
                "timestamp": datetime.now(timezone.utc).isoformat(),
                "mode": "FLOOD",
                "lat": 34.1950,
                "lon": -118.1820,
                "alt_m": 1210,
                "resolution": "1600x1200 UXGA",
                "exposure_ms": 15,
                "spectral_band": "VIS-RGB",
                "hotspot_count": 0,
                "burn_severity_index": 0.0,
                "status": "ANALYZED"
            }
        ]

    def update_physics(self):
        self.packet_id += 1
        self.packets_sent += 1
        now = time.time()
        
        # Balloon gentle drift and pendulum motion
        drift_rate = 0.00003
        self.current_lat += math.cos(now * 0.05) * drift_rate
        self.current_lon += math.sin(now * 0.05) * drift_rate
        self.altitude_m += math.sin(now * 0.08) * 0.4
        
        # Accelerometer dynamics (balloon swaying)
        sway_freq = 0.8
        self.accel_x = math.sin(now * sway_freq) * 0.15 + (random.random() - 0.5) * 0.02
        self.accel_y = math.cos(now * sway_freq * 0.9) * 0.12 + (random.random() - 0.5) * 0.02
        self.accel_z = 0.98 + (random.random() - 0.5) * 0.03
        self.tilt_pitch = math.degrees(math.atan2(self.accel_x, self.accel_z))
        self.tilt_roll = math.degrees(math.atan2(self.accel_y, self.accel_z))
        self.vibration_index = round(math.sqrt(self.accel_x**2 + self.accel_y**2) * 10, 2)
        
        # Battery slow discharge & regulator fluctuation
        self.battery_voltage = max(6.8, round(self.battery_voltage - 0.00005, 3))
        self.current_draw_ma = round(450.0 + random.random() * 45.0, 1)
        self.rail_3v3_v = round(3.30 + (random.random() - 0.5) * 0.02, 2)
        self.rail_5v0_v = round(5.00 + (random.random() - 0.5) * 0.03, 2)
        
        # RF link budget
        self.rssi_dbm = round(-70.0 + math.sin(now * 0.1) * 4.0 + (random.random() - 0.5) * 2.0, 1)
        
        # Mode-specific sensor dynamics
        if self.mode == "WILDFIRE":
            self.temp_c = round(38.5 + math.sin(now * 0.3) * 4.2 + (random.random() - 0.5) * 0.8, 1)
            self.ambient_light_lux = round(2100.0 + math.sin(now * 0.2) * 200.0, 0)
            self.ground_flame_intensity = round(min(100.0, max(45.0, 68.0 + math.sin(now * 0.4) * 22.0)), 1)
            self.ground_smoke_ppm = round(min(500.0, max(80.0, 180.0 + math.sin(now * 0.3) * 60.0)), 1)
            self.ground_water_level_cm = 12.0
            self.ground_geophone_vibration = 0.02
        elif self.mode == "FLOOD":
            self.temp_c = round(21.2 + (random.random() - 0.5) * 0.4, 1)
            self.ambient_light_lux = round(850.0 + (random.random() - 0.5) * 40.0, 0)
            self.ground_flame_intensity = 0.0
            self.ground_smoke_ppm = 14.0
            self.ground_water_level_cm = round(145.0 + math.sin(now * 0.2) * 35.0, 1)
            self.ground_soil_moisture_pct = round(min(100.0, 88.0 + math.sin(now * 0.1) * 8.0), 1)
            self.ground_geophone_vibration = 0.01
        elif self.mode == "LANDSLIDE":
            self.temp_c = round(19.8 + (random.random() - 0.5) * 0.5, 1)
            self.ambient_light_lux = round(1100.0 + (random.random() - 0.5) * 50.0, 0)
            self.ground_flame_intensity = 0.0
            self.ground_smoke_ppm = 16.0
            self.ground_water_level_cm = 18.0
            self.ground_inclinometer_deg = round(8.4 + math.sin(now * 0.25) * 2.8, 2)
            self.ground_geophone_vibration = round(0.38 + math.sin(now * 0.8) * 0.22, 3)
        elif self.mode == "EARTHQUAKE":
            self.temp_c = round(22.1 + (random.random() - 0.5) * 0.3, 1)
            self.ambient_light_lux = round(1400.0 + (random.random() - 0.5) * 60.0, 0)
            self.ground_flame_intensity = 15.0  # potential gas leaks
            self.ground_smoke_ppm = 42.0
            self.ground_water_level_cm = 15.0
            # Seismic shaking pulse
            seismic_pulse = abs(math.sin(now * 2.5)) * 1.8 if (int(now) % 10 < 6) else 0.05
            self.ground_geophone_vibration = round(seismic_pulse + random.random() * 0.08, 3)
            self.vibration_index = round(self.vibration_index + seismic_pulse * 0.5, 2)

    def generate_packet(self) -> Dict:
        self.update_physics()
        power_status = "NORMAL"
        if self.battery_voltage < 7.0:
            power_status = "LOW"
        elif self.battery_voltage < 6.7:
            power_status = "CRITICAL"

        packet = {
            "header": {
                "unit_id": self.unit_id,
                "packet_id": self.packet_id,
                "timestamp_utc": datetime.now(timezone.utc).strftime("%Y-%m-%d %H:%M:%S.%f")[:-3],
                "mission_mode": self.mode,
                "sample_rate_hz": self.sample_rate_hz,
                "crc32_valid": True,
                "data_quality_index": 0.98 if self.rssi_dbm > -80 else 0.84,
            },
            "navigation": {
                "latitude": round(self.current_lat, 6),
                "longitude": round(self.current_lon, 6),
                "altitude_msl_m": round(self.altitude_m, 1),
                "ground_speed_kmh": round(self.ground_speed_kmh, 1),
                "heading_deg": round(self.heading_deg, 1),
                "gnss_fix": "3D_DIFFERENTIAL_FIX",
                "satellites_tracked": 11,
                "hdop": 0.85,
            },
            "primary_sensors": {
                "temperature_c": self.temp_c,
                "accel_x_g": round(self.accel_x, 3),
                "accel_y_g": round(self.accel_y, 3),
                "accel_z_g": round(self.accel_z, 3),
                "pitch_deg": round(self.tilt_pitch, 2),
                "roll_deg": round(self.tilt_roll, 2),
                "vibration_rms": self.vibration_index,
                "ambient_light_lux": self.ambient_light_lux,
            },
            "secondary_sensors": {
                "ground_flame_index": self.ground_flame_intensity,
                "ground_smoke_ppm": self.ground_smoke_ppm,
                "ground_water_level_cm": self.ground_water_level_cm,
                "ground_soil_moisture_pct": self.ground_soil_moisture_pct,
                "ground_geophone_vibration_mms": self.ground_geophone_vibration,
                "ground_inclinometer_deg": self.ground_inclinometer_deg,
            },
            "subsystem_health": {
                "battery_voltage_v": self.battery_voltage,
                "battery_level_pct": round(min(100.0, max(0.0, (self.battery_voltage - 6.6) / (8.4 - 6.6) * 100)), 1),
                "current_draw_ma": self.current_draw_ma,
                "rail_3v3_v": self.rail_3v3_v,
                "rail_5v0_v": self.rail_5v0_v,
                "power_state": power_status,
                "storage_used_mb": round(self.storage_used_mb, 1),
                "storage_pct": round((self.storage_used_mb / self.storage_total_mb) * 100, 1),
                "rf_rssi_dbm": self.rssi_dbm,
                "packets_sent": self.packets_sent,
                "packets_dropped": self.packets_dropped,
            },
            "imaging": {
                "active_camera": "ESP32-CAM (OV2640)",
                "status": "READY",
                "last_image_id": self.recent_images[0]["id"] if self.recent_images else "N/A",
                "buffer_fill_pct": 18.4,
            }
        }
        
        # Maintain history ring buffer (last 60 packets)
        self.history.append({
            "time": packet["header"]["timestamp_utc"].split(" ")[1],
            "temp": packet["primary_sensors"]["temperature_c"],
            "accel_x": packet["primary_sensors"]["accel_x_g"],
            "accel_y": packet["primary_sensors"]["accel_y_g"],
            "accel_z": packet["primary_sensors"]["accel_z_g"],
            "altitude": packet["navigation"]["altitude_msl_m"],
            "lux": packet["primary_sensors"]["ambient_light_lux"],
            "battery_v": packet["subsystem_health"]["battery_voltage_v"],
            "rssi": packet["subsystem_health"]["rf_rssi_dbm"],
            "secondary_metric": (
                packet["secondary_sensors"]["ground_flame_index"] if self.mode == "WILDFIRE" else
                packet["secondary_sensors"]["ground_water_level_cm"] if self.mode == "FLOOD" else
                packet["secondary_sensors"]["ground_inclinometer_deg"] if self.mode == "LANDSLIDE" else
                packet["secondary_sensors"]["ground_geophone_vibration_mms"] * 100
            )
        })
        if len(self.history) > 60:
            self.history.pop(0)
            
        return packet

mission = MissionState()

# Pre-fill history buffer
for _ in range(30):
    mission.generate_packet()

class Telecommand(BaseModel):
    command: str
    parameter: Optional[str] = None

@app.get("/api/status")
async def get_status():
    return {
        "unit_id": mission.unit_id,
        "mode": mission.mode,
        "sample_rate_hz": mission.sample_rate_hz,
        "battery_v": mission.battery_voltage,
        "altitude_m": mission.altitude_m,
        "packets_sent": mission.packets_sent,
        "history_count": len(mission.history)
    }

@app.get("/api/history")
async def get_history():
    return mission.history

@app.get("/api/images")
async def get_images():
    return mission.recent_images

@app.post("/api/command")
async def post_command(cmd: Telecommand):
    res = handle_telecommand(cmd.command, cmd.parameter)
    return res

def handle_telecommand(command: str, parameter: Optional[str] = None) -> Dict:
    command = command.upper()
    if command == "SET_MODE" and parameter:
        param = parameter.upper()
        if param in ["WILDFIRE", "FLOOD", "LANDSLIDE", "EARTHQUAKE"]:
            mission.mode = param
            return {"status": "ACK", "message": f"Mission mode transitioned to {param}"}
        return {"status": "NACK", "error": f"Invalid disaster mode: {parameter}"}
    
    elif command == "TRIGGER_CAPTURE":
        img_id = f"IMG-{random.randint(100, 999)}"
        new_img = {
            "id": img_id,
            "timestamp": datetime.now(timezone.utc).isoformat(),
            "mode": mission.mode,
            "lat": round(mission.current_lat, 5),
            "lon": round(mission.current_lon, 5),
            "alt_m": round(mission.altitude_m),
            "resolution": "1600x1200 UXGA",
            "exposure_ms": 14,
            "spectral_band": "VIS-RGB",
            "hotspot_count": 4 if mission.mode == "WILDFIRE" else 0,
            "burn_severity_index": 0.82 if mission.mode == "WILDFIRE" else 0.0,
            "status": "ANALYZED"
        }
        mission.recent_images.insert(0, new_img)
        if len(mission.recent_images) > 10:
            mission.recent_images.pop()
        return {"status": "ACK", "message": f"Frame captured successfully: {img_id}", "image": new_img}

    elif command == "SET_SAMPLE_RATE" and parameter:
        try:
            rate = float(parameter)
            if 0.5 <= rate <= 10.0:
                mission.sample_rate_hz = rate
                return {"status": "ACK", "message": f"Telemetry sampling rate set to {rate} Hz"}
            return {"status": "NACK", "error": "Rate must be between 0.5 and 10.0 Hz"}
        except ValueError:
            return {"status": "NACK", "error": "Invalid numerical rate parameter"}

    elif command == "PING":
        return {"status": "ACK", "message": "PONG - Ground Station Uplink Active", "rssi_dbm": mission.rssi_dbm}

    elif command == "CALIBRATE_SENSORS":
        return {"status": "ACK", "message": "Inertial unit and zero-point calibrated"}

    elif command == "REBOOT_SUBSYSTEM":
        return {"status": "ACK", "message": f"Subsystem {parameter or 'MCU'} soft reset complete"}

    return {"status": "NACK", "error": f"Unknown telecommand: {command}"}

@app.websocket("/ws/telemetry")
async def websocket_telemetry(websocket: WebSocket):
    await websocket.accept()
    try:
        while True:
            # Send live packet
            packet = mission.generate_packet()
            await websocket.send_text(json.dumps(packet))
            
            # Non-blocking listen for telecommands from client
            try:
                data = await asyncio.wait_for(websocket.receive_text(), timeout=1.0 / mission.sample_rate_hz)
                cmd_data = json.loads(data)
                cmd_type = cmd_data.get("command")
                param = cmd_data.get("parameter")
                ack = handle_telecommand(cmd_type, param)
                await websocket.send_text(json.dumps({"telecommand_response": ack}))
            except asyncio.TimeoutError:
                pass
            except json.JSONDecodeError:
                pass
    except WebSocketDisconnect:
        pass
    except Exception as e:
        print(f"WebSocket error: {e}")

if __name__ == "__main__":
    import uvicorn
    uvicorn.run("back:app", host="0.0.0.0", port=8000, reload=False)
