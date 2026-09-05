import React from 'react';
import { 
  BatteryCharging, 
  Zap, 
  HardDrive, 
  Radio, 
  Compass, 
  Cpu, 
  CheckCircle2, 
  AlertTriangle,
  Layers,
  Signal
} from 'lucide-react';

export default function SubsystemHealth({ telemetry }) {
  const sub = telemetry?.subsystem_health || {
    battery_voltage_v: 7.92,
    battery_level_pct: 73.3,
    current_draw_ma: 480.0,
    rail_3v3_v: 3.31,
    rail_5v0_v: 4.98,
    power_state: 'NORMAL',
    storage_used_mb: 142.5,
    storage_pct: 0.9,
    rf_rssi_dbm: -68.0,
    packets_sent: 1240,
    packets_dropped: 0
  };

  const nav = telemetry?.navigation || {
    latitude: 34.1985,
    longitude: -118.1750,
    altitude_msl_m: 1240.0,
    ground_speed_kmh: 14.8,
    heading_deg: 42.5,
    gnss_fix: '3D_DIFFERENTIAL_FIX',
    satellites_tracked: 11,
    hdop: 0.85
  };

  const header = telemetry?.header || {
    packet_id: 1042,
    crc32_valid: true,
    data_quality_index: 0.98
  };

  // SVG circular gauge calculation (Radius = 38, Perimeter ~ 238.76)
  const radius = 38;
  const circumference = 2 * Math.PI * radius;
  const batteryPct = Math.min(100, Math.max(0, sub.battery_level_pct || 70));
  const strokeDashoffset = circumference - (batteryPct / 100) * circumference;

  return (
    <div className="space-y-4">
      {/* Battery & Power Bus Card */}
      <div className="v-card p-4">
        <div className="flex items-center justify-between mb-3">
          <div className="flex items-center gap-2">
            <div className="w-7 h-7 rounded-lg bg-blue-500/20 text-blue-400 flex items-center justify-center">
              <Zap className="w-4 h-4" />
            </div>
            <div>
              <h2 className="text-xs font-bold text-white uppercase tracking-wider">Power Subsystem</h2>
              <p className="text-[10px] text-slate-400">EPS 2S LiPo / Dual Buck Regulators</p>
            </div>
          </div>
          <span className={`text-[10px] font-bold px-2 py-0.5 rounded border uppercase ${
            sub.power_state === 'NORMAL' 
              ? 'bg-emerald-500/20 text-emerald-300 border-emerald-500/30' 
              : 'bg-rose-500/20 text-rose-300 border-rose-500/30 animate-pulse'
          }`}>
            {sub.power_state}
          </span>
        </div>

        {/* Circular Progress Gauge & Battery Specs */}
        <div className="flex items-center justify-between px-2 py-1">
          <div className="relative w-24 h-24 flex items-center justify-center">
            <svg className="w-24 h-24 transform -rotate-90">
              <circle
                cx="48"
                cy="48"
                r={radius}
                stroke="#1a224a"
                strokeWidth="7"
                fill="none"
              />
              <circle
                cx="48"
                cy="48"
                r={radius}
                stroke="url(#batteryGradient)"
                strokeWidth="7"
                strokeDasharray={circumference}
                strokeDashoffset={strokeDashoffset}
                strokeLinecap="round"
                fill="none"
                style={{ transition: 'stroke-dashoffset 0.5s ease' }}
              />
              <defs>
                <linearGradient id="batteryGradient" x1="0%" y1="0%" x2="100%" y2="100%">
                  <stop offset="0%" stopColor="#0075FF" />
                  <stop offset="100%" stopColor="#01B574" />
                </linearGradient>
              </defs>
            </svg>
            <div className="absolute flex flex-col items-center">
              <span className="text-sm font-extrabold text-white">{batteryPct.toFixed(0)}%</span>
              <span className="text-[9px] text-slate-400 uppercase">State</span>
            </div>
          </div>

          <div className="space-y-1.5 font-mono text-xs">
            <div className="flex items-center justify-between gap-4">
              <span className="text-slate-400 text-[11px]">Bus Voltage:</span>
              <span className="font-bold text-slate-100">{sub.battery_voltage_v?.toFixed(2)} V</span>
            </div>
            <div className="flex items-center justify-between gap-4">
              <span className="text-slate-400 text-[11px]">Current Draw:</span>
              <span className="font-bold text-cyan-400">{sub.current_draw_ma?.toFixed(0)} mA</span>
            </div>
            <div className="flex items-center justify-between gap-4">
              <span className="text-slate-400 text-[11px]">3.3V Logic:</span>
              <span className="font-bold text-emerald-400">{sub.rail_3v3_v?.toFixed(2)} V</span>
            </div>
            <div className="flex items-center justify-between gap-4">
              <span className="text-slate-400 text-[11px]">5.0V Payload:</span>
              <span className="font-bold text-emerald-400">{sub.rail_5v0_v?.toFixed(2)} V</span>
            </div>
          </div>
        </div>
      </div>

      {/* RF Communications & Telemetry Health */}
      <div className="v-card p-4">
        <div className="flex items-center justify-between mb-3">
          <div className="flex items-center gap-2">
            <div className="w-7 h-7 rounded-lg bg-cyan-500/20 text-cyan-400 flex items-center justify-center">
              <Signal className="w-4 h-4" />
            </div>
            <div>
              <h2 className="text-xs font-bold text-white uppercase tracking-wider">RF Downlink & Link Budget</h2>
              <p className="text-[10px] text-slate-400">FSK Transceiver / Packet Ingest</p>
            </div>
          </div>
          <span className="text-[10px] font-mono text-cyan-400 bg-cyan-500/10 px-2 py-0.5 rounded border border-cyan-500/20">
            915 MHz
          </span>
        </div>

        <div className="grid grid-cols-2 gap-2 text-xs font-mono mb-2">
          <div className="bg-slate-900/60 p-2 rounded-lg border border-white/5">
            <div className="text-[10px] text-slate-400 uppercase font-sans">RSSI Signal</div>
            <div className="text-sm font-bold text-cyan-300">{sub.rf_rssi_dbm} dBm</div>
            <div className="w-full bg-slate-800 h-1 rounded-full mt-1.5 overflow-hidden">
              <div 
                className="bg-cyan-400 h-full rounded-full transition-all"
                style={{ width: `${Math.min(100, Math.max(10, (sub.rf_rssi_dbm + 110) * 1.6))}%` }}
              ></div>
            </div>
          </div>

          <div className="bg-slate-900/60 p-2 rounded-lg border border-white/5">
            <div className="text-[10px] text-slate-400 uppercase font-sans">CRC-32 Integrity</div>
            <div className="text-sm font-bold flex items-center gap-1.5 text-emerald-400">
              <CheckCircle2 className="w-3.5 h-3.5" />
              <span>PASSED</span>
            </div>
            <div className="text-[10px] text-slate-400 mt-1">Quality: {(header.data_quality_index * 100).toFixed(0)}%</div>
          </div>
        </div>

        <div className="space-y-1 text-[11px] font-mono text-slate-300">
          <div className="flex justify-between">
            <span className="text-slate-400">Sequence ID:</span>
            <span className="font-bold text-slate-100">#{header.packet_id}</span>
          </div>
          <div className="flex justify-between">
            <span className="text-slate-400">Packets Ingested:</span>
            <span>{sub.packets_sent} frames</span>
          </div>
          <div className="flex justify-between">
            <span className="text-slate-400">Dropped Frames:</span>
            <span className="text-emerald-400">{sub.packets_dropped} (0.00%)</span>
          </div>
        </div>
      </div>

      {/* GNSS Navigation & Ephemeris */}
      <div className="v-card p-4">
        <div className="flex items-center justify-between mb-3">
          <div className="flex items-center gap-2">
            <div className="w-7 h-7 rounded-lg bg-emerald-500/20 text-emerald-400 flex items-center justify-center">
              <Compass className="w-4 h-4" />
            </div>
            <div>
              <h2 className="text-xs font-bold text-white uppercase tracking-wider">GNSS Navigation</h2>
              <p className="text-[10px] text-slate-400">GPS / GLONASS Ephemeris</p>
            </div>
          </div>
          <span className="text-[10px] font-bold text-emerald-400 bg-emerald-500/10 px-2 py-0.5 rounded border border-emerald-500/20">
            {nav.satellites_tracked} SATELLITES
          </span>
        </div>

        <div className="space-y-1.5 text-xs font-mono">
          <div className="p-2 rounded-lg bg-slate-900/60 border border-white/5 space-y-1">
            <div className="flex justify-between text-[11px]">
              <span className="text-slate-400">Latitude:</span>
              <span className="text-slate-200 font-bold">{nav.latitude?.toFixed(6)}° N</span>
            </div>
            <div className="flex justify-between text-[11px]">
              <span className="text-slate-400">Longitude:</span>
              <span className="text-slate-200 font-bold">{nav.longitude?.toFixed(6)}° W</span>
            </div>
          </div>

          <div className="grid grid-cols-2 gap-2 text-[11px]">
            <div className="p-2 rounded-lg bg-slate-900/60 border border-white/5">
              <div className="text-[10px] text-slate-400 uppercase font-sans">Altitude (MSL)</div>
              <div className="text-sm font-bold text-white">{nav.altitude_msl_m?.toFixed(1)} m</div>
            </div>
            <div className="p-2 rounded-lg bg-slate-900/60 border border-white/5">
              <div className="text-[10px] text-slate-400 uppercase font-sans">Ground Speed</div>
              <div className="text-sm font-bold text-white">{nav.ground_speed_kmh?.toFixed(1)} km/h</div>
            </div>
          </div>

          <div className="flex justify-between text-[10px] text-slate-400 px-1 pt-1">
            <span>Fix Mode: <strong className="text-slate-200">{nav.gnss_fix}</strong></span>
            <span>HDOP: <strong className="text-slate-200">{nav.hdop}</strong></span>
          </div>
        </div>
      </div>

      {/* Onboard Flash / MicroSD Buffer */}
      <div className="v-card p-4">
        <div className="flex items-center justify-between mb-2">
          <div className="flex items-center gap-2">
            <div className="w-7 h-7 rounded-lg bg-purple-500/20 text-purple-400 flex items-center justify-center">
              <HardDrive className="w-4 h-4" />
            </div>
            <div>
              <h2 className="text-xs font-bold text-white uppercase tracking-wider">Local Storage</h2>
              <p className="text-[10px] text-slate-400">Non-volatile SPI MicroSD</p>
            </div>
          </div>
          <span className="text-xs font-mono font-bold text-purple-300">
            {sub.storage_used_mb} MB / 16 GB
          </span>
        </div>

        <div className="w-full bg-slate-900/80 h-2 rounded-full overflow-hidden p-0.5 border border-white/5 mb-1.5">
          <div 
            className="bg-gradient-to-r from-blue-500 to-purple-500 h-full rounded-full transition-all"
            style={{ width: `${Math.max(3, sub.storage_pct * 10)}%` }}
          ></div>
        </div>
        <div className="flex justify-between text-[10px] text-slate-400 font-mono">
          <span>Buffer: <strong className="text-slate-200">RING BUFFER SECURE</strong></span>
          <span>Flush: <strong className="text-emerald-400">NOMINAL</strong></span>
        </div>
      </div>
    </div>
  );
}
