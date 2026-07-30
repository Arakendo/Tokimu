use crate::{
    ParticleError, ParticleId, ParticleSpawn2d, ParticleSpawnReport, ParticleState2d,
    ParticleStepReport, ParticleSystemConfig, ParticleSystemSnapshot, ParticleVec2, ScalarRange,
};

/// Bounded deterministic CPU particle simulation.
#[derive(Clone, Debug)]
pub struct ParticleSystem2d {
    config: ParticleSystemConfig,
    seed: u32,
    rng: DeterministicRng,
    next_id: u64,
    dropped_total: u64,
    particles: Vec<ParticleState2d>,
}

impl ParticleSystem2d {
    pub fn new(config: ParticleSystemConfig, seed: u32) -> Result<Self, ParticleError> {
        let config = config.validate()?;
        Ok(Self {
            config,
            seed,
            rng: DeterministicRng::new(seed),
            next_id: 1,
            dropped_total: 0,
            particles: Vec::with_capacity(config.capacity),
        })
    }

    pub fn config(&self) -> ParticleSystemConfig {
        self.config
    }

    pub fn seed(&self) -> u32 {
        self.seed
    }

    pub fn particles(&self) -> &[ParticleState2d] {
        &self.particles
    }

    pub fn active_count(&self) -> usize {
        self.particles.len()
    }

    pub fn dropped_total(&self) -> u64 {
        self.dropped_total
    }

    pub fn snapshot(&self) -> ParticleSystemSnapshot<'_> {
        ParticleSystemSnapshot {
            schema: 1,
            seed: self.seed,
            capacity: self.config.capacity,
            next_id: self.next_id,
            dropped_total: self.dropped_total,
            particles: &self.particles,
        }
    }

    pub fn spawn(
        &mut self,
        request: ParticleSpawn2d,
    ) -> Result<ParticleSpawnReport, ParticleError> {
        request.validate(self.config)?;

        let available = self.config.capacity.saturating_sub(self.particles.len());
        let admitted = request.count.min(available);
        let dropped = request.count - admitted;

        for _ in 0..admitted {
            let direction = self.sample(request.direction_radians);
            let speed = self.sample(request.speed);
            let velocity = request.inherited_velocity.add_vec(ParticleVec2::new(
                direction.cos() * speed,
                direction.sin() * speed,
            ));
            let particle = ParticleState2d {
                id: ParticleId(self.take_id()),
                position: request.origin,
                velocity,
                acceleration: request.acceleration,
                age: 0.0,
                lifetime: self.sample(request.lifetime),
                initial_size: self.sample(request.initial_size),
                final_size: self.sample(request.final_size),
                rotation: self.sample(request.initial_rotation),
                angular_velocity: self.sample(request.angular_velocity),
                drag: request.drag,
                presentation_role: request.presentation_role,
            };
            self.particles.push(particle);
        }

        self.dropped_total = self.dropped_total.saturating_add(dropped as u64);
        Ok(ParticleSpawnReport {
            requested: request.count,
            spawned: admitted,
            dropped,
            active: self.particles.len(),
        })
    }

    pub fn step(&mut self, delta_seconds: f32) -> Result<ParticleStepReport, ParticleError> {
        if !delta_seconds.is_finite() {
            return Err(ParticleError::NonFinite {
                field: "delta_seconds",
            });
        }
        if delta_seconds < 0.0 {
            return Err(ParticleError::Negative {
                field: "delta_seconds",
            });
        }
        if delta_seconds > self.config.maximum_step_seconds {
            return Err(ParticleError::StepTooLarge {
                requested: delta_seconds,
                maximum: self.config.maximum_step_seconds,
            });
        }

        let active_before = self.particles.len();
        if delta_seconds > 0.0 {
            for particle in &mut self.particles {
                particle.velocity = particle
                    .velocity
                    .add_vec(particle.acceleration.scale(delta_seconds));
                let drag_factor = 1.0 / (1.0 + particle.drag * delta_seconds);
                particle.velocity = particle.velocity.scale(drag_factor);
                particle.position = particle
                    .position
                    .add_vec(particle.velocity.scale(delta_seconds));
                particle.rotation += particle.angular_velocity * delta_seconds;
                particle.age += delta_seconds;
            }
            self.particles
                .retain(|particle| particle.age < particle.lifetime);
        }

        Ok(ParticleStepReport {
            active_before,
            active_after: self.particles.len(),
            expired: active_before - self.particles.len(),
        })
    }

    pub fn reset(&mut self, seed: u32) {
        self.seed = seed;
        self.rng = DeterministicRng::new(seed);
        self.next_id = 1;
        self.dropped_total = 0;
        self.particles.clear();
    }

    fn sample(&mut self, range: ScalarRange) -> f32 {
        if range.minimum == range.maximum {
            range.minimum
        } else {
            range.minimum + (range.maximum - range.minimum) * self.rng.next_unit()
        }
    }

    fn take_id(&mut self) -> u64 {
        let id = self.next_id;
        self.next_id = self.next_id.wrapping_add(1).max(1);
        id
    }
}

#[derive(Clone, Copy, Debug)]
struct DeterministicRng {
    state: u32,
}

impl DeterministicRng {
    fn new(seed: u32) -> Self {
        Self { state: seed.max(1) }
    }

    fn next_unit(&mut self) -> f32 {
        let mut value = self.state;
        value ^= value << 13;
        value ^= value >> 17;
        value ^= value << 5;
        self.state = value;
        value as f32 / u32::MAX as f32
    }
}
