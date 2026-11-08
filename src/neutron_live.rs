//! Live preview; approved static shader remains available unchanged.
pub use crate::neutron_viewer::LiveRenderer;
#[derive(Clone, Copy, Debug, Default)]
pub struct Controls {
    pub orbit_x: f64,
    pub orbit_y: f64,
    pub zoom: f64,
    pub toggle_pause: bool,
    pub reset: bool,
}
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LiveState {
    initial_angle: f64,
    time: f64,
    paused: bool,
    yaw: f64,
    pitch: f64,
    distance: f64,
}
impl LiveState {
    pub fn new(initial_angle: f64) -> Result<Self, String> {
        if !initial_angle.is_finite() {
            return Err("El angulo debe ser finito".into());
        }
        Ok(Self {
            initial_angle,
            time: 0.0,
            paused: false,
            yaw: 0.0,
            pitch: 0.0,
            distance: 1.0,
        })
    }
    pub fn time(&self) -> f64 {
        self.time
    }
    pub fn angle(&self) -> f64 {
        self.initial_angle + self.time.rem_euclid(std::f64::consts::TAU / 0.42) * 0.42
    }
    pub fn orbit(&self) -> (f64, f64) {
        (self.yaw, self.pitch)
    }
    pub fn distance_scale(&self) -> f64 {
        self.distance
    }
    pub fn paused(&self) -> bool {
        self.paused
    }
    pub fn advance(&mut self, dt: f64, input: Controls) -> Result<(), String> {
        if !dt.is_finite() || dt < 0.0 || !(self.time + dt).is_finite() {
            return Err("dt invalido o tiempo desbordado".into());
        }
        if [input.orbit_x, input.orbit_y, input.zoom]
            .iter()
            .any(|x| !x.is_finite() || x.abs() > 1.0)
        {
            return Err("Controles invalidos".into());
        }
        if input.reset {
            *self = Self::new(self.initial_angle)?;
            return Ok(());
        }
        let yaw = self.yaw + input.orbit_x * 0.9 * dt;
        let pitch = self.pitch + input.orbit_y * 0.9 * dt;
        let distance = self.distance * (-input.zoom * 0.8 * dt).exp();
        if !yaw.is_finite() || !pitch.is_finite() || !distance.is_finite() {
            return Err("Movimiento de camara desbordado".into());
        }
        let mut next = *self;
        if input.toggle_pause {
            next.paused = !next.paused;
        }
        if !next.paused {
            next.time += dt;
        }
        next.yaw = yaw.rem_euclid(std::f64::consts::TAU);
        next.pitch = pitch.clamp(-1.2, 1.2);
        next.distance = distance.clamp(0.85, 2.4);
        *self = next;
        Ok(())
    }
}

pub fn pulse_at(time: f64) -> Result<f64, String> {
    if !time.is_finite() || time < 0.0 {
        return Err("Tiempo invalido".into());
    }
    let phase = time.rem_euclid(3.6) / 3.6;
    Ok((1.0 - 0.80 * (0.5 - 0.5 * (std::f64::consts::TAU * phase).cos())).clamp(0.20, 1.0))
}
