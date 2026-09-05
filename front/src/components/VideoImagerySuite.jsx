import React, { useState, useEffect, useRef } from 'react';
import { 
  Camera, 
  Eye, 
  Flame, 
  Maximize, 
  Minimize, 
  Layers, 
  Crosshair, 
  Scan, 
  Sliders, 
  Check, 
  Download,
  AlertCircle
} from 'lucide-react';

export default function VideoImagerySuite({ telemetry, activeMode, onCaptureImage }) {
  const [filterMode, setFilterMode] = useState('RGB'); // 'RGB', 'THERMAL', 'EDGE'
  const [capturing, setCapturing] = useState(false);
  const [snapshots, setSnapshots] = useState([]);
  const canvasRef = useRef(null);

  // Platform tilt from accelerometer
  const pitch = telemetry?.primary_sensors?.pitch_deg || 2.1;
  const roll = telemetry?.primary_sensors?.roll_deg || -1.4;
  const alt = telemetry?.navigation?.altitude_msl_m || 1240;
  const heading = telemetry?.navigation?.heading_deg || 42.5;

  // Initialize sample captured snapshots
  useEffect(() => {
    setSnapshots([
      {
        id: 'CAP-0092',
        time: '14:32:04 UTC',
        mode: 'WILDFIRE',
        band: 'OPTICAL-RGB',
        hotspots: 4,
        metric: 'Burn Index: 0.81',
        url: 'https://images.unsplash.com/photo-1602980085566-480922883f3a?auto=format&fit=crop&w=400&q=80'
      },
      {
        id: 'CAP-0091',
        time: '14:31:18 UTC',
        mode: 'WILDFIRE',
        band: 'THERMAL-IR',
        hotspots: 3,
        metric: 'Peak Temp: 462°C',
        url: 'https://images.unsplash.com/photo-1509198397868-475647b2a1e5?auto=format&fit=crop&w=400&q=80'
      },
      {
        id: 'CAP-0090',
        time: '14:29:45 UTC',
        mode: 'FLOOD',
        band: 'OPTICAL-RGB',
        hotspots: 0,
        metric: 'Inundation: 3.4 km²',
        url: 'https://images.unsplash.com/photo-1547683905-f686c993aae5?auto=format&fit=crop&w=400&q=80'
      }
    ]);
  }, []);

  // Animated aerial canvas simulation
  useEffect(() => {
    const canvas = canvasRef.current;
    if (!canvas) return;
    const ctx = canvas.getContext('2d');
    let animationFrameId;
    let offset = 0;

    const render = () => {
      offset += 0.4;
      const width = canvas.width;
      const height = canvas.height;

      // Base terrain background
      if (filterMode === 'THERMAL') {
        // High-contrast false-color thermal ironbow
        const bgGrad = ctx.createLinearGradient(0, 0, width, height);
        bgGrad.addColorStop(0, '#0a0026');
        bgGrad.addColorStop(0.3, '#330066');
        bgGrad.addColorStop(0.7, '#800040');
        bgGrad.addColorStop(1, '#ff3300');
        ctx.fillStyle = bgGrad;
        ctx.fillRect(0, 0, width, height);

        // Heat signatures / thermal clusters
        const heatX = (width * 0.55) + Math.sin(offset * 0.02) * 20;
        const heatY = (height * 0.45) + Math.cos(offset * 0.02) * 15;
        
        const radial = ctx.createRadialGradient(heatX, heatY, 10, heatX, heatY, 120);
        radial.addColorStop(0, '#ffffff');
        radial.addColorStop(0.3, '#ffff00');
        radial.addColorStop(0.6, '#ff4400');
        radial.addColorStop(1, 'transparent');
        ctx.fillStyle = radial;
        ctx.beginPath();
        ctx.arc(heatX, heatY, 120, 0, Math.PI * 2);
        ctx.fill();

        // Secondary thermal spot
        const rad2 = ctx.createRadialGradient(heatX - 80, heatY + 50, 5, heatX - 80, heatY + 50, 60);
        rad2.addColorStop(0, '#ffffdd');
        rad2.addColorStop(0.4, '#ff6600');
        rad2.addColorStop(1, 'transparent');
        ctx.fillStyle = rad2;
        ctx.beginPath();
        ctx.arc(heatX - 80, heatY + 50, 60, 0, Math.PI * 2);
        ctx.fill();

      } else if (filterMode === 'EDGE') {
        // High-pass Sobel gradient / structural outline vectoring
        ctx.fillStyle = '#030712';
        ctx.fillRect(0, 0, width, height);

        ctx.strokeStyle = '#00f2ff';
        ctx.lineWidth = 1.5;

        // Vector terrain contours
        for (let i = -2; i < 7; i++) {
          const y = ((i * 50 + offset * 1.5) % height);
          ctx.beginPath();
          ctx.moveTo(0, y);
          ctx.bezierCurveTo(width * 0.3, y - 25, width * 0.7, y + 25, width, y);
          ctx.stroke();
        }

        // Road / river feature outline
        ctx.strokeStyle = '#39ff14';
        ctx.lineWidth = 2.5;
        ctx.beginPath();
        const roadX = width * 0.45 + Math.sin(offset * 0.01) * 30;
        ctx.moveTo(roadX, 0);
        ctx.lineTo(roadX + 40, height * 0.4);
        ctx.lineTo(roadX - 20, height);
        ctx.stroke();

        // Building / structural bounding boxes (deterministic edge detection)
        ctx.strokeStyle = '#ff0055';
        ctx.strokeRect(width * 0.6, height * 0.3, 48, 36);
        ctx.strokeRect(width * 0.68, height * 0.4, 40, 30);
        ctx.strokeRect(width * 0.3, height * 0.65, 55, 42);

      } else {
        // Standard RGB Optical Observation
        const grad = ctx.createLinearGradient(0, 0, width, height);
        grad.addColorStop(0, '#1c311c');
        grad.addColorStop(0.5, '#2d4a27');
        grad.addColorStop(1, '#415e37');
        ctx.fillStyle = grad;
        ctx.fillRect(0, 0, width, height);

        // Terrain hills & shadows
        ctx.fillStyle = '#22381f';
        ctx.beginPath();
        ctx.ellipse(width * 0.7, height * 0.3, 140, 80, 0.4, 0, Math.PI * 2);
        ctx.fill();

        // River / Water body
        ctx.fillStyle = '#1e3a5f';
        ctx.beginPath();
        const riverX = width * 0.35 + Math.sin(offset * 0.01) * 20;
        ctx.moveTo(riverX, 0);
        ctx.bezierCurveTo(riverX + 50, height * 0.4, riverX - 30, height * 0.7, riverX + 20, height);
        ctx.lineWidth = 32;
        ctx.strokeStyle = '#1e3a5f';
        ctx.stroke();

        // Smoke / Burn plume if in Wildfire mode
        if (activeMode === 'WILDFIRE') {
          ctx.fillStyle = 'rgba(70, 70, 70, 0.45)';
          for (let s = 0; s < 4; s++) {
            const sx = (width * 0.58) + Math.sin(offset * 0.03 + s) * 25 + s * 18;
            const sy = (height * 0.4) - (offset * 0.8 + s * 30) % (height * 0.7);
            ctx.beginPath();
            ctx.arc(sx, sy, 35 + s * 10, 0, Math.PI * 2);
            ctx.fill();
          }
          // Active fire glow
          ctx.fillStyle = 'rgba(255, 90, 0, 0.85)';
          ctx.beginPath();
          ctx.arc(width * 0.58, height * 0.45, 14, 0, Math.PI * 2);
          ctx.fill();
        }
      }

      // HUD Reticle Overlay (Crosshair & Gyro attitude indicators)
      const cx = width / 2;
      const cy = height / 2;

      // Platform sway shift
      const shiftX = roll * 4;
      const shiftY = pitch * 4;

      ctx.save();
      ctx.translate(cx + shiftX, cy + shiftY);

      ctx.strokeStyle = 'rgba(0, 240, 255, 0.75)';
      ctx.lineWidth = 1.5;

      // Center crosshairs
      ctx.beginPath();
      ctx.arc(0, 0, 36, 0, Math.PI * 2);
      ctx.stroke();

      ctx.beginPath();
      ctx.moveTo(-50, 0);
      ctx.lineTo(-12, 0);
      ctx.moveTo(12, 0);
      ctx.lineTo(50, 0);
      ctx.moveTo(0, -50);
      ctx.lineTo(0, -12);
      ctx.moveTo(0, 12);
      ctx.lineTo(0, 50);
      ctx.stroke();

      // Pitch / Roll pitch ladder
      ctx.strokeStyle = 'rgba(255, 255, 255, 0.5)';
      ctx.lineWidth = 1;
      ctx.beginPath();
      ctx.moveTo(-35, -20);
      ctx.lineTo(-20, -20);
      ctx.moveTo(20, -20);
      ctx.lineTo(35, -20);
      ctx.moveTo(-35, 20);
      ctx.lineTo(-20, 20);
      ctx.moveTo(20, 20);
      ctx.lineTo(35, 20);
      ctx.stroke();

      ctx.restore();

      animationFrameId = requestAnimationFrame(render);
    };

    render();
    return () => cancelAnimationFrame(animationFrameId);
  }, [filterMode, pitch, roll, activeMode]);

  const handleCaptureTrigger = () => {
    setCapturing(true);
    if (onCaptureImage) {
      onCaptureImage();
    }
    setTimeout(() => {
      const newSnap = {
        id: `CAP-${Math.floor(1000 + Math.random() * 9000)}`,
        time: new Date().toUTCString().slice(17, 25) + ' UTC',
        mode: activeMode,
        band: filterMode === 'THERMAL' ? 'THERMAL-IR' : filterMode === 'EDGE' ? 'EDGE-SOBEL' : 'OPTICAL-RGB',
        hotspots: activeMode === 'WILDFIRE' ? 4 : 0,
        metric: activeMode === 'WILDFIRE' ? 'Burn Index: 0.84' : activeMode === 'FLOOD' ? 'Inundation: 4.1 km²' : 'Terrain Displ: 12cm',
        url: filterMode === 'THERMAL' 
          ? 'https://images.unsplash.com/photo-1509198397868-475647b2a1e5?auto=format&fit=crop&w=400&q=80'
          : 'https://images.unsplash.com/photo-1602980085566-480922883f3a?auto=format&fit=crop&w=400&q=80'
      };
      setSnapshots(prev => [newSnap, ...prev.slice(0, 3)]);
      setCapturing(false);
    }, 600);
  };

  return (
    <div className="v-card p-4 space-y-3">
      {/* Header bar */}
      <div className="flex items-center justify-between">
        <div className="flex items-center gap-2">
          <div className="w-7 h-7 rounded-lg bg-blue-500/20 text-blue-400 flex items-center justify-center">
            <Eye className="w-4 h-4" />
          </div>
          <div>
            <div className="flex items-center gap-2">
              <h2 className="text-xs font-bold text-white uppercase tracking-wider">
                AIRBORNE OPTICAL SUITE
              </h2>
              <span className="text-[10px] font-mono text-emerald-400 bg-emerald-500/10 px-1.5 py-0.2 rounded border border-emerald-500/20 flex items-center gap-1">
                <span className="w-1.5 h-1.5 rounded-full bg-emerald-400 animate-ping"></span>
                LIVE DOWNLINK
              </span>
            </div>
            <p className="text-[10px] text-slate-400 font-mono">
              ESP32-CAM (OV2640) • 1600x1200 UXGA • 24 FPS • 14ms EXP
            </p>
          </div>
        </div>

        {/* Spectral Filter Switcher */}
        <div className="flex items-center gap-1 bg-slate-900/80 p-1 rounded-xl border border-white/5 text-xs font-mono">
          <button
            onClick={() => setFilterMode('RGB')}
            className={`px-2.5 py-1 rounded-lg transition font-bold ${
              filterMode === 'RGB'
                ? 'bg-blue-600 text-white shadow-md shadow-blue-500/30'
                : 'text-slate-400 hover:text-white'
            }`}
          >
            RGB Optical
          </button>
          <button
            onClick={() => setFilterMode('THERMAL')}
            className={`px-2.5 py-1 rounded-lg transition font-bold flex items-center gap-1 ${
              filterMode === 'THERMAL'
                ? 'bg-gradient-to-r from-amber-600 to-rose-600 text-white shadow-md shadow-amber-500/30'
                : 'text-slate-400 hover:text-white'
            }`}
          >
            <Flame className="w-3 h-3 text-amber-300" />
            Thermal IR
          </button>
          <button
            onClick={() => setFilterMode('EDGE')}
            className={`px-2.5 py-1 rounded-lg transition font-bold flex items-center gap-1 ${
              filterMode === 'EDGE'
                ? 'bg-cyan-600 text-white shadow-md shadow-cyan-500/30'
                : 'text-slate-400 hover:text-white'
            }`}
          >
            <Scan className="w-3 h-3 text-cyan-300" />
            Edge Vectoring
          </button>
        </div>
      </div>

      {/* Primary Video Feed Display */}
      <div className="relative rounded-xl overflow-hidden border border-white/10 bg-black aspect-video flex items-center justify-center shadow-2xl">
        <canvas
          ref={canvasRef}
          width={640}
          height={360}
          className="w-full h-full object-cover"
        />

        {/* Reticle scanlines overlay */}
        <div className="absolute inset-0 pointer-events-none hud-scanline opacity-40"></div>

        {/* Telemetry Corner Readouts (Aerospace HUD) */}
        <div className="absolute top-3 left-3 font-mono text-[11px] text-cyan-300 bg-slate-950/70 backdrop-blur-md px-2.5 py-1.5 rounded-lg border border-cyan-500/30 space-y-0.5 pointer-events-none">
          <div>FOV: <strong>68.5° AERIAL CONE</strong></div>
          <div>ALT: <strong>{alt.toFixed(0)} m MSL</strong></div>
          <div>HDG: <strong>{heading.toFixed(1)}°</strong></div>
        </div>

        <div className="absolute top-3 right-3 font-mono text-[11px] text-slate-200 bg-slate-950/70 backdrop-blur-md px-2.5 py-1.5 rounded-lg border border-white/10 text-right space-y-0.5 pointer-events-none">
          <div className="text-emerald-400 font-bold">DOWNLINK 1.2 Mbps</div>
          <div>PITCH: <strong className="text-cyan-300">{pitch > 0 ? `+${pitch}` : pitch}°</strong></div>
          <div>ROLL: <strong className="text-cyan-300">{roll > 0 ? `+${roll}` : roll}°</strong></div>
        </div>

        {/* Optical Vectoring Filter Badge */}
        <div className="absolute bottom-3 left-3 font-mono text-[10px] text-slate-300 bg-slate-950/80 px-2.5 py-1 rounded-md border border-white/10 flex items-center gap-2 pointer-events-none">
          <Crosshair className="w-3 h-3 text-cyan-400" />
          <span>FILTER: <strong className="text-white">{filterMode} PIPELINE</strong></span>
          <span>•</span>
          <span>SOBEL: <strong className="text-cyan-300">{filterMode === 'EDGE' ? 'ACTIVE' : 'STANDBY'}</strong></span>
        </div>

        {/* Capture Telecommand Action */}
        <div className="absolute bottom-3 right-3">
          <button
            onClick={handleCaptureTrigger}
            disabled={capturing}
            className="flex items-center gap-2 bg-gradient-to-r from-blue-600 to-cyan-500 hover:from-blue-500 hover:to-cyan-400 text-white text-xs font-bold px-3.5 py-1.5 rounded-xl shadow-lg shadow-blue-500/30 active:scale-95 transition border border-white/20"
          >
            <Camera className={`w-3.5 h-3.5 ${capturing ? 'animate-spin' : ''}`} />
            <span>{capturing ? 'TRANSMITTING...' : 'TRIGGER BURST CAPTURE'}</span>
          </button>
        </div>
      </div>

      {/* Snapshot Downlink Strip */}
      <div>
        <div className="flex items-center justify-between text-[11px] text-slate-400 font-mono mb-2">
          <span className="uppercase font-sans font-bold text-slate-300">Recent Captured Frames (ESP32-CAM Buffer)</span>
          <span>{snapshots.length} STORED IN FLIGHT CATALOG</span>
        </div>

        <div className="grid grid-cols-3 gap-2">
          {snapshots.map((snap) => (
            <div 
              key={snap.id} 
              className="bg-slate-900/70 border border-white/10 rounded-xl p-2 flex flex-col justify-between hover:border-blue-500/40 transition group"
            >
              <div className="flex justify-between items-start mb-1 text-[10px] font-mono">
                <span className="font-bold text-slate-200">{snap.id}</span>
                <span className="text-slate-400">{snap.time}</span>
              </div>
              <div className="text-[11px] font-bold text-cyan-300 truncate">
                {snap.metric}
              </div>
              <div className="flex items-center justify-between text-[9px] text-slate-400 font-mono mt-1 pt-1 border-t border-white/5">
                <span className="text-slate-300">{snap.band}</span>
                <span className="text-emerald-400">SYNCED</span>
              </div>
            </div>
          ))}
        </div>
      </div>
    </div>
  );
}
