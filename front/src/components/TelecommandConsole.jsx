import React, { useState } from 'react';
import { 
  Terminal, 
  Send, 
  RotateCcw, 
  Sliders, 
  Radio, 
  Camera, 
  CheckCircle2, 
  AlertCircle 
} from 'lucide-react';

export default function TelecommandConsole({ 
  onSendCommand, 
  logs = [], 
  sampleRate = 2.0, 
  onRateChange 
}) {
  const [customCmd, setCustomCmd] = useState('');

  const handleManualSubmit = (e) => {
    e.preventDefault();
    if (!customCmd.trim()) return;
    const parts = customCmd.trim().split(' ');
    const cmd = parts[0];
    const param = parts.slice(1).join(' ') || null;
    onSendCommand(cmd, param);
    setCustomCmd('');
  };

  return (
    <div className="v-card p-4 space-y-3">
      {/* Header */}
      <div className="flex items-center justify-between">
        <div className="flex items-center gap-2">
          <div className="w-7 h-7 rounded-lg bg-cyan-500/20 text-cyan-400 flex items-center justify-center">
            <Terminal className="w-4 h-4" />
          </div>
          <div>
            <h2 className="text-xs font-bold text-white uppercase tracking-wider">
              UPLINK TELECOMMAND & DISPATCHER
            </h2>
            <p className="text-[10px] text-slate-400 font-mono">
              Two-Way Ground Control • Rate Modulation • Subsystem Triggers
            </p>
          </div>
        </div>

        {/* Sampling Frequency Modulation */}
        <div className="flex items-center gap-1 bg-slate-900/80 p-1 rounded-xl border border-white/5 text-[11px] font-mono">
          <span className="text-slate-400 px-1 text-[10px]">RATE:</span>
          {[0.5, 1.0, 2.0, 5.0].map((rate) => (
            <button
              key={rate}
              onClick={() => onRateChange(rate)}
              className={`px-2 py-0.5 rounded-lg font-bold transition ${
                sampleRate === rate
                  ? 'bg-blue-600 text-white'
                  : 'text-slate-400 hover:text-white'
              }`}
            >
              {rate}Hz
            </button>
          ))}
        </div>
      </div>

      {/* Quick Action Telecommand Buttons */}
      <div className="grid grid-cols-2 sm:grid-cols-4 gap-2 text-xs font-mono">
        <button
          onClick={() => onSendCommand('PING')}
          className="p-2 rounded-xl bg-slate-900/80 hover:bg-blue-600/30 border border-white/10 hover:border-blue-500/50 text-slate-200 transition flex items-center justify-center gap-1.5 font-bold"
        >
          <Radio className="w-3.5 h-3.5 text-cyan-400" />
          <span>PING LINK</span>
        </button>

        <button
          onClick={() => onSendCommand('TRIGGER_CAPTURE')}
          className="p-2 rounded-xl bg-slate-900/80 hover:bg-emerald-600/30 border border-white/10 hover:border-emerald-500/50 text-slate-200 transition flex items-center justify-center gap-1.5 font-bold"
        >
          <Camera className="w-3.5 h-3.5 text-emerald-400" />
          <span>CAPTURE</span>
        </button>

        <button
          onClick={() => onSendCommand('CALIBRATE_SENSORS')}
          className="p-2 rounded-xl bg-slate-900/80 hover:bg-amber-600/30 border border-white/10 hover:border-amber-500/50 text-slate-200 transition flex items-center justify-center gap-1.5 font-bold"
        >
          <Sliders className="w-3.5 h-3.5 text-amber-400" />
          <span>CALIBRATE</span>
        </button>

        <button
          onClick={() => onSendCommand('REBOOT_SUBSYSTEM', 'MCU')}
          className="p-2 rounded-xl bg-slate-900/80 hover:bg-rose-600/30 border border-white/10 hover:border-rose-500/50 text-slate-200 transition flex items-center justify-center gap-1.5 font-bold"
        >
          <RotateCcw className="w-3.5 h-3.5 text-rose-400" />
          <span>SOFT RESET</span>
        </button>
      </div>

      {/* Ground Station Terminal Output Log */}
      <div className="bg-slate-950/90 rounded-xl p-3 border border-white/10 font-mono text-[11px] h-32 overflow-y-auto space-y-1">
        {logs.length === 0 ? (
          <div className="text-slate-500 italic">No telecommand activity logged yet...</div>
        ) : (
          logs.map((log, idx) => (
            <div key={idx} className="flex items-start gap-2 leading-tight">
              <span className="text-slate-500 text-[10px]">{log.time}</span>
              <span className={`font-bold ${
                log.type === 'TX' ? 'text-blue-400' :
                log.type === 'ACK' ? 'text-emerald-400' :
                log.type === 'NACK' ? 'text-rose-400' : 'text-slate-300'
              }`}>
                [{log.type}]
              </span>
              <span className="text-slate-200 flex-1">{log.message}</span>
            </div>
          ))
        )}
      </div>

      {/* CLI Input for Custom Commands */}
      <form onSubmit={handleManualSubmit} className="flex items-center gap-2">
        <div className="relative flex-1">
          <span className="absolute left-3 top-2 text-slate-500 font-mono text-xs">CMD&gt;</span>
          <input
            type="text"
            value={customCmd}
            onChange={(e) => setCustomCmd(e.target.value)}
            placeholder="Type telecommand (e.g. SET_MODE WILDFIRE, PING)..."
            className="w-full bg-slate-950/80 border border-white/10 focus:border-blue-500 rounded-xl pl-12 pr-4 py-1.5 text-xs font-mono text-white placeholder-slate-500 outline-none transition"
          />
        </div>
        <button
          type="submit"
          className="bg-blue-600 hover:bg-blue-500 text-white p-2 rounded-xl border border-blue-400/30 transition shadow-md shadow-blue-500/20"
        >
          <Send className="w-3.5 h-3.5" />
        </button>
      </form>
    </div>
  );
}
