use std::{f32::consts, sync::Arc};

mod artifacts;

use particle_tools::{
    lower_particle_instances_2d, ParticleEmitter2d, ParticlePresentationRole, ParticleSpawn2d,
    ParticleSystem2d, ParticleSystemConfig, ParticleVec2, ParticleView2d, ScalarRange,
};
use tokimu::{
    run_window_with_app, ClearCommand, Color, DrawMeshCommand, FrameOutcome, Instance2d, KeyCode,
    Material, MaterialHandle, Mesh, MeshHandle, NativeWindow, Pipeline, PipelineHandle,
    PipelineKind, PlatformEventHandler, PlatformInputEvent, PlatformResult, RenderCommand,
    Renderer, WgpuBackend, WindowConfig,
};

const PARTICLE_MESH: MeshHandle = MeshHandle(1);
const ORIGIN_MESH: MeshHandle = MeshHandle(2);
const STREAM_MATERIAL: MaterialHandle = MaterialHandle(1);
const SPRAY_MATERIAL: MaterialHandle = MaterialHandle(2);
const BURST_MATERIAL: MaterialHandle = MaterialHandle(3);
const ORIGIN_MATERIAL: MaterialHandle = MaterialHandle(4);

const STREAM_ROLE: ParticlePresentationRole = ParticlePresentationRole(1);
const SPRAY_ROLE: ParticlePresentationRole = ParticlePresentationRole(2);
const BURST_ROLE: ParticlePresentationRole = ParticlePresentationRole(3);

const STREAM_ORIGIN: ParticleVec2 = ParticleVec2::new(-0.58, -0.42);
const SPRAY_ORIGIN: ParticleVec2 = ParticleVec2::new(0.58, 0.18);
const BURST_ORIGIN: ParticleVec2 = ParticleVec2::new(0.08, -0.04);
const FIXED_STEP_SECONDS: f32 = 1.0 / 120.0;
const AUTOMATIC_BURST_SECONDS: f32 = 1.5;
const PARTICLE_SEED: u32 = 0x544f_4b49;

fn main() -> PlatformResult<()> {
    artifacts::write_structural_artifacts().expect("write hello-particles structural artifacts");
    run_window_with_app(
        WindowConfig {
            title: "Tokimu Hello Particles".into(),
            width: 1100,
            height: 720,
        },
        HelloParticlesApp::new(),
    )
}

struct HelloParticlesApp {
    renderer: Option<WgpuBackend>,
    window: Option<Arc<NativeWindow>>,
    pipeline: PipelineHandle,
    particles: ParticleSystem2d,
    stream: ParticleEmitter2d,
    spray: ParticleEmitter2d,
    frame_accumulator: f32,
    burst_accumulator: f32,
    paused: bool,
    closing: bool,
}

impl HelloParticlesApp {
    fn new() -> Self {
        let config = ParticleSystemConfig {
            capacity: 512,
            maximum_burst: 96,
            maximum_lifetime: 4.0,
            maximum_step_seconds: 1.0 / 60.0,
        };
        let particles =
            ParticleSystem2d::new(config, PARTICLE_SEED).expect("particle config is valid");
        let stream = ParticleEmitter2d::new(stream_request(), 42.0)
            .expect("stream emitter configuration is valid");
        let spray = ParticleEmitter2d::new(spray_request(), 30.0)
            .expect("spray emitter configuration is valid");

        Self {
            renderer: None,
            window: None,
            pipeline: PipelineHandle(0),
            particles,
            stream,
            spray,
            frame_accumulator: 0.0,
            burst_accumulator: AUTOMATIC_BURST_SECONDS,
            paused: false,
            closing: false,
        }
    }

    fn reset(&mut self) {
        self.particles.reset(PARTICLE_SEED);
        self.stream.reset();
        self.spray.reset();
        self.frame_accumulator = 0.0;
        self.burst_accumulator = AUTOMATIC_BURST_SECONDS;
        self.spawn_burst();
    }

    fn spawn_burst(&mut self) {
        self.particles
            .spawn(burst_request())
            .expect("burst request is valid");
    }

