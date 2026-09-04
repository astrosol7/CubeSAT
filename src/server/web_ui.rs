pub fn get_tactical_dashboard_html() -> &'static str {
    r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Aegis-Resilience: Tactical Disaster Situational Awareness</title>
    <link rel="stylesheet" href="https://unpkg.com/leaflet@1.9.4/dist/leaflet.css" />
    <script src="https://unpkg.com/leaflet@1.9.4/dist/leaflet.js"></script>
    <style>
        :root {
            --bg-dark: #0a0e17;
            --panel-bg: #111827;
            --panel-border: #1f2937;
            --accent-cyan: #06b6d4;
            --accent-green: #10b981;
            --accent-red: #ef4444;
            --accent-amber: #f59e0b;
            --text-main: #f3f4f6;
            --text-dim: #9ca3af;
        }
        * { box-sizing: border-box; margin: 0; padding: 0; font-family: 'Segoe UI', system-ui, sans-serif; }
        body { background: var(--bg-dark); color: var(--text-main); display: flex; flex-direction: column; height: 100vh; overflow: hidden; }
        header { background: #0f172a; border-bottom: 2px solid var(--accent-cyan); padding: 12px 24px; display: flex; justify-content: space-between; align-items: center; }
        .logo-box { display: flex; align-items: center; gap: 12px; }
        .logo-box h1 { font-size: 1.25rem; font-weight: 700; letter-spacing: 1px; color: #fff; text-transform: uppercase; }
        .badge-live { background: rgba(16, 185, 129, 0.2); color: var(--accent-green); border: 1px solid var(--accent-green); font-size: 0.75rem; padding: 3px 8px; border-radius: 4px; font-weight: 600; text-transform: uppercase; animation: pulse 2s infinite; }
        @keyframes pulse { 0%, 100% { opacity: 1; } 50% { opacity: 0.4; } }
        .main-grid { display: grid; grid-template-columns: 360px 1fr 400px; flex: 1; gap: 12px; padding: 12px; overflow: hidden; }
        .panel { background: var(--panel-bg); border: 1px solid var(--panel-border); border-radius: 8px; display: flex; flex-direction: column; overflow: hidden; }
        .panel-header { background: #1a2234; padding: 10px 14px; font-size: 0.85rem; font-weight: 700; letter-spacing: 0.5px; text-transform: uppercase; color: var(--accent-cyan); border-bottom: 1px solid var(--panel-border); display: flex; justify-content: space-between; align-items: center; }
        .panel-content { padding: 14px; flex: 1; overflow-y: auto; }
        #map { width: 100%; height: 100%; border-radius: 6px; }
        .stat-grid { display: grid; grid-template-columns: 1fr 1fr; gap: 8px; margin-bottom: 14px; }
        .stat-card { background: #162032; border: 1px solid #243048; padding: 8px 10px; border-radius: 6px; }
        .stat-card .label { font-size: 0.7rem; color: var(--text-dim); text-transform: uppercase; }
        .stat-card .value { font-size: 1.1rem; font-weight: 700; color: #fff; margin-top: 2px; }
        .hazard-meter { height: 8px; background: #243048; border-radius: 4px; overflow: hidden; margin-top: 6px; }
        .hazard-fill { height: 100%; width: 0%; transition: width 0.3s ease, background-color 0.3s ease; }
        .btn { background: #1e293b; color: #fff; border: 1px solid #334155; padding: 8px 12px; border-radius: 6px; cursor: pointer; font-size: 0.8rem; font-weight: 600; width: 100%; margin-bottom: 8px; transition: all 0.2s; text-align: left; }
        .btn:hover { background: #334155; border-color: var(--accent-cyan); }
        .btn-fire { border-left: 4px solid var(--accent-red); }
        .btn-flood { border-left: 4px solid var(--accent-cyan); }
        .btn-slide { border-left: 4px solid var(--accent-amber); }
        .btn-quake { border-left: 4px solid #a855f7; }
        .btn-danger { background: rgba(239, 68, 68, 0.2); border-color: var(--accent-red); color: #fca5a5; text-align: center; }
        .btn-danger:hover { background: rgba(239, 68, 68, 0.4); }
        .alert-box { background: rgba(239, 68, 68, 0.15); border: 1px solid var(--accent-red); border-radius: 6px; padding: 10px; margin-bottom: 12px; font-size: 0.85rem; }
        .alert-box.nominal { background: rgba(16, 185, 129, 0.15); border-color: var(--accent-green); }
        .bulletin-card { background: #162032; border-left: 4px solid var(--accent-cyan); padding: 10px; border-radius: 4px; margin-bottom: 8px; font-size: 0.8rem; line-height: 1.4; }
        .lora-box { background: #000; font-family: monospace; font-size: 0.75rem; color: #10b981; padding: 8px; border-radius: 4px; word-break: break-all; margin-top: 6px; }
    </style>
</head>
<body>
    <header>
        <div class="logo-box">
            <h1>AEGIS-SENTINEL // RAPID DISASTER SITUATIONAL AWARENESS</h1>
            <span class="badge-live" id="sys-status-badge">ONLINE // ACTIVE RECON</span>
        </div>
        <div style="font-size: 0.8rem; color: var(--text-dim);" id="clock">UTC 00:00:00</div>
    </header>

    <div class="main-grid">
        <!-- LEFT PANEL: CONTROLS & BASES -->
        <div class="panel">
            <div class="panel-header">Strategic Dispatch & Mission Controls</div>
            <div class="panel-content">
                <p style="font-size: 0.75rem; color: var(--text-dim); margin-bottom: 10px;">Select a simulated disaster scenario to trigger strategic base optimization, nearest platform dispatch, and automatic mode transition:</p>
                <button class="btn btn-fire" onclick="triggerIncident('Wildfire', 39.965, 32.890, 0.88)">🔥 Scenario 1: Wildfire Flash Front (Ridge Pass)</button>
                <button class="btn btn-flood" onclick="triggerIncident('Flood', 39.735, 32.720, 0.92)">🌊 Scenario 2: Flash Flood & Bridge Inundation</button>
                <button class="btn btn-slide" onclick="triggerIncident('Landslide', 39.995, 33.120, 0.79)">⛰️ Scenario 3: Canyon Highway Landslide</button>
                <button class="btn btn-quake" onclick="triggerIncident('Earthquake', 39.930, 32.845, 0.85)">🏚️ Scenario 4: Urban Earthquake Debris Screening</button>
                
                <hr style="border-color: #1f2937; margin: 14px 0;">
                <div class="panel-header" style="background: transparent; padding: 0 0 8px 0;">Strategic Deployment Bases</div>
                <div id="bases-list" style="font-size: 0.8rem;"></div>

                <hr style="border-color: #1f2937; margin: 14px 0;">
                <button class="btn btn-danger" id="failsafe-btn" onclick="toggleFailsafe()">⚠️ Trigger Off-Grid RF Loss-of-Signal</button>
                <button class="btn" style="text-align: center; border-color: #475569;" onclick="returnToBase()">🔄 Command Return to Base</button>
            </div>
        </div>

        <!-- CENTER PANEL: LIVE GIS TACTICAL MAP -->
        <div class="panel">
            <div class="panel-header">
                <span>Tactical Geospatial Operations Map</span>
                <span id="platform-pos" style="color: var(--text-dim); font-size: 0.75rem;">GPS: FIX 3D</span>
            </div>
            <div class="panel-content" style="padding: 0;">
                <div id="map"></div>
            </div>
        </div>

        <!-- RIGHT PANEL: TELEMETRY & DECISION SUPPORT -->
        <div class="panel">
            <div class="panel-header">Platform Telemetry & Incident Intelligence</div>
            <div class="panel-content">
                <div class="alert-box" id="route-alert-box">
                    <strong id="route-alert-title">CORRIDOR STATUS</strong>
                    <div id="route-alert-text" style="margin-top: 4px; font-size: 0.75rem;">Analyzing access artery...</div>
                </div>

                <div class="stat-grid">
                    <div class="stat-card">
                        <div class="label">Mission Mode</div>
                        <div class="value" id="mode-val" style="color: var(--accent-cyan);">STANDBY</div>
                    </div>
                    <div class="stat-card">
                        <div class="label">Platform State</div>
                        <div class="value" id="state-val">STANDBY</div>
                    </div>
                    <div class="stat-card">
                        <div class="label">Battery SoC</div>
                        <div class="value" id="battery-val">95%</div>
                    </div>
                    <div class="stat-card">
                        <div class="label">LoRa RF Link</div>
                        <div class="value" id="lora-val">-78 dBm</div>
                    </div>
                </div>

                <div style="margin-bottom: 12px;">
                    <div style="display: flex; justify-content: space-between; font-size: 0.75rem;">
                        <span style="color: var(--text-dim);">SITUATIONAL HAZARD INDEX</span>
                        <strong id="hazard-val">0.0 / 10.0</strong>
                    </div>
                    <div class="hazard-meter">
                        <div class="hazard-fill" id="hazard-bar"></div>
                    </div>
                </div>

                <div class="panel-header" style="background: transparent; padding: 4px 0 8px 0;">Tactical Decision Directives</div>
                <div id="bulletin-list"></div>

                <div class="panel-header" style="background: transparent; padding: 8px 0 4px 0;">Compact Off-Grid LoRa Binary Frame</div>
                <div class="lora-box" id="lora-hex">SAT 0001 00 00 01 ... [32 BYTES COMPACT]</div>
            </div>
        </div>
    </div>

    <script>
        setInterval(() => {
            document.getElementById('clock').innerText = 'UTC ' + new Date().toISOString().substring(11, 19);
        }, 1000);

        const map = L.map('map').setView([39.92, 32.85], 11);
        L.tileLayer('https://{s}.tile.openstreetmap.org/{z}/{x}/{y}.png', {
            attribution: 'Map &copy; OpenStreetMap contributors'
        }).addTo(map);

        let podMarker = null;
        let targetCircle = null;
        let baseMarkers = [];

        async function fetchStatus() {
            try {
                const res = await fetch('/api/status');
                const data = await res.json();
                updateUI(data);
            } catch (err) {
                console.error("Telemetry sync error:", err);
            }
        }

        function updateUI(data) {
            const p = data.platform;
            document.getElementById('mode-val').innerText = p.active_mode;
            document.getElementById('state-val').innerText = p.state;
            document.getElementById('battery-val').innerText = p.power.battery_soc_pct.toFixed(0) + '%';
            document.getElementById('lora-val').innerText = p.comms.rssi_dbm + ' dBm';
            document.getElementById('platform-pos').innerText = `Lat ${p.current_coords.lat.toFixed(4)}, Lon ${p.current_coords.lon.toFixed(4)}, Alt ${p.current_coords.alt_meters.toFixed(0)}m`;

            // Hazard Bar
            const h = p.last_hazard_index;
            document.getElementById('hazard-val').innerText = `${h.toFixed(1)} / 10.0`;
            const hbar = document.getElementById('hazard-bar');
            hbar.style.width = (h * 10) + '%';
            hbar.style.backgroundColor = h > 7.0 ? '#ef4444' : (h > 4.0 ? '#f59e0b' : '#10b981');

            // Route Alert Box
            const rbox = document.getElementById('route-alert-box');
            const rtitle = document.getElementById('route-alert-title');
            const rtext = document.getElementById('route-alert-text');
            if (p.last_route_blocked) {
                rbox.className = 'alert-box';
                rtitle.innerText = 'CRITICAL CORRIDOR CUTOFF';
                rtext.innerText = p.last_route_status;
            } else {
                rbox.className = 'alert-box nominal';
                rtitle.innerText = 'EVACUATION ROUTE PASSABLE';
                rtext.innerText = p.last_route_status;
            }

            // Map Update
            if (!podMarker) {
                podMarker = L.circleMarker([p.current_coords.lat, p.current_coords.lon], {
                    radius: 9, fillColor: '#06b6d4', color: '#fff', weight: 2, fillOpacity: 0.9
                }).addTo(map).bindPopup('<b>POD-SENTINEL-01</b><br>State: ' + p.state);
            } else {
                podMarker.setLatLng([p.current_coords.lat, p.current_coords.lon]);
                podMarker.getPopup().setContent(`<b>POD-SENTINEL-01</b><br>Mode: ${p.active_mode}<br>Speed: ${p.speed_kmh.toFixed(1)} km/h`);
            }

            // Strategic Bases render
            const blist = document.getElementById('bases-list');
            blist.innerHTML = '';
            data.bases.forEach(b => {
                blist.innerHTML += `
                    <div style="margin-bottom: 6px; padding: 6px; background: #162032; border-radius: 4px;">
                        <div style="font-weight: 600; color: #06b6d4;">${b.name}</div>
                        <div style="font-size: 0.7rem; color: #9ca3af;">Radius: ${b.max_operational_radius_km} km | Stationed: ${b.stationed_platform_ids.join(', ')}</div>
                    </div>
                `;
            });

            // Decision Bulletin List
            const bl = document.getElementById('bulletin-list');
            if (data.latest_bulletin && data.latest_bulletin.critical_decision_recommendations) {
                bl.innerHTML = data.latest_bulletin.critical_decision_recommendations.map(r => `
                    <div class="bulletin-card">⚡ ${r}</div>
                `).join('');
            }

            // LoRa frame hex
            if (data.latest_lora_hex) {
                document.getElementById('lora-hex').innerText = data.latest_lora_hex;
            }

            // Failsafe button status
            const fbtn = document.getElementById('failsafe-btn');
            if (p.failsafe.is_autonomous_mode) {
                fbtn.innerText = '⚠️ RF LINK SEVERED (AUTONOMOUS DEGRADED HOLD)';
                fbtn.style.background = 'rgba(239, 68, 68, 0.6)';
            } else {
                fbtn.innerText = '⚠️ Trigger Off-Grid RF Loss-of-Signal';
                fbtn.style.background = 'rgba(239, 68, 68, 0.2)';
            }
        }

        async function triggerIncident(disaster, lat, lon, sev) {
            if (targetCircle) map.removeLayer(targetCircle);
            targetCircle = L.circle([lat, lon], {
                radius: 1200, color: '#ef4444', fillColor: '#ef4444', fillOpacity: 0.3
            }).addTo(map);

            await fetch('/api/dispatch', {
                method: 'POST',
                headers: { 'Content-Type': 'application/json' },
                body: JSON.stringify({ disaster_type: disaster, lat: lat, lon: lon, severity: sev })
            });
            fetchStatus();
        }

        async function toggleFailsafe() {
            await fetch('/api/failsafe', { method: 'POST' });
            fetchStatus();
        }

        async function returnToBase() {
            await fetch('/api/return', { method: 'POST' });
            fetchStatus();
        }

        setInterval(fetchStatus, 1000);
        fetchStatus();
    </script>
</body>
</html>
"#
}
