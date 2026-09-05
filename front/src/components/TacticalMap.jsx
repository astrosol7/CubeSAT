import React, { useState } from 'react';
import { 
  MapPin, 
  Navigation, 
  Layers, 
  Radio, 
  Wind, 
  ShieldCheck, 
  AlertTriangle,
  LocateFixed
} from 'lucide-react';

export default function TacticalMap({ telemetry, activeMode }) {
  const [showNodes, setShowNodes] = useState(true);
  const [showPerimeter, setShowPerimeter] = useState(true);
  const [showEvacRoute, setShowEvacRoute] = useState(true);

  const lat = telemetry?.navigation?.latitude || 34.1985;
  const lon = telemetry?.navigation?.longitude || -118.1750;
  const alt = telemetry?.navigation?.altitude_msl_m || 1240;

  // Mode specific disaster epicenter info
  const getDisasterDetails = () => {
    switch (activeMode) {
      case 'WILDFIRE':
        return {
          title: 'ACTIVE FIRE PERIMETER #W-44',
          coords: '34.2045° N, -118.1680° W',
          area: '14.8 km²',
          hazard: 'High Thermal Radiant Flux',
          color: 'stroke-amber-500 fill-amber-500/20'
        };
      case 'FLOOD':
        return {
          title: 'INUNDATION BASIN OVERFLOW #FL-12',
          coords: '34.1950° N, -118.1820° W',
          area: '22.3 km²',
          hazard: 'Bridge Inundation & Road Blockage',
          color: 'stroke-cyan-500 fill-cyan-500/20'
        };
      case 'LANDSLIDE':
        return {
          title: 'SLOPE DEFORMATION ZONE #LS-08',
          coords: '34.2010° N, -118.1790° W',
          area: '3.1 km²',
          hazard: 'Highway 2 Route Severance',
          color: 'stroke-emerald-500 fill-emerald-500/20'
        };
      case 'EARTHQUAKE':
        return {
          title: 'POST-SEISMIC FAULT LINE #EQ-02',
          coords: '34.1970° N, -118.1730° W',
          area: 'Radius 18 km',
          hazard: 'Structural Structural Shear Inspection',
          color: 'stroke-rose-500 fill-rose-500/20'
        };
      default:
        return {
          title: 'OBSERVATION CORRIDOR',
          coords: '34.1985° N, -118.1750° W',
          area: '10.0 km²',
          hazard: 'None',
          color: 'stroke-blue-500 fill-blue-500/20'
        };
    }
  };

  const disaster = getDisasterDetails();

  return (
    <div className="v-card p-4 space-y-3">
      {/* Header & Tactical Layer Selectors */}
      <div className="flex flex-wrap items-center justify-between gap-2">
        <div className="flex items-center gap-2">
          <div className="w-7 h-7 rounded-lg bg-emerald-500/20 text-emerald-400 flex items-center justify-center">
            <Navigation className="w-4 h-4" />
          </div>
          <div>
            <h2 className="text-xs font-bold text-white uppercase tracking-wider">
              TACTICAL GEOSPATIAL MAP
            </h2>
            <p className="text-[10px] text-slate-400 font-mono">
              WGS-84 Projection • Real-time Aerial Trajectory & Exclusion Zones
            </p>
          </div>
        </div>

        {/* Map Layer Toggles */}
        <div className="flex items-center gap-1.5 text-[11px] font-mono">
          <button
            onClick={() => setShowPerimeter(!showPerimeter)}
            className={`px-2 py-0.5 rounded border transition ${
              showPerimeter 
                ? 'bg-amber-500/20 text-amber-300 border-amber-500/40' 
                : 'bg-slate-900/60 text-slate-500 border-white/5'
            }`}
          >
            Hazard Zone
          </button>
          <button
            onClick={() => setShowNodes(!showNodes)}
            className={`px-2 py-0.5 rounded border transition ${
              showNodes 
                ? 'bg-cyan-500/20 text-cyan-300 border-cyan-500/40' 
                : 'bg-slate-900/60 text-slate-500 border-white/5'
            }`}
          >
            Field Nodes
          </button>
          <button
            onClick={() => setShowEvacRoute(!showEvacRoute)}
            className={`px-2 py-0.5 rounded border transition ${
              showEvacRoute 
                ? 'bg-emerald-500/20 text-emerald-300 border-emerald-500/40' 
                : 'bg-slate-900/60 text-slate-500 border-white/5'
            }`}
          >
            Evac Corridor
          </button>
        </div>
      </div>

      {/* SVG Tactical Vector Map */}
      <div className="relative w-full h-56 rounded-xl overflow-hidden border border-white/10 bg-slate-950 flex items-center justify-center shadow-inner">
        <svg className="w-full h-full" viewBox="0 0 600 240">
          <defs>
            {/* Grid pattern */}
            <pattern id="tacticalGrid" width="40" height="40" patternUnits="userSpaceOnUse">
              <path d="M 40 0 L 0 0 0 40" fill="none" stroke="rgba(255, 255, 255, 0.05)" strokeWidth="1" />
            </pattern>
            {/* Radial glow for hazard */}
            <radialGradient id="hazardGlow">
              <stop offset="0%" stopColor="rgba(255, 120, 0, 0.35)" />
              <stop offset="100%" stopColor="rgba(255, 120, 0, 0)" />
            </radialGradient>
          </defs>

          {/* Background grid */}
          <rect width="600" height="240" fill="url(#tacticalGrid)" />

          {/* Contour topographic elevation rings */}
          <path d="M 50 180 Q 150 120 280 160 T 520 140" fill="none" stroke="rgba(255, 255, 255, 0.08)" strokeWidth="1.5" strokeDasharray="4 4" />
          <path d="M 80 80 Q 220 50 360 90 T 580 70" fill="none" stroke="rgba(255, 255, 255, 0.08)" strokeWidth="1.5" strokeDasharray="4 4" />

          {/* Evacuation Route Corridor */}
          {showEvacRoute && (
            <g>
              <path 
                d="M 60 210 L 160 170 L 250 190 L 370 120 L 520 110" 
                fill="none" 
                stroke="#01b574" 
                strokeWidth="3" 
                strokeDasharray="6 4"
              />
              <text x="70" y="225" fill="#01b574" fontSize="10" fontFamily="monospace" fontWeight="bold">
                ROUTE-12 SECURE (EVACUATION ACCESS)
              </text>
            </g>
          )}

          {/* Active Disaster Hazard Perimeter Polygon */}
          {showPerimeter && (
            <g>
              <polygon
                points="310,60 420,50 470,120 390,160 300,130"
                className={`${disaster.color} transition-all duration-700`}
                strokeWidth="2"
              />
              <circle cx="380" cy="100" r="45" fill="url(#hazardGlow)" />
              <text x="330" y="105" fill="#ffffff" fontSize="10" fontFamily="monospace" fontWeight="bold">
                {activeMode} ZONE
              </text>
            </g>
          )}

          {/* Strategic Deployment Base Origin */}
          <g transform="translate(110, 160)">
            <circle cx="0" cy="0" r="10" fill="none" stroke="#0075ff" strokeWidth="2" />
            <circle cx="0" cy="0" r="4" fill="#0075ff" />
            <text x="14" y="4" fill="#60a5fa" fontSize="9" fontFamily="monospace" fontWeight="bold">
              BASE-BRAVO (ORIGIN)
            </text>
          </g>

          {/* Balloon Trajectory Line */}
          <path
            d="M 110 160 Q 180 140 240 110 T 320 85"
            fill="none"
            stroke="#00f2ff"
            strokeWidth="2"
            strokeDasharray="3 3"
          />

          {/* Current CubeSat Airborne Position Marker */}
          <g transform="translate(320, 85)">
            <circle cx="0" cy="0" r="16" fill="none" stroke="#00f2ff" strokeWidth="1.5" className="animate-ping" opacity="0.6" />
            <circle cx="0" cy="0" r="8" fill="#0075ff" stroke="#ffffff" strokeWidth="2" />
            <polygon points="0,-12 4,-6 -4,-6" fill="#00f2ff" />
            <text x="14" y="-2" fill="#00f2ff" fontSize="11" fontFamily="monospace" fontWeight="bold">
              CUBESAT-2U ({alt.toFixed(0)}m)
            </text>
          </g>

          {/* Ground Field Sensor Nodes */}
          {showNodes && (
            <g>
              {/* Node 1 */}
              <g transform="translate(290, 150)">
                <circle cx="0" cy="0" r="4" fill="#ffb547" />
                <text x="8" y="3" fill="#fbbf24" fontSize="9" fontFamily="monospace">GN-01 (Flame/Temp)</text>
              </g>
              {/* Node 2 */}
              <g transform="translate(420, 140)">
                <circle cx="0" cy="0" r="4" fill="#22d3ee" />
                <text x="8" y="3" fill="#38bdf8" fontSize="9" fontFamily="monospace">GN-02 (Water Depth)</text>
              </g>
              {/* Node 3 */}
              <g transform="translate(340, 45)">
                <circle cx="0" cy="0" r="4" fill="#a855f7" />
                <text x="8" y="3" fill="#c084fc" fontSize="9" fontFamily="monospace">GN-03 (Geophone)</text>
              </g>
            </g>
          )}

          {/* Wind Vector Direction */}
          <g transform="translate(540, 40)">
            <circle cx="0" cy="0" r="18" fill="rgba(15, 23, 42, 0.8)" stroke="rgba(255, 255, 255, 0.2)" strokeWidth="1" />
            <line x1="0" y1="10" x2="0" y2="-10" stroke="#ffb547" strokeWidth="2" transform="rotate(42)" />
            <polygon points="0,-12 4,-6 -4,-6" fill="#ffb547" transform="rotate(42)" />
            <text x="-12" y="28" fill="#cbd5e1" fontSize="9" fontFamily="monospace">WIND 14kt</text>
          </g>
        </svg>

        {/* Map Coordinates Legend Overlay */}
        <div className="absolute bottom-2 left-2 bg-slate-900/80 backdrop-blur-md px-2 py-1 rounded border border-white/10 text-[10px] font-mono text-slate-300 flex items-center gap-3">
          <span>LAT: <strong className="text-white">{lat.toFixed(5)}°N</strong></span>
          <span>LON: <strong className="text-white">{lon.toFixed(5)}°W</strong></span>
          <span>GRID: <strong>UTM 11S</strong></span>
        </div>

        {/* Tactical Status Banner */}
        <div className="absolute top-2 left-2 bg-slate-900/80 backdrop-blur-md px-2.5 py-1 rounded border border-white/10 text-[10px] font-mono text-slate-300">
          <div className="font-bold text-white uppercase">{disaster.title}</div>
          <div className="text-[9px] text-slate-400">IMPACT: {disaster.area} • {disaster.hazard}</div>
        </div>
      </div>
    </div>
  );
}
