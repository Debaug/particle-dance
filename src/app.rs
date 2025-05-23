use std::{
    mem,
    sync::Arc,
    time::{Duration, Instant},
};

use color_eyre::eyre::Result;
use itertools::Itertools;
use log::error;
use wgpu as g;
use winit::{self as w};

pub struct App {
    inner: AppInner,
    start: Instant,
    time: Time,
    next_frame_time: Instant,
    target_delta_time: Duration,
}

enum AppInner {
    Created {
        window_attributes: w::window::WindowAttributes,
        sub_app_builders: Vec<Box<dyn SubAppBuilder>>,
    },
    Ready {
        context: Context,
        sub_apps: Vec<Box<dyn SubApp>>,
    },
}

#[derive(Debug)]
pub struct Context {
    pub instance: g::Instance,
    pub adapter: g::Adapter,
    pub device: g::Device,
    pub queue: g::Queue,
    pub window: Arc<w::window::Window>,
    pub surface: g::Surface<'static>,
    pub surface_config: g::SurfaceConfiguration,
}

#[derive(Debug, Clone, Copy)]
pub struct Time {
    pub time: Instant,
    pub elapsed: Duration,
    pub elapsed_f32: f32,
    pub delta: Duration,
    pub delta_f32: f32,
}

pub trait SubAppBuilder: 'static {
    fn build(self: Box<Self>, context: &Context) -> Result<Box<dyn SubApp>>;
}

pub trait SubApp: 'static {
    fn update(&mut self, context: &Context, time: Time) -> Result<()>;
}

impl App {
    pub fn new(delta_time: Duration, window_attributes: w::window::WindowAttributes) -> Self {
        let now = Instant::now();

        let time = Time {
            time: now,
            elapsed: Duration::ZERO,
            elapsed_f32: 0.0,
            delta: Duration::ZERO,
            delta_f32: 0.0,
        };

        Self {
            inner: AppInner::Created {
                sub_app_builders: vec![],
                window_attributes,
            },
            start: now,
            time,
            next_frame_time: Instant::now(),
            target_delta_time: delta_time,
        }
    }

    pub fn add_sub_app<T: SubAppBuilder>(&mut self, builder: T) -> &mut Self {
        let AppInner::Created {
            sub_app_builders, ..
        } = &mut self.inner
        else {
            panic!("tried to add a sub app to a ready app")
        };
        sub_app_builders.push(Box::new(builder));
        self
    }

    pub fn run(&mut self) -> Result<()> {
        let event_loop = w::event_loop::EventLoop::new()?;
        Ok(event_loop.run_app(self)?)
    }

    fn request_redraw_if_needed(&mut self) {
        let AppInner::Ready { context, .. } = &self.inner else {
            return;
        };

        let now = Instant::now();
        if now < self.next_frame_time {
            return;
        }

        self.time.delta = now - self.time.time;
        self.time.delta_f32 = self.time.delta.as_secs_f32();
        self.time.time = now;
        self.time.elapsed = self.time.time - self.start;
        self.time.elapsed_f32 = self.time.elapsed.as_secs_f32();
        self.next_frame_time = now + self.target_delta_time;
        context.window.request_redraw();
    }
}

impl w::application::ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &w::event_loop::ActiveEventLoop) {
        let AppInner::Created {
            sub_app_builders,
            window_attributes,
        } = &mut self.inner
        else {
            return;
        };

        event_loop.set_control_flow(w::event_loop::ControlFlow::Poll);

        let window = match event_loop.create_window(mem::take(window_attributes)) {
            Ok(window) => window,
            Err(error) => {
                error!("failed to create window: {error:?}");
                event_loop.exit();
                return;
            }
        };

        let context = match futures::executor::block_on(Context::new(window)) {
            Ok(context) => context,
            Err(error) => {
                error!("failed to create app context: {error:?}");
                event_loop.exit();
                return;
            }
        };

        let sub_apps = match sub_app_builders
            .drain(..)
            .map(|builder| builder.build(&context))
            .try_collect()
        {
            Ok(sub_apps) => sub_apps,
            Err(error) => {
                error!("failed to build sub-apps: {error:?}");
                event_loop.exit();
                return;
            }
        };

        self.inner = AppInner::Ready { context, sub_apps };

        self.next_frame_time = Instant::now() + self.target_delta_time;
    }

    fn window_event(
        &mut self,
        event_loop: &w::event_loop::ActiveEventLoop,
        _: w::window::WindowId,
        event: w::event::WindowEvent,
    ) {
        let AppInner::Ready { context, sub_apps } = &mut self.inner else {
            return;
        };

        use w::event::WindowEvent as E;
        match event {
            E::CloseRequested => event_loop.exit(),

            E::RedrawRequested => {
                for sub_app in sub_apps {
                    if let Err(error) = sub_app.update(context, self.time) {
                        error!("failed to update sub-app: {error:?}");
                        event_loop.exit();
                        return;
                    }
                }
            }

            _ => {}
        }
    }

    fn about_to_wait(&mut self, event_loop: &w::event_loop::ActiveEventLoop) {
        if event_loop.exiting() {
            return;
        }

        self.request_redraw_if_needed();
        event_loop.set_control_flow(winit::event_loop::ControlFlow::WaitUntil(
            self.next_frame_time,
        ));
    }
}

impl Context {
    async fn new(window: w::window::Window) -> Result<Self> {
        let window = Arc::new(window);

        let instance = g::Instance::new(&g::InstanceDescriptor {
            backends: g::Backends::all(),
            ..Default::default()
        });

        let surface = instance.create_surface(window.clone())?;

        let adapter = instance
            .request_adapter(&g::RequestAdapterOptions {
                compatible_surface: Some(&surface),
                ..Default::default()
            })
            .await?;

        let (device, queue) = adapter
            .request_device(&g::DeviceDescriptor {
                label: Some("Device"),
                ..Default::default()
            })
            .await?;

        let capabilities = surface.get_capabilities(&adapter);
        let window_size = window.inner_size();
        let surface_config = g::SurfaceConfiguration {
            usage: g::TextureUsages::RENDER_ATTACHMENT,
            format: capabilities.formats[0],
            width: window_size.width,
            height: window_size.height,
            present_mode: capabilities.present_modes[0],
            desired_maximum_frame_latency: 2,
            alpha_mode: capabilities.alpha_modes[0],
            view_formats: vec![],
        };
        surface.configure(&device, &surface_config);

        Ok(Self {
            instance,
            adapter,
            device,
            queue,
            window,
            surface,
            surface_config,
        })
    }
}
