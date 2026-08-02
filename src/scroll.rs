use crate::*;

#[derive(Debug, Clone)]
pub struct ScrollState {
    pub max_scroll: i32,
    pub content_height: i32,
    pub scrolled: bool,
    /// 1 up, -1 down.
    pub direction: i32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Gesture {
    #[default]
    Idle,
    Active,
    Momentum,
    Bouncing,
}

/// Pixels per unit of wheel notch.
pub const WHEEL_STEP: f32 = 100.0;
pub const RUBBER_BAND_STIFFNESS: f32 = 20.0;
pub const RELEASE_AMPLITUDE: f32 = 0.31;
pub const RELEASE_PERIOD: f32 = 1.6;
/// Overscroll below this never engages the band, so a stray sideways twitch cannot bounce.
pub const ENGAGE: f32 = 10.0;
pub const VELOCITY_TIMEOUT: f32 = 0.1;
/// Time a discrete wheel notch takes to animate, and the curve it follows.
pub const WHEEL_DURATION: f32 = 9.0 / 60.0;
pub const WHEEL_CURVE: (f32, f32, f32, f32) = (0.42, 0.0, 0.58, 1.0);
/// Middle click autoscroll, speed of `distance ^ AUTOSCROLL_EXPONENT * AUTOSCROLL_SPEED`.
pub const AUTOSCROLL_DEAD_ZONE: f32 = 15.0;
pub const AUTOSCROLL_EXPONENT: f32 = 1.5;
pub const AUTOSCROLL_SPEED: f32 = 1.44;

#[derive(Debug, Clone, Copy, Default)]
pub struct Scroll {
    /// Position within the content, always inside `0..=max_scroll`.
    pub offset: f32,
    /// Overscroll displacement past an edge, negative past the top. Zero when not stretched.
    pub stretch: f32,
    /// Finger speed in pixels per second, measured between events.
    pub velocity: f32,
    pub gesture: Gesture,
    /// Raw finger travel past the edge. `stretch` is this divided by `RUBBER_BAND_STIFFNESS`.
    pub accumulated: f32,
    /// Overscroll too small to have engaged the band yet.
    pub pending: f32,
    pub elapsed: f32,
    pub initial_stretch: f32,
    pub initial_velocity: f32,
    /// Where a discrete wheel is animating to, and the curve taking it there.
    pub wheel_target: f32,
    pub wheel_start: f32,
    pub wheel_slope: f32,
    pub wheel_elapsed: f32,
    pub last_timestamp: Option<f64>,
    /// Where the middle button went down, while it is still held.
    pub anchor: Option<i32>,
}

impl Scroll {
    pub fn new() -> Self {
        let mut scroll = Self::default();
        scroll.wheel_elapsed = f32::MAX;
        scroll
    }

    pub fn jump(&mut self, offset: f32) {
        self.offset = offset;
        self.stretch = 0.0;
        self.velocity = 0.0;
        self.gesture = Gesture::Idle;
        self.accumulated = 0.0;
        self.pending = 0.0;
        self.wheel_target = offset;
        self.wheel_elapsed = f32::MAX;
        self.anchor = None;
    }

    /// Holding the middle button scrolls at a speed based on distance from an anchor.
    /// Returns which way it is travelling while held, zero inside the dead zone.
    pub fn autoscroll(&mut self, mouse_y: i32, held: bool, max: f32, dt: f32) -> Option<i32> {
        if !held {
            self.anchor = None;
            return None;
        }

        let anchor = self.anchor?;
        let distance = (mouse_y - anchor) as f32;
        if distance.abs() <= AUTOSCROLL_DEAD_ZONE {
            return Some(0);
        }

        // Past the dead zone the whole distance counts, so speed steps up rather than easing in.
        let speed = distance.abs().powf(AUTOSCROLL_EXPONENT) * AUTOSCROLL_SPEED;
        self.offset = (self.offset + speed * distance.signum() * dt).clamp(0.0, max);
        // The pointer owns the position now, so retire any wheel animation still running.
        self.wheel_target = self.offset;
        self.wheel_elapsed = f32::MAX;

        Some(distance.signum() as i32)
    }

