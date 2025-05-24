use std::{f32, iter};

use bytemuck::{Pod, Zeroable};
use color_eyre::eyre::Result;
use glam::{Affine2, Mat3, Vec2, Vec4};
use itertools::Itertools;
use render::Renderer;
use sim::Simulator;
use transformations::TransformationGenerator;
use wgpu as g;

use crate::{
    app::{Context, SubApp, SubAppBuilder, Time},
    data::{Buffer, WgpuMat3x3},
    random::Rng,
};

pub mod render;
pub mod sim;
pub mod transformations;

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
    point_buffer: Buffer<Point>,
    transformation_generator: TransformationGenerator,
    transformation_buffer: Buffer<ComputedTransformation>,
    simulator: Simulator,
    renderer: Renderer,
}

impl DanceSubApp {
    pub fn new(n_points: usize, transformation_colors: Vec<Vec4>, context: &Context) -> Self {
        let mut rng = Rng::new();

        let points = iter::repeat_with(|| Point {
            pos: rng.random::<Vec2>() * 2.0 - 1.0,
        })
        .take(n_points)
        .collect_vec();
        let point_buffer = Buffer::from_data(
            &points,
            Some("point buffer"),
            g::BufferUsages::STORAGE | g::BufferUsages::VERTEX,
            context,
        );

        let transformation_generator = TransformationGenerator::new(transformation_colors);

        let transformations = transformation_generator
            .generate(0.0)
            .into_iter()
            .map(ComputedTransformation::new)
            .collect_vec();
        let transformation_buffer = Buffer::from_data(
            &transformations,
            Some("transformation buffer"),
            g::BufferUsages::STORAGE | g::BufferUsages::COPY_DST,
            context,
        );

        let simulator = Simulator::new(&point_buffer, &transformation_buffer, context);

        let renderer = Renderer::new(
            &transformation_buffer,
            context.surface_config.format,
            context,
        );

        Self {
            point_buffer,
            transformation_generator,
            transformation_buffer,
            simulator,
            renderer,
        }
    }
}

pub struct DanceSubAppBuilder {
    pub n_points: usize,
    pub transformation_colors: Vec<Vec4>,
}

impl SubAppBuilder for DanceSubAppBuilder {
    fn build(self: Box<Self>, context: &Context) -> Result<Box<dyn SubApp>> {
        Ok(Box::new(DanceSubApp::new(
            self.n_points,
            self.transformation_colors,
            context,
        )))
    }
}

impl SubApp for DanceSubApp {
    fn update(&mut self, context: &Context, time: Time) -> Result<()> {
        let texture = context.surface.get_current_texture()?;
        let texture_view = texture.texture.create_view(&g::TextureViewDescriptor {
            label: Some("surface texture view"),
            ..Default::default()
        });
        self.renderer
            .render(&self.point_buffer, &texture_view, context)?;
        context.window.pre_present_notify();
        texture.present();

        let transformations = self
            .transformation_generator
            .generate(time.elapsed_f32 * 0.1)
            .into_iter()
            .map(ComputedTransformation::new)
            .collect_vec();
        context.queue.write_buffer(
            &self.transformation_buffer,
            0,
            bytemuck::cast_slice(&transformations),
        );

        self.simulator.step(context);

        Ok(())
    }
}
