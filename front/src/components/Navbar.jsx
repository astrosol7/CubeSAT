import React, { useState, useEffect } from 'react';
import { 
  Radio, 
  Satellite, 
  Activity, 
  Clock, 
  Wifi, 
  WifiOff, 
  ShieldAlert, 
  Maximize2, 
  RotateCcw,
  Sliders
} from 'lucide-react';

export default function Navbar({ 
  telemetry, 
  connected, 
  activeMode, 
  onModeChange, 
  onSendCommand 
}) {
  const [utcTime, setUtcTime] = useState('');
  const [metSeconds, setMetSeconds] = useState(1482);

  useEffect(() => {
    const timer = setInterval(() => {
      const now = new Date();
      setUtcTime(now.toUTCString().slice(17, 25) + ' UTC');
      setMetSeconds(prev => prev + 1);
    }, 1000);
    return () => clearInterval(timer);
  }, []);

  const formatMet = (totalSec) => {
    const hrs = String(Math.floor(totalSec / 3600)).padStart(2, '0');
    const mins = String(Math.floor((totalSec % 3600) / 60)).padStart(2, '0');
    const secs = String(totalSec % 60).padStart(2, '0');
    return `T+${hrs}:${mins}:${secs}`;
  };

  const getModeBadge = () => {
    switch (activeMode) {
      case 'WILDFIRE':
        return 'bg-amber-500/20 text-amber-400 border-amber-500/30';
      case 'FLOOD':
        return 'bg-cyan-500/20 text-cyan-400 border-cyan-500/30';
      case 'LANDSLIDE':
        return 'bg-emerald-500/20 text-emerald-400 border-emerald-500/30';
      case 'EARTHQUAKE':
        return 'bg-rose-500/20 text-rose-400 border-rose-500/30';
      default:
        return 'bg-blue-500/20 text-blue-400 border-blue-500/30';
    }
  };

  return (
    <header className="v-card px-5 py-3 mb-4 flex flex-wrap items-center justify-between gap-4">
      {/* Brand & Unit Identifier */}
      <div className="flex items-center gap-3">
        <div className="w-10 h-10 rounded-xl bg-gradient-to-tr from-blue-600 to-cyan-400 flex items-center justify-center shadow-lg shadow-blue-500/20">
          <Satellite className="w-5 h-5 text-white" />
        </div>
        <div>
          <div className="flex items-center gap-2">
            <h1 className="font-bold tracking-wide text-white text-base">
              CUBESAT MISSION CONTROL
            </h1>
            <span className="text-[10px] font-semibold tracking-wider uppercase px-2 py-0.5 rounded-full border border-blue-500/30 bg-blue-500/10 text-blue-400">
              2U AIRBORNE PLATFORM
            </span>
          </div>
          <p className="text-xs text-slate-400 font-mono flex items-center gap-2">
            <span>UNIT: <strong className="text-slate-200">U01-ALPHA</strong></span>
            <span>•</span>
            <span>HEX: <span className="text-cyan-400">0x4152-BALLOON</span></span>
          </p>
        </div>
      </div>

      {/* Clocks & Ephemeris */}
      <div className="flex items-center gap-6 text-xs font-mono">
        <div className="bg-slate-900/60 border border-white/5 px-3 py-1.5 rounded-lg flex items-center gap-2">
          <Clock className="w-3.5 h-3.5 text-slate-400" />
          <div>
            <div className="text-[10px] text-slate-400 leading-tight uppercase font-sans">Time Standard</div>
            <div className="text-slate-200 font-bold tracking-wider">{utcTime || '14:32:18 UTC'}</div>
          </div>
        </div>

        <div className="bg-slate-900/60 border border-white/5 px-3 py-1.5 rounded-lg flex items-center gap-2">
          <Activity className="w-3.5 h-3.5 text-cyan-400" />
          <div>
            <div className="text-[10px] text-slate-400 leading-tight uppercase font-sans">Mission Elapsed</div>
            <div className="text-cyan-300 font-bold tracking-wider">{formatMet(metSeconds)}</div>
          </div>
        </div>

        {/* Downlink Carrier Status */}
        <div className="bg-slate-900/60 border border-white/5 px-3 py-1.5 rounded-lg flex items-center gap-2">
          {connected ? (
            <Wifi className="w-3.5 h-3.5 text-emerald-400 animate-pulse" />
          ) : (
            <WifiOff className="w-3.5 h-3.5 text-rose-400" />
          )}
          <div>
            <div className="text-[10px] text-slate-400 leading-tight uppercase font-sans">Downlink Link</div>
            <div className="flex items-center gap-1.5 font-bold">
              <span className={`w-2 h-2 rounded-full ${connected ? 'bg-emerald-400' : 'bg-rose-400'}`}></span>
              <span className={connected ? 'text-emerald-300' : 'text-rose-300'}>
                {connected ? 'CARRIER LOCKED' : 'DISCONNECTED'}
              </span>
            </div>
          </div>
        </div>
      </div>

      {/* Mode Status & Quick Actions */}
      <div className="flex items-center gap-3">
        <div className={`px-3 py-1.5 rounded-xl border text-xs font-bold flex items-center gap-2 ${getModeBadge()}`}>
          <ShieldAlert className="w-3.5 h-3.5" />
          <span>{activeMode} RESPONSE</span>
        </div>

        <button 
          onClick={() => onSendCommand('PING')}
          title="Send Uplink Ping"
          className="p-2 rounded-xl bg-white/5 hover:bg-white/10 border border-white/10 text-slate-300 hover:text-white transition"
        >
          <Radio className="w-4 h-4" />
        </button>

        <button 
          onClick={() => {
            if (!document.fullscreenElement) {
              document.documentElement.requestFullscreen();
            } else {
              document.exitFullscreen();
            }
          }}
          title="Toggle Fullscreen Ground Station"
          className="p-2 rounded-xl bg-white/5 hover:bg-white/10 border border-white/10 text-slate-300 hover:text-white transition"
        >
          <Maximize2 className="w-4 h-4" />
        </button>
      </div>
    </header>
  );
}