    /// Advances one frame of rubber-band overscroll. Returns whether anything is still moving,
    /// which has to keep the frame clock running or the animations stall.
    /// While the fingers are down the stretch is a pure function of accumulated travel.
    /// The spring is a separate one-shot animation seeded at the moment the gesture ends.
    pub fn elastic(&mut self, events: &[ScrollEvent], hovered: bool, max: f32, dt: f32) -> bool {
        for event in events {
            if !hovered {
                continue;
            }

            // Fingers back on the trackpad take over immediately.
            // Whatever is left of the previous fling is stale.
            if self.gesture == Gesture::Active && event.phase.momentum() {
                continue;
            }

            let mut delta = -event.delta.1 as f32;
            if !event.precise {
                delta *= WHEEL_STEP;
            }

            if !event.phase.ended() {
                let gap = self
                    .last_timestamp
                    .map(|last| (event.timestamp - last) as f32)
                    .unwrap_or(f32::MAX);
                self.velocity = if gap > 0.0 && gap < VELOCITY_TIMEOUT {
                    delta / gap
                } else {
                    0.0
                };
                self.last_timestamp = Some(event.timestamp);
            }

            match event.phase {
                ScrollPhase::Began => {
                    self.gesture = Gesture::Active;
                    self.velocity = 0.0;
                    self.pending = 0.0;
                }
                ScrollPhase::MomentumBegan => {
                    if self.gesture != Gesture::Bouncing {
                        self.gesture = Gesture::Momentum;
                    }
                }
                _ => {}
            }

            // A discrete wheel has no gesture to end, so it never stretches.
            if event.phase == ScrollPhase::None {
                let travelled = (self.wheel_elapsed / WHEEL_DURATION).min(1.0);
                let span = self.wheel_target - self.wheel_start;
                let curve = |x: f32| {
                    cubic_bezier(
                        WHEEL_CURVE.0,
                        WHEEL_CURVE.1 * self.wheel_slope,
                        WHEEL_CURVE.2,
                        WHEEL_CURVE.3,
                        x,
                    )
                };
                let velocity = if travelled < 1.0 {
                    let step = 1e-3;
                    (curve((travelled + step).min(1.0)) - curve(travelled)) / step * span / WHEEL_DURATION
                } else {
                    0.0
                };

                self.wheel_target = (self.wheel_target + delta).clamp(0.0, max);
                self.wheel_start = self.offset;
                let span = self.wheel_target - self.wheel_start;
                // Carry the speed of the animation already running into the slope of the new one,
                // so a burst of notches reads as one accelerating scroll instead of a stutter.
                self.wheel_slope = if span.abs() > f32::EPSILON {
                    velocity * WHEEL_DURATION / span
                } else {
                    0.0
                };
                self.wheel_elapsed = 0.0;
                continue;
            }

            // Scrolling back toward the content closes the gap one to one with the finger, and
            // only what is left over moves the content. The gearing applies to pulling the band
            // open, not to letting it go: a stretch left behind by a fling was never paid for in
            // finger travel, so charging twenty times its size to undo it reads as a lock-up.
            if self.stretch != 0.0 && delta.signum() != self.stretch.signum() {
                let closed = delta.abs().min(self.stretch.abs()) * delta.signum();
                self.stretch += closed;
                self.accumulated = self.stretch * RUBBER_BAND_STIFFNESS;
                delta -= closed;
            }

            let want = self.offset + delta;
            self.offset = want.clamp(0.0, max);
            // Precise input drives the offset directly, so retire any wheel animation still
            // running rather than letting it drag the position back onto its curve.
            self.wheel_target = self.offset;
            self.wheel_elapsed = f32::MAX;
            let unused = want - self.offset;

            let stretching = matches!(self.gesture, Gesture::Active | Gesture::Momentum);
            if unused != 0.0 && stretching {
                if self.stretch != 0.0 || (unused + self.pending).abs() >= ENGAGE {
                    self.accumulated += unused + self.pending;
                    self.pending = 0.0;
                    self.stretch = self.accumulated / RUBBER_BAND_STIFFNESS;
                } else {
                    self.pending += unused;
                }
            }

            // Once a fling has run into the edge there is nothing left to steer, so take over with
            // the bounce immediately and let the rest of the momentum events fall on the floor.
            // A bounce already in flight owns the stretch and must not be restarted.
            let ended = matches!(self.gesture, Gesture::Active | Gesture::Momentum)
                && (event.phase.ended() || (self.gesture == Gesture::Momentum && self.stretch != 0.0));
            if ended {
                if self.stretch == 0.0 {
                    self.gesture = Gesture::Idle;
                    self.accumulated = 0.0;
                    self.pending = 0.0;
                } else {
                    self.gesture = Gesture::Bouncing;
                    self.elapsed = 0.0;
                    self.initial_stretch = self.stretch;
                    self.initial_velocity = self.velocity;
                }
            }
        }

        if self.gesture == Gesture::Bouncing {
            self.elapsed += dt;
            let decay = (-self.elapsed * RUBBER_BAND_STIFFNESS / RELEASE_PERIOD).exp();
            self.stretch = (self.initial_stretch + self.initial_velocity * self.elapsed * RELEASE_AMPLITUDE) * decay;
            if self.stretch.abs() < 1.0 {
                self.stretch = 0.0;
                self.accumulated = 0.0;
                self.gesture = Gesture::Idle;
            } else {
                // Keep the accumulator in step so a new gesture can take over mid-bounce.
                self.accumulated = self.stretch * RUBBER_BAND_STIFFNESS;
            }
        }

        if self.wheel_elapsed < WHEEL_DURATION {
            self.wheel_elapsed += dt;
            let travelled = (self.wheel_elapsed / WHEEL_DURATION).min(1.0);
            let progress = cubic_bezier(
                WHEEL_CURVE.0,
                WHEEL_CURVE.1 * self.wheel_slope,
                WHEEL_CURVE.2,
                WHEEL_CURVE.3,
                travelled,
            );
            self.offset = self.wheel_start + (self.wheel_target - self.wheel_start) * progress;
        }

        self.offset = self.offset.clamp(0.0, max);
        self.gesture != Gesture::Idle || self.wheel_elapsed < WHEEL_DURATION
    }
}

pub fn cubic_bezier(x1: f32, y1: f32, x2: f32, y2: f32, x: f32) -> f32 {
    let coefficients = |p1: f32, p2: f32| {
        let c = 3.0 * p1;
        let b = 3.0 * (p2 - p1) - c;
        (1.0 - c - b, b, c)
    };
    let (ax, bx, cx) = coefficients(x1, x2);
    let (ay, by, cy) = coefficients(y1, y2);
    let curve = |(a, b, c): (f32, f32, f32), t: f32| ((a * t + b) * t + c) * t;
    let slope = |(a, b, c): (f32, f32, f32), t: f32| (3.0 * a * t + 2.0 * b) * t + c;

    let mut t = x;
    for _ in 0..8 {
        let d = slope((ax, bx, cx), t);
        if d.abs() < 1e-6 {
            break;
        }
        let error = curve((ax, bx, cx), t) - x;
        if error.abs() < 1e-6 {
            return curve((ay, by, cy), t);
        }
        t -= error / d;
    }

    let (mut low, mut high) = (0.0, 1.0);
    t = x;
    for _ in 0..16 {
        let error = curve((ax, bx, cx), t) - x;
        if error.abs() < 1e-6 {
            break;
        }
        if error > 0.0 {
            high = t;
        } else {
            low = t;
        }
        t = (low + high) / 2.0;
    }
    curve((ay, by, cy), t)
}
