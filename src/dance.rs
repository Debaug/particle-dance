use std::{f32, iter};

use bytemuck::{Pod, Zeroable};
use color_eyre::eyre::Result;
use glam::{vec2, vec4, Affine2, Mat3, Vec2, Vec4};
use itertools::Itertools;
use rand::Rng;
use render::Renderer;
use sim::Simulator;
use wgpu as g;

use crate::{
    app::{Context, SubApp, SubAppBuilder},
    data::{Buffer, WgpuMat3x3},
};

pub mod render;
pub mod sim;

#[derive(Debug, Clone, Copy, Zeroable, Pod)]
#[repr(C)]
pub struct Point {
    pub pos: Vec2,
}

#[derive(Debug, Clone, Copy, Zeroable, Pod)]
#[repr(C)]
pub struct Transformation {
    pub center: Vec2,
    pub scale: f32,
    pub angle: f32,
    pub color: Vec4,
}

#[derive(Debug, Clone, Copy, Zeroable, Pod)]
#[repr(C)]
struct ComputedTransformation {
    transformation: Transformation,
    matrix: WgpuMat3x3,
}

impl ComputedTransformation {
    fn new(transformation: Transformation) -> Self {
        Self {
            transformation,
            matrix: Self::compute(transformation),
        }
    }

    fn compute(transformation: Transformation) -> WgpuMat3x3 {
        let affine = Affine2::from_scale_angle_translation(
            Vec2::splat(transformation.scale),
            transformation.angle,
            transformation.center,
        ) * Affine2::from_translation(-transformation.center);
        Mat3::from(affine).into()
    }
}

#[derive(Debug)]
pub struct DanceSubApp {
    points: Buffer<Point>,
    transformations: Buffer<ComputedTransformation>,
    simulator: Simulator,
    renderer: Renderer,
}

impl DanceSubApp {
    pub fn new(n_points: usize, context: &Context) -> Self {
        let mut rng = rand::rng();
        let points = iter::repeat_with(|| Point {
            pos: rng.random::<Vec2>() * 2.0 - 1.0,
        })
        .take(n_points)
        .collect_vec();
        let points = Buffer::from_data(
            &points,
            Some("point buffer"),
            g::BufferUsages::STORAGE | g::BufferUsages::VERTEX,
            context,
        );

        let transformations = [
            Transformation {
                center: vec2(0.0, -0.75),
                scale: 0.33,
                angle: 0.0,
                color: vec4(1.0, 0.0, 0.0, 1.0),
            },
            Transformation {
                center: vec2(-0.5, -0.5),
                scale: 0.5,
                angle: f32::consts::FRAC_PI_3,
                color: vec4(0.0, 1.0, 0.0, 1.0),
            },
            Transformation {
                center: vec2(0.25, 0.25),
                scale: 0.8,
                angle: f32::consts::FRAC_PI_4,
                color: vec4(0.0, 0.0, 1.0, 1.0),
            },
        ]
        .map(ComputedTransformation::new);
        let transformations = Buffer::from_data(
            &transformations,
            Some("transformation buffer"),
            g::BufferUsages::STORAGE,
            context,
        );

        let simulator = Simulator::new(&points, &transformations, context);

        let renderer = Renderer::new(&transformations, context.surface_config.format, context);

        Self {
            points,
            transformations,
            simulator,
            renderer,
        }
    }
}

pub struct DanceSubAppBuilder {
    pub n_points: usize,
}

impl SubAppBuilder for DanceSubAppBuilder {
    fn build(self: Box<Self>, context: &Context) -> Result<Box<dyn SubApp>> {
        Ok(Box::new(DanceSubApp::new(self.n_points, context)))
    }
}

impl SubApp for DanceSubApp {
    fn update(&mut self, context: &Context, _delta_time: std::time::Duration) -> Result<()> {
        let texture = context.surface.get_current_texture()?;
        let texture_view = texture.texture.create_view(&g::TextureViewDescriptor {
            label: Some("surface texture view"),
            ..Default::default()
        });
        self.renderer.render(&self.points, &texture_view, context)?;
        context.window.pre_present_notify();
        texture.present();

        self.simulator.step(context);

        Ok(())
    }
}
