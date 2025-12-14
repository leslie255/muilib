use wgpu::util::DeviceExt as _;

use crate::{RectSize, utils::*};

#[derive(Debug, Clone, Copy)]
pub struct ImageRef<'a> {
    pub size: RectSize<u32>,
    pub format: wgpu::TextureFormat,
    pub data: &'a [u8],
}

impl<'a> ImageRef<'a> {
    pub fn from_rgba_image(image: &'a image::RgbaImage) -> Self {
        Self {
            size: RectSize::new(image.width(), image.height()),
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            data: image.as_ref(),
        }
    }

    pub fn width(&self) -> u32 {
        self.size.width
    }

    pub fn height(&self) -> u32 {
        self.size.height
    }

    pub fn size_f(&self) -> RectSize<f32> {
        RectSize {
            width: self.size.width as f32,
            height: self.size.height as f32,
        }
    }

    pub fn width_f(&self) -> f32 {
        self.size.width as f32
    }

    pub fn height_f(&self) -> f32 {
        self.size.height as f32
    }
}

#[derive(Debug, Clone)]
pub struct Texture2d {
    size: RectSize<u32>,
    wgpu_texture_view: wgpu::TextureView,
}

impl Texture2d {
    pub fn from_raw_parts(size: RectSize<u32>, wgpu_texture_view: wgpu::TextureView) -> Self {
        Self {
            size,
            wgpu_texture_view,
        }
    }

    pub fn create(device: &wgpu::Device, queue: &wgpu::Queue, image: ImageRef) -> Self {
        let texture = device.create_texture_with_data(
            queue,
            &wgpu::TextureDescriptor {
                label: None,
                size: wgpu::Extent3d {
                    width: image.width(),
                    height: image.height(),
                    depth_or_array_layers: 1,
                },
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: image.format,
                usage: wgpu::TextureUsages::COPY_DST | wgpu::TextureUsages::TEXTURE_BINDING,
                view_formats: &[],
            },
            wgpu::wgt::TextureDataOrder::MipMajor,
            image.data,
        );
        let texture_view = texture.create_view(&the_default());
        Self::from_raw_parts(image.size, texture_view)
    }

    pub fn wgpu_texture_view(&self) -> &wgpu::TextureView {
        &self.wgpu_texture_view
    }

    pub fn size(&self) -> RectSize<u32> {
        self.size
    }

    pub fn size_f(&self) -> RectSize<f32> {
        RectSize {
            width: self.size.width as f32,
            height: self.size.height as f32,
        }
    }
}
