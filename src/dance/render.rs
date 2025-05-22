use std::{iter, mem};

use color_eyre::eyre::Result;
use wgpu::{self as g, TextureView};

use crate::{app::Context, data::Buffer};

use super::{ComputedTransformation, Point};

#[derive(Debug)]
pub(super) struct Renderer {
    bind_group: g::BindGroup,
    pipeline: g::RenderPipeline,
}

impl Renderer {
    pub(super) fn new(
        transformations: &Buffer<ComputedTransformation>,
        dst_format: g::TextureFormat,
        context: &Context,
    ) -> Self {
        let shader = context
            .device
            .create_shader_module(g::include_wgsl!("render.wgsl"));

        let bind_group_layout =
            context
                .device
                .create_bind_group_layout(&g::BindGroupLayoutDescriptor {
                    label: Some("render bind group layout"),
                    entries: &[g::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: g::ShaderStages::VERTEX,
                        ty: g::BindingType::Buffer {
                            ty: g::BufferBindingType::Storage { read_only: true },
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    }],
                });

        let bind_group = context.device.create_bind_group(&g::BindGroupDescriptor {
            label: Some("render bind group"),
            layout: &bind_group_layout,
            entries: &[g::BindGroupEntry {
                binding: 0,
                resource: transformations.as_entire_binding(),
            }],
        });

        let pipeline_layout = context
            .device
            .create_pipeline_layout(&g::PipelineLayoutDescriptor {
                label: Some("render pipeline layout"),
                bind_group_layouts: &[&bind_group_layout],
                push_constant_ranges: &[],
            });

        let vertex_buffer_layout = g::VertexBufferLayout {
            array_stride: mem::size_of::<Point>() as u64,
            step_mode: g::VertexStepMode::Vertex,
            attributes: &g::vertex_attr_array![0 => Float32x2],
        };

        let pipeline = context
            .device
            .create_render_pipeline(&g::RenderPipelineDescriptor {
                label: Some("render pipeline"),
                layout: Some(&pipeline_layout),
                primitive: g::PrimitiveState {
                    topology: g::PrimitiveTopology::PointList,
                    ..Default::default()
                },
                vertex: g::VertexState {
                    module: &shader,
                    entry_point: Some("vertex"),
                    compilation_options: Default::default(),
                    buffers: &[vertex_buffer_layout],
                },
                fragment: Some(g::FragmentState {
                    module: &shader,
                    entry_point: Some("fragment"),
                    compilation_options: Default::default(),
                    targets: &[Some(g::ColorTargetState {
                        format: dst_format,
                        blend: Some(g::BlendState::REPLACE),
                        write_mask: g::ColorWrites::ALL,
                    })],
                }),
                depth_stencil: None,
                multisample: Default::default(),
                multiview: None,
                cache: None,
            });

        Self {
            bind_group,
            pipeline,
        }
    }

    pub(super) fn render(
        &self,
        points: &Buffer<Point>,
        dst: &TextureView,
        context: &Context,
    ) -> Result<()> {
        let mut encoder = context
            .device
            .create_command_encoder(&g::CommandEncoderDescriptor {
                label: Some("render command encoder"),
            });
        {
            let mut render_pass = encoder.begin_render_pass(&g::RenderPassDescriptor {
                label: Some("render pass"),
                color_attachments: &[Some(g::RenderPassColorAttachment {
                    view: dst,
                    resolve_target: None,
                    ops: g::Operations {
                        load: g::LoadOp::Clear(g::Color::BLACK),
                        store: g::StoreOp::Store,
                    },
                })],
                ..Default::default()
            });
            render_pass.set_pipeline(&self.pipeline);
            render_pass.set_vertex_buffer(0, points.slice(..));
            render_pass.set_bind_group(0, &self.bind_group, &[]);
            render_pass.draw(0..(points.len() as u32), 0..1);
        }

        context.queue.submit(iter::once(encoder.finish()));
        Ok(())
    }
}
