import React from 'react';
import { 
  ResponsiveContainer, 
  AreaChart, 
  Area, 
  LineChart, 
  Line, 
  XAxis, 
  YAxis, 
  Tooltip, 
  CartesianGrid 
} from 'recharts';
import { 
  Thermometer, 
  Sun, 
  Activity, 
  Flame, 
  Droplets, 
  Mountain, 
  Zap, 
  Waves,
  ShieldAlert
} from 'lucide-react';

export default function SensorTelemetryGraphs({ 
  telemetry, 
  history, 
  activeMode, 
  onModeChange 
}) {
  const primary = telemetry?.primary_sensors || {
    temperature_c: 28.4,
    accel_x_g: 0.04,
    accel_y_g: -0.02,
    accel_z_g: 0.99,
    vibration_rms: 0.08,
    ambient_light_lux: 1840.0
  };

  const secondary = telemetry?.secondary_sensors || {
    ground_flame_index: 68.0,
    ground_smoke_ppm: 180.0,
    ground_water_level_cm: 145.0,
    ground_soil_moisture_pct: 88.0,
    ground_geophone_vibration_mms: 0.38,
    ground_inclinometer_deg: 8.4
  };

  const getSecondaryInfo = () => {
    switch (activeMode) {
      case 'WILDFIRE':
        return {
          title: 'Ground Node: Flame IR / Smoke',
          val1: `${secondary.ground_flame_index || 68}%`,
          lbl1: 'Flame Intensity',
          val2: `${secondary.ground_smoke_ppm || 180} PPM`,
          lbl2: 'Smoke / CO Density',
          color: '#ffb547',
          icon: <Flame className="w-4 h-4 text-amber-400" />
        };
      case 'FLOOD':
        return {
          title: 'Ground Node: Inundation Depth',
          val1: `${secondary.ground_water_level_cm || 145} cm`,
          lbl1: 'Water Surface Gauge',
          val2: `${secondary.ground_soil_moisture_pct || 88}%`,
          lbl2: 'Soil Saturation',
          color: '#00f2ff',
          icon: <Droplets className="w-4 h-4 text-cyan-400" />
        };
      case 'LANDSLIDE':
        return {
          title: 'Ground Node: Slope Inclinometer',
          val1: `${secondary.ground_inclinometer_deg || 8.4}°`,
          lbl1: 'Displacement Tilt',
          val2: `${(secondary.ground_geophone_vibration_mms || 0.38).toFixed(2)} mm/s`,
          lbl2: 'Micro-seismic Velocity',
          color: '#01b574',
          icon: <Mountain className="w-4 h-4 text-emerald-400" />
        };
      case 'EARTHQUAKE':
        return {
          title: 'Ground Node: Geophone Acceleration',
          val1: `${(secondary.ground_geophone_vibration_mms || 0.85).toFixed(2)} g`,
          lbl1: 'Peak Ground Accel (PGA)',
          val2: 'INTENSITY VII',
          lbl2: 'Modified Mercalli',
          color: '#f43f5e',
          icon: <Waves className="w-4 h-4 text-rose-400" />
        };
      default:
        return {
          title: 'Secondary Sensing Nodes',
          val1: 'NOMINAL',
          lbl1: 'Field Nodes',
          val2: 'STANDBY',
          lbl2: 'Telemetry Bus',
          color: '#0075ff',
          icon: <Activity className="w-4 h-4 text-blue-400" />
        };
    }
  };

  const secInfo = getSecondaryInfo();

  return (
    <div className="v-card p-4 space-y-4">
      {/* 4 Disaster Response Mode Selector */}
      <div>
        <div className="flex items-center justify-between mb-2">
          <div className="flex items-center gap-2">
            <ShieldAlert className="w-4 h-4 text-blue-400" />
            <h2 className="text-xs font-bold text-white uppercase tracking-wider">
              Disaster Response Mode (4 Modes)
            </h2>
          </div>
          <span className="text-[10px] text-slate-400 font-mono">AUTONOMOUS RECONFIGURATION</span>
        </div>

        <div className="grid grid-cols-2 sm:grid-cols-4 gap-1.5">
          <button
            onClick={() => onModeChange('WILDFIRE')}
            className={`px-2.5 py-1.5 rounded-xl border text-xs font-bold transition flex items-center justify-center gap-1.5 ${
              activeMode === 'WILDFIRE'
                ? 'bg-gradient-to-r from-amber-600 to-orange-600 text-white border-amber-400 shadow-lg shadow-amber-500/20'
                : 'bg-slate-900/60 text-slate-400 border-white/5 hover:text-white hover:border-white/10'
            }`}
          >
            <Flame className="w-3.5 h-3.5" />
            <span>Wildfire</span>
          </button>

          <button
            onClick={() => onModeChange('FLOOD')}
            className={`px-2.5 py-1.5 rounded-xl border text-xs font-bold transition flex items-center justify-center gap-1.5 ${
              activeMode === 'FLOOD'
                ? 'bg-gradient-to-r from-cyan-600 to-blue-600 text-white border-cyan-400 shadow-lg shadow-cyan-500/20'
                : 'bg-slate-900/60 text-slate-400 border-white/5 hover:text-white hover:border-white/10'
            }`}
          >
            <Droplets className="w-3.5 h-3.5" />
            <span>Flood</span>
          </button>

          <button
            onClick={() => onModeChange('LANDSLIDE')}
            className={`px-2.5 py-1.5 rounded-xl border text-xs font-bold transition flex items-center justify-center gap-1.5 ${
              activeMode === 'LANDSLIDE'
                ? 'bg-gradient-to-r from-emerald-600 to-teal-600 text-white border-emerald-400 shadow-lg shadow-emerald-500/20'
                : 'bg-slate-900/60 text-slate-400 border-white/5 hover:text-white hover:border-white/10'
            }`}
          >
            <Mountain className="w-3.5 h-3.5" />
            <span>Landslide</span>
          </button>

          <button
            onClick={() => onModeChange('EARTHQUAKE')}
            className={`px-2.5 py-1.5 rounded-xl border text-xs font-bold transition flex items-center justify-center gap-1.5 ${
              activeMode === 'EARTHQUAKE'
                ? 'bg-gradient-to-r from-rose-600 to-red-600 text-white border-rose-400 shadow-lg shadow-rose-500/20'
                : 'bg-slate-900/60 text-slate-400 border-white/5 hover:text-white hover:border-white/10'
            }`}
          >
            <Waves className="w-3.5 h-3.5" />
            <span>Earthquake</span>
          </button>
        </div>
      </div>

      {/* Sensor KPI Stat Cards */}
      <div className="grid grid-cols-3 gap-2">
        {/* Temp Card */}
        <div className="bg-slate-900/70 p-2.5 rounded-xl border border-white/5">
          <div className="flex items-center justify-between text-slate-400 mb-1">
            <span className="text-[10px] uppercase font-sans">Temp</span>
            <Thermometer className="w-3.5 h-3.5 text-amber-400" />
          </div>
          <div className="text-base font-extrabold text-white font-mono">
            {primary.temperature_c?.toFixed(1)}°C
          </div>
          <div className="text-[9px] text-slate-400 font-mono mt-0.5">
            Lapse: -6.5°C/km
          </div>
        </div>

        {/* Accel RMS Card */}
        <div className="bg-slate-900/70 p-2.5 rounded-xl border border-white/5">
          <div className="flex items-center justify-between text-slate-400 mb-1">
            <span className="text-[10px] uppercase font-sans">Dynamics</span>
            <Activity className="w-3.5 h-3.5 text-cyan-400" />
          </div>
          <div className="text-base font-extrabold text-white font-mono">
            {primary.vibration_rms?.toFixed(2)} g
          </div>
          <div className="text-[9px] text-slate-400 font-mono mt-0.5">
            RMS Vibration
          </div>
        </div>

        {/* Light Card */}
        <div className="bg-slate-900/70 p-2.5 rounded-xl border border-white/5">
          <div className="flex items-center justify-between text-slate-400 mb-1">
            <span className="text-[10px] uppercase font-sans">Lux</span>
            <Sun className="w-3.5 h-3.5 text-yellow-400" />
          </div>
          <div className="text-base font-extrabold text-white font-mono">
            {primary.ambient_light_lux?.toFixed(0)} lx
          </div>
          <div className="text-[9px] text-slate-400 font-mono mt-0.5">
            Daylight Index
          </div>
        </div>
      </div>

      {/* Secondary Ground Sensor Live Card */}
      <div className="bg-slate-900/70 p-3 rounded-xl border border-white/10 flex items-center justify-between">
        <div className="flex items-center gap-2.5">
          <div className="w-8 h-8 rounded-lg bg-white/5 flex items-center justify-center">
            {secInfo.icon}
          </div>
          <div>
            <div className="text-xs font-bold text-white">{secInfo.title}</div>
            <div className="text-[10px] text-slate-400 font-mono">Disaster Priority Sensor Node</div>
          </div>
        </div>

        <div className="flex items-center gap-4 text-right font-mono">
          <div>
            <div className="text-xs font-bold text-white">{secInfo.val1}</div>
            <div className="text-[9px] text-slate-400 uppercase font-sans">{secInfo.lbl1}</div>
          </div>
          <div>
            <div className="text-xs font-bold text-cyan-400">{secInfo.val2}</div>
            <div className="text-[9px] text-slate-400 uppercase font-sans">{secInfo.lbl2}</div>
          </div>
        </div>
      </div>

      {/* Live Chart 1: Temperature & Ambient Light */}
      <div className="space-y-1">
        <div className="flex justify-between text-[11px] font-mono text-slate-300">
          <span className="uppercase font-sans font-bold">Ambient Temperature (°C) Telemetry</span>
          <span className="text-cyan-400">SAMPLE BUFFER: {history.length} FRAMES</span>
        </div>
        <div className="h-28 w-full">
          <ResponsiveContainer width="100%" height="100%">
            <AreaChart data={history} margin={{ top: 5, right: 5, left: -25, bottom: 0 }}>
              <defs>
                <linearGradient id="tempGradient" x1="0" y1="0" x2="0" y2="1">
                  <stop offset="5%" stopColor="#0075FF" stopOpacity={0.4} />
                  <stop offset="95%" stopColor="#0075FF" stopOpacity={0.0} />
                </linearGradient>
              </defs>
              <CartesianGrid strokeDasharray="3 3" stroke="rgba(255,255,255,0.05)" />
              <XAxis dataKey="time" stroke="#64748b" tick={{ fontSize: 9 }} hide />
              <YAxis stroke="#64748b" tick={{ fontSize: 9 }} domain={['auto', 'auto']} />
              <Tooltip 
                contentStyle={{ backgroundColor: '#0b1437', borderColor: '#1e293b', fontSize: '11px', borderRadius: '8px' }}
                labelStyle={{ color: '#94a3b8' }}
              />
              <Area 
                type="monotone" 
                dataKey="temp" 
                stroke="#0075FF" 
                strokeWidth={2} 
                fillOpacity={1} 
                fill="url(#tempGradient)" 
                isAnimationActive={false}
              />
            </AreaChart>
          </ResponsiveContainer>
        </div>
      </div>

      {/* Live Chart 2: 3-Axis Platform Dynamics (Accel X, Y, Z in g) */}
      <div className="space-y-1">
        <div className="flex justify-between text-[11px] font-mono text-slate-300">
          <span className="uppercase font-sans font-bold">Platform Dynamics (Accel X/Y/Z g)</span>
          <div className="flex gap-2 text-[10px]">
            <span className="text-rose-400">X</span>
            <span className="text-emerald-400">Y</span>
            <span className="text-cyan-400">Z</span>
          </div>
        </div>
        <div className="h-28 w-full">
          <ResponsiveContainer width="100%" height="100%">
            <LineChart data={history} margin={{ top: 5, right: 5, left: -25, bottom: 0 }}>
              <CartesianGrid strokeDasharray="3 3" stroke="rgba(255,255,255,0.05)" />
              <XAxis dataKey="time" stroke="#64748b" tick={{ fontSize: 9 }} hide />
              <YAxis stroke="#64748b" tick={{ fontSize: 9 }} domain={[-0.5, 1.5]} />
              <Tooltip 
                contentStyle={{ backgroundColor: '#0b1437', borderColor: '#1e293b', fontSize: '11px', borderRadius: '8px' }}
                labelStyle={{ color: '#94a3b8' }}
              />
              <Line type="monotone" dataKey="accel_x" stroke="#f43f5e" strokeWidth={1.5} dot={false} isAnimationActive={false} />
              <Line type="monotone" dataKey="accel_y" stroke="#01b574" strokeWidth={1.5} dot={false} isAnimationActive={false} />
              <Line type="monotone" dataKey="accel_z" stroke="#0075ff" strokeWidth={1.5} dot={false} isAnimationActive={false} />
            </LineChart>
          </ResponsiveContainer>
        </div>
      </div>
    </div>
  );
}
