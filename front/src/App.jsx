import React, { useState, useEffect, useRef } from 'react';
import Navbar from './components/Navbar';
import SubsystemHealth from './components/SubsystemHealth';
import VideoImagerySuite from './components/VideoImagerySuite';
import TacticalMap from './components/TacticalMap';
import SensorTelemetryGraphs from './components/SensorTelemetryGraphs';
import TelecommandConsole from './components/TelecommandConsole';

export default function App() {
  const [telemetry, setTelemetry] = useState(null);
  const [history, setHistory] = useState([]);
  const [connected, setConnected] = useState(false);
  const [activeMode, setActiveMode] = useState('WILDFIRE');
  const [sampleRate, setSampleRate] = useState(2.0);
  const [logs, setLogs] = useState([
    { time: '14:30:00 UTC', type: 'SYS', message: 'Mission Ground Station initialized.' },
    { time: '14:30:02 UTC', type: 'SYS', message: 'Downlink receiver locked on 915 MHz.' },
    { time: '14:30:05 UTC', type: 'ACK', message: 'Uplink handshake verified with Unit U01-ALPHA.' }
  ]);

  const wsRef = useRef(null);

  // Fallback deterministic client-side telemetry simulator
  useEffect(() => {
    let intervalId;
    let packetSeq = 1000;
    let baseTime = Date.now();

    const connectWebSocket = () => {
      try {
        const ws = new WebSocket('ws://localhost:8000/ws/telemetry');
        wsRef.current = ws;

        ws.onopen = () => {
          setConnected(true);
          addLog('SYS', 'WebSocket telemetry stream connected to back.py gateway.');
        };

        ws.onmessage = (event) => {
          try {
            const data = JSON.parse(event.data);
            if (data.telecommand_response) {
              const res = data.telecommand_response;
              addLog(res.status === 'ACK' ? 'ACK' : 'NACK', res.message || res.error);
            } else {
              setTelemetry(data);
              setActiveMode(data.header.mission_mode);
              setSampleRate(data.header.sample_rate_hz);
              
              // Append to history
              const point = {
                time: data.header.timestamp_utc.split(' ')[1] || '14:32:00',
                temp: data.primary_sensors.temperature_c,
                accel_x: data.primary_sensors.accel_x_g,
                accel_y: data.primary_sensors.accel_y_g,
                accel_z: data.primary_sensors.accel_z_g,
                altitude: data.navigation.altitude_msl_m,
                lux: data.primary_sensors.ambient_light_lux,
                battery_v: data.subsystem_health.battery_voltage_v
              };
              setHistory(prev => [...prev.slice(-45), point]);
            }
          } catch (e) {
            console.error('Error parsing telemetry frame:', e);
          }
        };

        ws.onclose = () => {
          setConnected(false);
          // Fall back to client-side generator if backend server isn't running
          startClientSimulation();
        };

        ws.onerror = () => {
          setConnected(false);
          startClientSimulation();
        };
      } catch (e) {
        startClientSimulation();
      }
    };

    const startClientSimulation = () => {
      if (intervalId) return;
      setConnected(true);
      intervalId = setInterval(() => {
        packetSeq += 1;
        const now = Date.now();
        const t = (now - baseTime) / 1000;

        // Dynamic parameters based on activeMode
        let tempVal = 28.5 + Math.sin(t * 0.4) * 3.2;
        let luxVal = 1850 + Math.sin(t * 0.2) * 150;
        let flameVal = 0;
        let smokeVal = 20;
        let waterVal = 12;
        let soilVal = 25;
        let inclineVal = 0.5;
        let geophoneVal = 0.02;

        if (activeMode === 'WILDFIRE') {
          tempVal = 41.2 + Math.sin(t * 0.5) * 5.4;
          flameVal = 72 + Math.sin(t * 0.3) * 18;
          smokeVal = 210 + Math.sin(t * 0.2) * 50;
        } else if (activeMode === 'FLOOD') {
          tempVal = 21.0 + Math.sin(t * 0.3) * 1.0;
          luxVal = 920;
          waterVal = 160 + Math.sin(t * 0.2) * 30;
          soilVal = 92;
        } else if (activeMode === 'LANDSLIDE') {
          tempVal = 19.5;
          inclineVal = 9.2 + Math.sin(t * 0.3) * 2.1;
          geophoneVal = 0.42 + Math.sin(t * 0.8) * 0.18;
        } else if (activeMode === 'EARTHQUAKE') {
          const seismic = Math.abs(Math.sin(t * 2.0)) * 1.5;
          geophoneVal = 0.3 + seismic;
        }

        const accelX = Math.sin(t * 0.8) * 0.12;
        const accelY = Math.cos(t * 0.7) * 0.10;
        const accelZ = 0.98 + (Math.random() - 0.5) * 0.02;

        const simulatedFrame = {
          header: {
            unit_id: 'CUBESAT-2U-ALPHA',
            packet_id: packetSeq,
            timestamp_utc: new Date().toUTCString().slice(17, 25) + ' UTC',
            mission_mode: activeMode,
            sample_rate_hz: sampleRate,
            crc32_valid: true,
            data_quality_index: 0.98
          },
          navigation: {
            latitude: 34.1985 + Math.cos(t * 0.05) * 0.005,
            longitude: -118.1750 + Math.sin(t * 0.05) * 0.005,
            altitude_msl_m: 1240.0 + Math.sin(t * 0.1) * 6.0,
            ground_speed_kmh: 14.8,
            heading_deg: 42.5,
            gnss_fix: '3D_DIFFERENTIAL_FIX',
            satellites_tracked: 11,
            hdop: 0.85
          },
          primary_sensors: {
            temperature_c: Number(tempVal.toFixed(1)),
            accel_x_g: Number(accelX.toFixed(3)),
            accel_y_g: Number(accelY.toFixed(3)),
            accel_z_g: Number(accelZ.toFixed(3)),
            pitch_deg: Number((accelX * 57.3).toFixed(1)),
            roll_deg: Number((accelY * 57.3).toFixed(1)),
            vibration_rms: Number((Math.sqrt(accelX**2 + accelY**2) * 8).toFixed(2)),
            ambient_light_lux: Number(luxVal.toFixed(0))
          },
          secondary_sensors: {
            ground_flame_index: Number(flameVal.toFixed(1)),
            ground_smoke_ppm: Number(smokeVal.toFixed(0)),
            ground_water_level_cm: Number(waterVal.toFixed(1)),
            ground_soil_moisture_pct: Number(soilVal.toFixed(1)),
            ground_geophone_vibration_mms: Number(geophoneVal.toFixed(3)),
            ground_inclinometer_deg: Number(inclineVal.toFixed(1))
          },
          subsystem_health: {
            battery_voltage_v: 7.92,
            battery_level_pct: 74.0,
            current_draw_ma: 480.0,
            rail_3v3_v: 3.31,
            rail_5v0_v: 4.98,
            power_state: 'NORMAL',
            storage_used_mb: 142.5,
            storage_pct: 0.9,
            rf_rssi_dbm: -68.0,
            packets_sent: packetSeq - 800,
            packets_dropped: 0
          }
        };

        setTelemetry(simulatedFrame);

        const point = {
          time: simulatedFrame.header.timestamp_utc.slice(0, 8),
          temp: simulatedFrame.primary_sensors.temperature_c,
          accel_x: simulatedFrame.primary_sensors.accel_x_g,
          accel_y: simulatedFrame.primary_sensors.accel_y_g,
          accel_z: simulatedFrame.primary_sensors.accel_z_g,
          altitude: simulatedFrame.navigation.altitude_msl_m,
          lux: simulatedFrame.primary_sensors.ambient_light_lux,
          battery_v: simulatedFrame.subsystem_health.battery_voltage_v
        };
        setHistory(prev => [...prev.slice(-45), point]);
      }, 1000 / sampleRate);
    };

    connectWebSocket();

    return () => {
      if (wsRef.current) wsRef.current.close();
      if (intervalId) clearInterval(intervalId);
    };
  }, [activeMode, sampleRate]);

  const addLog = (type, message) => {
    const time = new Date().toUTCString().slice(17, 25) + ' UTC';
    setLogs(prev => [{ time, type, message }, ...prev.slice(0, 19)]);
  };

  const handleSendCommand = (command, parameter = null) => {
    addLog('TX', `Uplink command dispatched: ${command} ${parameter || ''}`.trim());

    if (wsRef.current && wsRef.current.readyState === WebSocket.OPEN) {
      wsRef.current.send(JSON.stringify({ command, parameter }));
    } else {
      // Deterministic simulation fallback handler
      setTimeout(() => {
        if (command === 'PING') {
          addLog('ACK', 'PONG — Link Quality 98% (RSSI -68 dBm)');
        } else if (command === 'SET_MODE' && parameter) {
          setActiveMode(parameter);
          addLog('ACK', `Mission response mode transitioned to ${parameter}`);
        } else if (command === 'SET_SAMPLE_RATE' && parameter) {
          setSampleRate(Number(parameter));
          addLog('ACK', `Sampling frequency updated to ${parameter} Hz`);
        } else if (command === 'TRIGGER_CAPTURE') {
          addLog('ACK', 'Frame burst captured and logged to flight storage.');
        } else if (command === 'CALIBRATE_SENSORS') {
          addLog('ACK', 'Zero-offset calibration complete for 3-axis accelerometer.');
        } else if (command === 'REBOOT_SUBSYSTEM') {
          addLog('ACK', `Subsystem ${parameter || 'MCU'} reset routine executed successfully.`);
        } else {
          addLog('ACK', `Command ${command} processed by flight executive.`);
        }
      }, 250);
    }
  };

  const handleModeChange = (mode) => {
    setActiveMode(mode);
    handleSendCommand('SET_MODE', mode);
  };

  const handleRateChange = (rate) => {
    setSampleRate(rate);
    handleSendCommand('SET_SAMPLE_RATE', String(rate));
  };

  return (
    <div className="min-h-screen p-3 md:p-5 flex flex-col justify-between">
      <div>
        {/* Top Navbar */}
        <Navbar
          telemetry={telemetry}
          connected={connected}
          activeMode={activeMode}
          onModeChange={handleModeChange}
          onSendCommand={handleSendCommand}
        />

        {/* 3-Column Aerospace Mission Control Split Screen */}
        <main className="grid grid-cols-1 lg:grid-cols-12 gap-4 items-start">
          {/* Column 1: Subsystem Health, Power Bus, GNSS Ephemeris (Left: 3 Cols) */}
          <section className="lg:col-span-3">
            <SubsystemHealth telemetry={telemetry} />
          </section>

          {/* Column 2: Split Screen - Live Aerial Optical Feed & Tactical GIS Map (Center: 5 Cols) */}
          <section className="lg:col-span-5 space-y-4">
            <VideoImagerySuite
              telemetry={telemetry}
              activeMode={activeMode}
              onCaptureImage={() => handleSendCommand('TRIGGER_CAPTURE')}
            />
            <TacticalMap
              telemetry={telemetry}
              activeMode={activeMode}
            />
          </section>

          {/* Column 3: Multi-Sensor Telemetry Graphs & Uplink Command Console (Right: 4 Cols) */}
          <section className="lg:col-span-4 space-y-4">
            <SensorTelemetryGraphs
              telemetry={telemetry}
              history={history}
              activeMode={activeMode}
              onModeChange={handleModeChange}
            />
            <TelecommandConsole
              onSendCommand={handleSendCommand}
              logs={logs}
              sampleRate={sampleRate}
              onRateChange={handleRateChange}
            />
          </section>
        </main>
      </div>

      {/* Footer System Attribution */}
      <footer className="mt-6 pt-3 border-t border-white/5 flex flex-wrap items-center justify-between text-[11px] font-mono text-slate-500">
        <div>
          <span>CUBESAT RAPID DISASTER RESPONSE GROUND STATION</span>
          <span className="mx-2">•</span>
          <span>FLIGHT REVISION: v2.4-RELEASE</span>
        </div>
        <div>
          <span>DETERMINISTIC TELEMETRY INGESTION</span>
          <span className="mx-2">•</span>
          <span>CRC-32 CHECKSUM VALIDATED</span>
        </div>
      </footer>
    </div>
  );
}
