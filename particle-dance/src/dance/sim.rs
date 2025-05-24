use std::{mem, num::NonZero};

use wgpu as g;

use crate::{app::Context, data::Buffer};

use super::{ComputedTransformation, Point};

#[derive(Debug)]
pub(super) struct Simulator {
    transformation_bind_group: g::BindGroup,
    full_point_chunk_bind_group: Option<g::BindGroup>,
    point_rest_chunk_bind_group: Option<g::BindGroup>,
    pipeline: g::ComputePipeline,
    n_full_dispatches: u32,
    n_rest_points: u32,
}

impl Simulator {
    const INVOCATIONS_PER_WORKGROUP: u32 = 64;
    const MAX_WORKGROUPS_PER_DISPATCH: u32 = 65535;
    const FULL_POINT_CHUNK_LEN: u32 =
        Self::INVOCATIONS_PER_WORKGROUP * Self::MAX_WORKGROUPS_PER_DISPATCH;

    pub(super) fn new(
        points: &Buffer<Point>,
        transformations: &Buffer<ComputedTransformation>,
        context: &Context,
    ) -> Self {
        assert!(u32::try_from(points.size()).is_ok());
        let n_points = points.len() as u32;

        let shader = context
            .device
            .create_shader_module(g::include_wgsl!("sim.wgsl"));

        let transformation_bind_group_layout =
            context
                .device
                .create_bind_group_layout(&g::BindGroupLayoutDescriptor {
                    label: Some("simulation transformation bind group layout"),
                    entries: &[g::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: g::ShaderStages::COMPUTE,
                        ty: g::BindingType::Buffer {
                            ty: g::BufferBindingType::Storage { read_only: true },
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    }],
                });

        let transformation_bind_group = context.device.create_bind_group(&g::BindGroupDescriptor {
            label: Some("simulation transformation bind group"),
            layout: &transformation_bind_group_layout,
            entries: &[g::BindGroupEntry {
                binding: 0,
                resource: transformations.as_entire_binding(),
            }],
        });

        let n_full_point_chunks = n_points / Self::FULL_POINT_CHUNK_LEN;
        let n_rest_points = n_points % Self::FULL_POINT_CHUNK_LEN;

        let point_bind_group_layout =
            context
                .device
                .create_bind_group_layout(&g::BindGroupLayoutDescriptor {
                    label: Some("simulation point bind group layout"),
                    entries: &[g::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: g::ShaderStages::COMPUTE,
                        ty: g::BindingType::Buffer {
                            ty: g::BufferBindingType::Storage { read_only: false },
                            has_dynamic_offset: true,
                            min_binding_size: None,
                        },
                        count: None,
                    }],
                });

        let full_point_chunk_bind_group = (n_full_point_chunks != 0).then(|| {
            context.device.create_bind_group(&g::BindGroupDescriptor {
                label: Some("simulation full point chunk bind group"),
                layout: &point_bind_group_layout,
                entries: &[g::BindGroupEntry {
                    binding: 0,
                    resource: g::BindingResource::Buffer(g::BufferBinding {
                        buffer: points,
                        offset: 0,
                        size: NonZero::new(
                            Self::FULL_POINT_CHUNK_LEN as u64 * mem::size_of::<Point>() as u64,
                        ),
                    }),
                }],
            })
        });

        let point_rest_chunk_bind_group = (n_rest_points != 0).then(|| {
            context.device.create_bind_group(&g::BindGroupDescriptor {
                label: Some("simulation bind group"),
                layout: &point_bind_group_layout,
                entries: &[g::BindGroupEntry {
                    binding: 0,
                    resource: g::BindingResource::Buffer(g::BufferBinding {
                        buffer: points,
                        offset: n_full_point_chunks as u64
                            * Self::FULL_POINT_CHUNK_LEN as u64
                            * mem::size_of::<Point>() as u64,
                        size: NonZero::new(n_rest_points as u64 * mem::size_of::<Point>() as u64),
                    }),
                }],
            })
        });

        let pipeline_layout = context
            .device
            .create_pipeline_layout(&g::PipelineLayoutDescriptor {
                label: Some("simulation pipeline layout"),
                bind_group_layouts: &[&transformation_bind_group_layout, &point_bind_group_layout],
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
            transformation_bind_group,
            full_point_chunk_bind_group,
            point_rest_chunk_bind_group,
            pipeline,
            n_full_dispatches: n_full_point_chunks,
            n_rest_points,
        }
    }

    pub(super) fn step(&self, context: &Context) {
        let bind_groups_offsets_lens = (0..self.n_full_dispatches)
            .filter_map(|i| {
                Some((
                    self.full_point_chunk_bind_group.as_ref()?,
                    i * Self::FULL_POINT_CHUNK_LEN,
                    Self::FULL_POINT_CHUNK_LEN,
                ))
            })
            .chain(
                self.point_rest_chunk_bind_group
                    .as_ref()
                    .map(|bind_group| (bind_group, 0, self.n_rest_points)),
            );

        let commands = bind_groups_offsets_lens.map(|(bind_group, offset, len)| {
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
                compute_pass.set_bind_group(0, &self.transformation_bind_group, &[]);
                compute_pass.set_bind_group(
                    1,
                    bind_group,
                    &[offset * mem::size_of::<Point>() as u32],
                );
                compute_pass.dispatch_workgroups(len / Self::INVOCATIONS_PER_WORKGROUP, 1, 1);
            }
            encoder.finish()
        });

        context.queue.submit(commands);
    }
}
