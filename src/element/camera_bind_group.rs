use cgmath::Matrix4;

use crate::wgpu_utils::{AsBindGroup, UniformBuffer};

#[derive(Debug, Clone, AsBindGroup)]
pub struct CameraBindGroup {
    #[binding(0)]
    #[uniform]
    pub projection: UniformBuffer<[[f32; 4]; 4]>,

    #[binding(1)]
    #[uniform]
    pub aaf: UniformBuffer<f32>,
}

impl CameraBindGroup {
    pub fn set_projection(&self, queue: &wgpu::Queue, projection: Matrix4<f32>) {
        self.projection.write(projection.into(), queue);
    }

    pub fn set_aaf(&self, queue: &wgpu::Queue, aaf: f32) {
        self.aaf.write(aaf, queue);
    }
}