    fn update(&mut self, delta_seconds: f64) {
        if self.paused {
            return;
        }

        self.frame_accumulator += (delta_seconds as f32).clamp(0.0, 0.1);
        while self.frame_accumulator >= FIXED_STEP_SECONDS {
            self.stream
                .emit(&mut self.particles, FIXED_STEP_SECONDS)
                .expect("fixed stream step is valid");
            self.spray
                .emit(&mut self.particles, FIXED_STEP_SECONDS)
                .expect("fixed spray step is valid");

            self.burst_accumulator += FIXED_STEP_SECONDS;
            if self.burst_accumulator >= AUTOMATIC_BURST_SECONDS {
                self.burst_accumulator -= AUTOMATIC_BURST_SECONDS;
                self.spawn_burst();
            }

            self.particles
                .step(FIXED_STEP_SECONDS)
                .expect("fixed particle step is valid");
            self.frame_accumulator -= FIXED_STEP_SECONDS;
        }
    }

    fn render(&mut self) -> PlatformResult<FrameOutcome> {
        let Some(renderer) = self.renderer.as_mut() else {
            return Ok(FrameOutcome::Continue);
        };

        renderer.begin_frame();
        let mut commands = vec![RenderCommand::Clear(ClearCommand {
            color: Color::rgb(0.035, 0.052, 0.075),
        })];

        for (origin, scale) in [
            (STREAM_ORIGIN, [0.055, 0.018]),
            (SPRAY_ORIGIN, [0.055, 0.018]),
            (BURST_ORIGIN, [0.025, 0.025]),
        ] {
            commands.push(draw(
                ORIGIN_MESH,
                ORIGIN_MATERIAL,
                self.pipeline,
                Instance2d::identity()
                    .with_translation([origin.x, origin.y])
                    .with_scale(scale),
            ));
        }

        let view = ParticleView2d::new(
            ParticleVec2::new(-1.05, -1.05),
            ParticleVec2::new(1.05, 1.05),
        )
        .expect("presentation view is valid");
        let batch = lower_particle_instances_2d(
            self.particles.particles(),
            view,
            self.particles.config().capacity,
        );
        for particle in batch.instances {
            let size = particle.size.max(0.001);
            let material = match particle.presentation_role {
                STREAM_ROLE => STREAM_MATERIAL,
                SPRAY_ROLE => SPRAY_MATERIAL,
                BURST_ROLE => BURST_MATERIAL,
                _ => ORIGIN_MATERIAL,
            };
            let shape_scale = if particle.presentation_role == STREAM_ROLE {
                [size * 0.65, size * 1.35]
            } else {
                [size, size]
            };
            commands.push(draw(
                PARTICLE_MESH,
                material,
                self.pipeline,
                Instance2d::identity()
                    .with_translation([particle.position.x, particle.position.y])
                    .with_scale(shape_scale)
                    .with_rotation(particle.rotation),
            ));
        }

        renderer.submit(&commands);
        let _ = renderer.present()?;
        self.update_window_title();
        Ok(FrameOutcome::Continue)
    }

    fn update_window_title(&self) {
        if let Some(window) = self.window.as_ref() {
            window.set_title(&format!(
                "Tokimu Hello Particles | active={} | dropped={} | {} | Space burst | Q pause | R reset",
                self.particles.active_count(),
                self.particles.dropped_total(),
                if self.paused { "paused" } else { "running" },
            ));
        }
    }
}

impl PlatformEventHandler for HelloParticlesApp {
    fn on_native_window_created(&mut self, window: Arc<NativeWindow>) -> PlatformResult<()> {
        let size = window.inner_size();
        self.window = Some(window.clone());

        let mut renderer = WgpuBackend::for_window(window, size.width, size.height)?;
        renderer.upload_mesh(PARTICLE_MESH, &Mesh::diamond());
        renderer.upload_mesh(ORIGIN_MESH, &Mesh::quad());
        renderer.upload_material(
            STREAM_MATERIAL,
            &Material::new("particle-stream", Color::rgb(0.30, 0.90, 0.84)),
        )?;
        renderer.upload_material(
            SPRAY_MATERIAL,
            &Material::new("particle-spray", Color::rgb(0.98, 0.68, 0.27)),
        )?;
        renderer.upload_material(
            BURST_MATERIAL,
            &Material::new("particle-burst", Color::rgb(0.76, 0.60, 0.98)),
        )?;
        renderer.upload_material(
            ORIGIN_MATERIAL,
            &Material::new("particle-origin", Color::rgb(0.38, 0.48, 0.60)),
        )?;
        self.pipeline = renderer.register_pipeline(&Pipeline::new(
            "particle-solid-color",
            PipelineKind::SolidColor2d,
        ))?;
        self.renderer = Some(renderer);
        self.reset();
        self.update_window_title();
        Ok(())
    }

