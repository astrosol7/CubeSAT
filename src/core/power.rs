use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PowerState {
    Nominal,
    Conservation,
    CriticalShedding,
    DepletedEmergency,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PowerSubsystem {
    pub battery_voltage_v: f32,
    pub battery_soc_pct: f32,
    pub power_draw_w: f32,
    pub power_state: PowerState,
    pub solar_charging_w: f32,
}

impl Default for PowerSubsystem {
    fn default() -> Self {
        Self {
            battery_voltage_v: 12.4, // 3S Li-ion nominal
            battery_soc_pct: 95.0,
            power_draw_w: 4.5,
            power_state: PowerState::Nominal,
            solar_charging_w: 0.0,
        }
    }
}

impl PowerSubsystem {
    pub fn update(&mut self, elapsed_secs: f32, active_camera: bool, active_tx: bool) {
        let mut draw = 2.2; // Base OBC + IMU + GNSS
        if active_camera {
            draw += 1.8;
        }
        if active_tx {
            draw += 1.4;
        }
        self.power_draw_w = draw;

        let net_draw_w = (self.power_draw_w - self.solar_charging_w).max(0.2);
        // Assuming a 3S 3000mAh pack ~ 33.3 Watt-hours
        let capacity_wh = 33.3;
        let consumed_wh = (net_draw_w * elapsed_secs) / 3600.0;
        let soc_delta = (consumed_wh / capacity_wh) * 100.0;
        self.battery_soc_pct = (self.battery_soc_pct - soc_delta).clamp(0.0, 100.0);

        // Convert SoC to approximate voltage curve
        self.battery_voltage_v = 9.6 + (self.battery_soc_pct / 100.0) * (12.6 - 9.6);

        self.power_state = if self.battery_soc_pct > 40.0 {
            PowerState::Nominal
        } else if self.battery_soc_pct > 20.0 {
            PowerState::Conservation
        } else if self.battery_soc_pct > 5.0 {
            PowerState::CriticalShedding
        } else {
            PowerState::DepletedEmergency
        };
    }

    pub fn is_camera_allowed(&self) -> bool {
        match self.power_state {
            PowerState::Nominal => true,
            PowerState::Conservation => true, // reduced framerate
            PowerState::CriticalShedding | PowerState::DepletedEmergency => false,
        }
    }
}
