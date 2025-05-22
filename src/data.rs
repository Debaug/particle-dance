use std::{
    marker::PhantomData,
    mem,
    ops::{Deref, RangeBounds},
};

use bytemuck::{Pod, Zeroable};
use color_eyre::eyre::Result;
use glam::{Mat3, Vec4, Vec4Swizzles};
use wgpu::{self as g, util::DeviceExt};

use crate::app::Context;

#[derive(Debug)]
pub struct Buffer<T: Pod> {
    raw: g::Buffer,
    _marker: PhantomData<T>,
}

impl<T: Pod> Deref for Buffer<T> {
    type Target = g::Buffer;
    fn deref(&self) -> &Self::Target {
        &self.raw
    }
}

impl<T: Pod> Buffer<T> {
    pub fn from_data(
        data: &[T],
        label: Option<&str>,
        usage: g::BufferUsages,
        context: &Context,
    ) -> Self {
        let raw = context
            .device
            .create_buffer_init(&g::util::BufferInitDescriptor {
                label,
                contents: bytemuck::cast_slice(data),
                usage,
            });

        Self {
            raw,
            _marker: PhantomData,
        }
    }

    pub fn len(&self) -> usize {
        self.size() as usize / mem::size_of::<T>()
    }

    pub fn is_empty(&self) -> bool {
        self.size() == 0
    }

    pub fn map_block(
        &mut self,
        mode: g::MapMode,
        bounds: impl RangeBounds<g::BufferAddress>,
        context: &Context,
    ) -> Result<()> {
        let (tx, rx) = std::sync::mpsc::channel();
        self.raw.map_async(mode, bounds, move |result| {
            tx.send(result).expect("failed to send buffer map result")
        });
        context.device.poll(g::PollType::Wait)?;
        rx.recv().expect("failed to recieve buffer map result")?;
        Ok(())
    }
}

#[derive(Debug, Clone, Copy, Zeroable, Pod)]
#[repr(transparent)]
pub struct WgpuMat3x3([Vec4; 3]);

impl From<Mat3> for WgpuMat3x3 {
    fn from(value: Mat3) -> Self {
        Self([0, 1, 2].map(|i| value.col(i).extend(0.0)))
    }
}

impl From<WgpuMat3x3> for Mat3 {
    fn from(value: WgpuMat3x3) -> Self {
        let cols = value.0.map(|col| col.xyz());
        Mat3::from_cols(cols[0], cols[1], cols[2])
    }
}
