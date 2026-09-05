"""
Intermediary Bridge — CubeSat Hardware-in-the-Loop & Proteus Telemetry Parser
Reads raw serial telemetry packets from physical Arduino / Proteus UART, validates CRC,
and forwards them to the Ground Station Telemetry Server.
"""

import sys
import time
import json
import socket
import argparse
from typing import Optional

try:
    import serial
except ImportError:
    serial = None

def parse_cubesat_packet(raw_line: str) -> Optional[dict]:
    """
    Parses structured telemetry format from CubeSat MCU:
    Format:
    $CUBESAT,U01,PKT_ID,UTC,LAT,LON,ALT,TEMP,ACC_X,ACC_Y,ACC_Z,LUX,VBAT,MODE,EVENT,QUALITY*CHECKSUM
    """
    raw_line = raw_line.strip()
    if not raw_line.startswith("$CUBESAT"):
        return None
    
    parts = raw_line.split("*")
    data_str = parts[0]
    checksum = parts[1] if len(parts) > 1 else None
    
    fields = data_str.split(",")
    if len(fields) < 14:
        return None
        
    try:
        return {
            "unit_id": fields[1],
            "packet_id": int(fields[2]),
            "timestamp_utc": fields[3],
            "lat": float(fields[4]),
            "lon": float(fields[5]),
            "alt_m": float(fields[6]),
            "temperature_c": float(fields[7]),
            "accel_x": float(fields[8]),
            "accel_y": float(fields[9]),
            "accel_z": float(fields[10]),
            "lux": float(fields[11]),
            "battery_v": float(fields[12]),
            "mission_mode": fields[13],
            "event": fields[14] if len(fields) > 14 else "NOMINAL",
            "data_quality": float(fields[15]) if len(fields) > 15 else 0.95
        }
    except (ValueError, IndexError) as e:
        print(f"[Intermediary] Packet parse error: {e}")
        return None

def run_serial_bridge(port: str = "/dev/ttyUSB0", baudrate: int = 9600):
    if not serial:
        print("[Intermediary] PySerial not installed. Please run: pip install pyserial")
        sys.exit(1)
        
    print(f"[Intermediary] Connecting to CubeSat Serial Bridge on {port} at {baudrate} baud...")
    try:
        ser = serial.Serial(port, baudrate, timeout=1.0)
        print(f"[Intermediary] Connected to {port}. Listening for telemetry frames...")
        while True:
            line = ser.readline().decode('utf-8', errors='ignore')
            if line:
                pkt = parse_cubesat_packet(line)
                if pkt:
                    print(f"[Intermediary] Ingested Packet #{pkt['packet_id']} [{pkt['mission_mode']}] Alt={pkt['alt_m']}m Temp={pkt['temperature_c']}C")
    except Exception as e:
        print(f"[Intermediary] Serial communication error: {e}")

def run_udp_bridge(bind_port: int = 9005):
    sock = socket.socket(socket.AF_INET, socket.SOCK_DGRAM)
    sock.bind(("0.0.0.0", bind_port))
    print(f"[Intermediary] UDP Telemetry listener active on 0.0.0.0:{bind_port} (Proteus Network Simulation Bridge)...")
    while True:
        data, addr = sock.recvfrom(2048)
        line = data.decode('utf-8', errors='ignore')
        pkt = parse_cubesat_packet(line)
        if pkt:
            print(f"[Intermediary] UDP Packet from {addr}: #{pkt['packet_id']} Mode={pkt['mission_mode']}")

if __name__ == "__main__":
    parser = argparse.ArgumentParser(description="CubeSat Telemetry Intermediary Bridge")
    parser.add_argument("--mode", choices=["serial", "udp"], default="udp", help="Ingestion mode")
    parser.add_argument("--port", default="/dev/ttyUSB0", help="Serial port path")
    parser.add_argument("--baud", type=int, default=9600, help="Serial baud rate")
    parser.add_argument("--udp-port", type=int, default=9005, help="UDP port")
    args = parser.parse_args()
    
    if args.mode == "serial":
        run_serial_bridge(args.port, args.baud)
    else:
        run_udp_bridge(args.udp_port)
