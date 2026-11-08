//! Seven-view presentation; individual renderer modules remain unchanged.
use crate::{
    neutron_live::{Controls, LiveState},
    pulsar::PulsarRenderer,
    renderer::framebuffer::Framebuffer,
};
pub struct Presentation {
    state: State,
    pulsar: PulsarRenderer,
    static_frame: Option<Framebuffer>,
    w: usize,
    h: usize,
    angle: f64,
    layers: u8,
    live: LiveState,
    initial_time: f64,
}
impl Presentation {
    pub fn new(
        w: usize,
        h: usize,
        angle: f64,
        time: f64,
        layers: u8,
        index: usize,
    ) -> Result<Self, String> {
        let state = State::new(index)?;
        let mut live = LiveState::new(angle)?;
        live.advance(time, Controls::default())?;
        let mut pulsar = PulsarRenderer::new(w, h, layers)?;
        pulsar.render(&live)?;
        let static_frame = Self::render_static(w, h, angle, layers, index)?;
        Ok(Self {
            state,
            pulsar,
            static_frame,
            w,
            h,
            angle,
            layers,
            live,
            initial_time: time,
        })
    }
    fn render_static(
        w: usize,
        h: usize,
        angle: f64,
        layers: u8,
        index: usize,
    ) -> Result<Option<Framebuffer>, String> {
        use crate::extras::{self, Body};
        let frame = match index {
            0 => return Ok(None),
            1 => crate::eyeball::render(w, h, angle, layers)?.0,
            2 => crate::gas_v2::render(w, h, angle, layers)?.0,
            3 => extras::render(w, h, angle, layers, Body::Volcanic, false)?.0,
            4 => extras::render(w, h, angle, layers, Body::Desert, true)?.0,
            5 => extras::render(w, h, angle, layers, Body::Temperate, false)?.0,
            _ => crate::render_neutron(w, h, angle, layers)?.0,
        };
        Ok(Some(frame))
    }
    pub fn state(&self) -> State {
        self.state
    }
    pub fn frame(&self) -> &Framebuffer {
        if self.state.index() == 0 {
            self.pulsar.framebuffer()
        } else {
            self.static_frame.as_ref().expect("validated static slide")
        }
    }
    pub fn advance(&mut self, dt: f64, input: Input) -> Result<(), String> {
        let mut next = self.state;
        next.advance(dt, input)?;
        let mut live = self.live;
        if input.reset {
            live = LiveState::new(self.angle)?;
            live.advance(self.initial_time, Controls::default())?;
        } else if self.state.index() == 0 && next.index() == 0 && !next.paused() {
            live.advance(dt, Controls::default())?;
        }
        if next.index() != self.state.index() {
            let frame = Self::render_static(self.w, self.h, self.angle, self.layers, next.index())?;
            self.static_frame = frame;
        }
        if next.index() == 0 && (live != self.live || self.state.index() != 0 || input.reset) {
            self.pulsar.render(&live)?;
        }
        self.live = live;
        self.state = next;
        Ok(())
    }
}
pub const LABELS: [&str; 7] = [
    "Pulsar",
    "LHS 1140 b | oceano-ojo",
    "Gigante gaseoso turbulento",
    "Volcanico",
    "Desertico + luna",
    "Templado",
    "Estrella base",
];
pub const SLIDE_SECONDS: f64 = 8.0;
#[derive(Clone, Copy, Debug, Default)]
pub struct Input {
    pub next: bool,
    pub previous: bool,
    pub toggle_pause: bool,
    pub reset: bool,
}
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct State {
    index: usize,
    elapsed: f64,
    paused: bool,
}
impl State {
    pub fn new(index: usize) -> Result<Self, String> {
        if index >= LABELS.len() {
            return Err("Diapositiva fuera de rango".into());
        }
        Ok(Self {
            index,
            elapsed: 0.0,
            paused: false,
        })
    }
    pub fn index(&self) -> usize {
        self.index
    }
    pub fn paused(&self) -> bool {
        self.paused
    }
    pub fn advance(&mut self, dt: f64, input: Input) -> Result<(), String> {
        if !dt.is_finite() || dt < 0.0 {
            return Err("dt invalido".into());
        }
        if input.reset {
            *self = Self::new(0)?;
            return Ok(());
        }
        if input.toggle_pause {
            self.paused = !self.paused;
        }
        if input.next != input.previous {
            self.index =
                (self.index + if input.next { 1 } else { LABELS.len() - 1 }) % LABELS.len();
            self.elapsed = 0.0;
            return Ok(());
        }
        if self.paused {
            return Ok(());
        }
        let total = self.elapsed + dt.rem_euclid(SLIDE_SECONDS * LABELS.len() as f64);
        self.index = (self.index + (total / SLIDE_SECONDS).floor() as usize) % LABELS.len();
        self.elapsed = total.rem_euclid(SLIDE_SECONDS);
        Ok(())
    }
}