    fn on_platform_event(&mut self, event: PlatformInputEvent) -> PlatformResult<()> {
        match event {
            PlatformInputEvent::CloseRequested => self.closing = true,
            PlatformInputEvent::KeyboardInput { key, pressed } if pressed => match key {
                KeyCode::Space => self.spawn_burst(),
                KeyCode::KeyQ => self.paused = !self.paused,
                KeyCode::KeyR => self.reset(),
                KeyCode::Escape => self.closing = true,
                _ => {}
            },
            PlatformInputEvent::Resized { width, height } => {
                if let Some(renderer) = self.renderer.as_mut() {
                    renderer.resize_surface(width, height);
                }
            }
            _ => {}
        }
        self.update_window_title();
        Ok(())
    }

    fn on_frame(&mut self, delta_seconds: f64) -> PlatformResult<FrameOutcome> {
        if self.closing {
            return Ok(FrameOutcome::Exit);
        }
        self.update(delta_seconds);
        self.render()
    }
}

fn stream_request() -> ParticleSpawn2d {
    ParticleSpawn2d {
        count: 0,
        origin: STREAM_ORIGIN,
        inherited_velocity: ParticleVec2::ZERO,
        direction_radians: range(1.36, 1.78, "direction_radians"),
        speed: range(0.32, 0.62, "speed"),
        lifetime: range(1.3, 2.1, "lifetime"),
        initial_size: range(0.022, 0.038, "initial_size"),
        final_size: range(0.004, 0.012, "final_size"),
        initial_rotation: range(0.0, consts::TAU, "initial_rotation"),
        angular_velocity: range(-2.0, 2.0, "angular_velocity"),
        acceleration: ParticleVec2::new(0.0, -0.24),
        drag: 0.18,
        presentation_role: STREAM_ROLE,
    }
}

fn spray_request() -> ParticleSpawn2d {
    ParticleSpawn2d {
        count: 0,
        origin: SPRAY_ORIGIN,
        inherited_velocity: ParticleVec2::ZERO,
        direction_radians: range(2.86, 3.42, "direction_radians"),
        speed: range(0.28, 0.56, "speed"),
        lifetime: range(0.65, 1.15, "lifetime"),
        initial_size: range(0.018, 0.032, "initial_size"),
        final_size: range(0.035, 0.055, "final_size"),
        initial_rotation: range(0.0, consts::TAU, "initial_rotation"),
        angular_velocity: range(-4.0, 4.0, "angular_velocity"),
        acceleration: ParticleVec2::new(0.0, -0.10),
        drag: 0.55,
        presentation_role: SPRAY_ROLE,
    }
}

fn burst_request() -> ParticleSpawn2d {
    ParticleSpawn2d {
        count: 42,
        origin: BURST_ORIGIN,
        inherited_velocity: ParticleVec2::ZERO,
        direction_radians: range(0.0, consts::TAU, "direction_radians"),
        speed: range(0.18, 0.58, "speed"),
        lifetime: range(0.55, 1.05, "lifetime"),
        initial_size: range(0.032, 0.052, "initial_size"),
        final_size: range(0.004, 0.012, "final_size"),
        initial_rotation: range(0.0, consts::TAU, "initial_rotation"),
        angular_velocity: range(-5.0, 5.0, "angular_velocity"),
        acceleration: ParticleVec2::ZERO,
        drag: 0.34,
        presentation_role: BURST_ROLE,
    }
}

fn range(minimum: f32, maximum: f32, field: &'static str) -> ScalarRange {
    ScalarRange::new(minimum, maximum, field).expect("particle range is valid")
}

fn draw(
    mesh: MeshHandle,
    material: MaterialHandle,
    pipeline: PipelineHandle,
    instance: Instance2d,
) -> RenderCommand {
    RenderCommand::DrawMesh(DrawMeshCommand {
        mesh,
        material,
        pipeline,
        instance,
        camera: None,
        viewport: None,
    })
}
