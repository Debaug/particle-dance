use std::iter;

use wgpu as g;

use crate::{app::Context, data::Buffer};

use super::{ComputedTransformation, Point};

#[derive(Debug)]
pub(super) struct Simulator {
    bind_group: g::BindGroup,
    pipeline: g::ComputePipeline,
    n_workgroups: u32,
}

impl Simulator {
    pub(super) fn new(
        points: &Buffer<Point>,
        transformations: &Buffer<ComputedTransformation>,
        context: &Context,
    ) -> Self {
        assert!(points.len() < u16::MAX as usize);
        assert!(points.len().is_multiple_of(64));

        let n_workgroups = points.len() as u32 / 64;

        let shader = context
            .device
            .create_shader_module(g::include_wgsl!("sim.wgsl"));

        let bind_group_layout =
            context
                .device
                .create_bind_group_layout(&g::BindGroupLayoutDescriptor {
                    label: Some("simulation bind group layout"),
                    entries: &[
                        g::BindGroupLayoutEntry {
                            binding: 0,
                            visibility: g::ShaderStages::COMPUTE,
                            ty: g::BindingType::Buffer {
                                ty: g::BufferBindingType::Storage { read_only: true },
                                has_dynamic_offset: false,
                                min_binding_size: None,
                            },
                            count: None,
                        },
                        g::BindGroupLayoutEntry {
                            binding: 1,
                            visibility: g::ShaderStages::COMPUTE,
                            ty: g::BindingType::Buffer {
                                ty: g::BufferBindingType::Storage { read_only: false },
                                has_dynamic_offset: false,
                                min_binding_size: None,
                            },
                            count: None,
                        },
                    ],
                });

        let bind_group = context.device.create_bind_group(&g::BindGroupDescriptor {
            label: Some("simulation bind group"),
            layout: &bind_group_layout,
            entries: &[
                g::BindGroupEntry {
                    binding: 0,
                    resource: transformations.as_entire_binding(),
                },
                g::BindGroupEntry {
                    binding: 1,
                    resource: points.as_entire_binding(),
                },
            ],
        });

        let pipeline_layout = context
            .device
            .create_pipeline_layout(&g::PipelineLayoutDescriptor {
                label: Some("simulation pipeline layout"),
                bind_group_layouts: &[&bind_group_layout],
                push_constant_ranges: &[],
            });

        let pipeline = context
            .device
            .create_compute_pipeline(&g::ComputePipelineDescriptor {
                label: Some("simulation pipeline"),
                layout: Some(&pipeline_layout),
                module: &shader,
                entry_point: Some("simulate"),
                compilation_options: Default::default(),
                cache: None,
            });

        Self {
            bind_group,
            pipeline,
            n_workgroups,
        }
    }

    pub(super) fn step(&self, context: &Context) {
        let mut encoder = context
            .device
            .create_command_encoder(&g::CommandEncoderDescriptor {
                label: Some("simulation command encoder"),
            });
        {
            let mut compute_pass = encoder.begin_compute_pass(&g::ComputePassDescriptor {
                label: Some("simulation compute pass"),
                timestamp_writes: None,
            });
            compute_pass.set_pipeline(&self.pipeline);
            compute_pass.set_bind_group(0, &self.bind_group, &[]);
            compute_pass.dispatch_workgroups(self.n_workgroups, 1, 1);
        }
        context.queue.submit(iter::once(encoder.finish()));
    }
}
