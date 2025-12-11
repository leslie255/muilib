use cgmath::*;

use crate::{
    Bounds, CanvasFormat, ImageRef, Texture2d,
    resources::{AppResources, LoadResourceError},
    utils::*,
    wgpu_utils::{AsBindGroup, UniformBuffer},
};

impl<'a> From<&'a image::RgbaImage> for ImageRef<'a> {
    fn from(image: &'a image::RgbaImage) -> Self {
        Self::from_rgba_image(image)
    }
}

#[derive(Debug, Clone, AsBindGroup)]
struct ImageBindGroup {
    #[binding(0)]
    #[uniform]
    model_view: UniformBuffer<[[f32; 4]; 4]>,

    #[binding(1)]
    #[uniform]
    projection: UniformBuffer<[[f32; 4]; 4]>,

    #[binding(2)]
    #[texture_view(sample_type = float, view_dimension = 2, multisampled = false)]
    texture_view: wgpu::TextureView,

    #[binding(3)]
    #[sampler(filtering)]
    sampler: wgpu::Sampler,
}

#[derive(Debug, Clone)]
pub struct ImageElement {
    bind_group: ImageBindGroup,
    wgpu_bind_group: wgpu::BindGroup,
}

impl ImageElement {
    pub fn set_model_view(&self, queue: &wgpu::Queue, model_view: Matrix4<f32>) {
        self.bind_group.model_view.write(model_view.into(), queue);
    }

    pub fn set_projection(&self, queue: &wgpu::Queue, projection: Matrix4<f32>) {
        self.bind_group.projection.write(projection.into(), queue);
    }

    /// Convenience function over `set_model_view`.
    /// Sets `model_view` according to the bounds and line width provided.
    pub fn set_parameters(&self, queue: &wgpu::Queue, bounds: Bounds<f32>) {
        let model_view = Matrix4::from_translation(bounds.origin.to_vec().extend(0.))
            * Matrix4::from_nonuniform_scale(bounds.size.width, bounds.size.height, 1.);
        self.set_model_view(queue, model_view);
    }
}

#[derive(Debug, Clone)]
pub struct ImageRenderer<'cx> {
    pipeline: wgpu::RenderPipeline,
    bind_group_layout: wgpu::BindGroupLayout,
    _shader: &'cx wgpu::ShaderModule,
    sampler: wgpu::Sampler,
}

impl<'cx> ImageRenderer<'cx> {
    pub fn create(
        device: &wgpu::Device,
        resources: &'cx AppResources,
        canvas_format: CanvasFormat,
    ) -> Result<Self, LoadResourceError> {
        let shader = resources.load_shader("shaders/image.wgsl", device)?;
        let bind_group_layout = ImageBindGroup::create_bind_group_layout(device);
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: None,
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[],
        });
        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: None,
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: shader,
                entry_point: Some("vs_main"),
                compilation_options: the_default(),
                buffers: &[],
            },
            fragment: Some(wgpu::FragmentState {
                module: shader,
                entry_point: Some("fs_main"),
                compilation_options: the_default(),
                targets: &[Some(wgpu::ColorTargetState {
                    format: canvas_format.color_format,
                    blend: Some(wgpu::BlendState {
                        color: wgpu::BlendComponent {
                            operation: wgpu::BlendOperation::Add,
                            src_factor: wgpu::BlendFactor::SrcAlpha,
                            dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha,
                        },
                        alpha: wgpu::BlendComponent::REPLACE,
                    }),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            primitive: the_default(),
            depth_stencil: canvas_format.depth_stencil_format.map(|format| {
                wgpu::DepthStencilState {
                    format,
                    depth_write_enabled: true,
                    depth_compare: wgpu::CompareFunction::Always,
                    stencil: the_default(),
                    bias: the_default(),
                }
            }),
            multisample: the_default(),
            multiview: None,
            cache: None,
        });
        let sampler = device.create_sampler(&wgpu::wgt::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::Repeat,
            address_mode_v: wgpu::AddressMode::Repeat,
            address_mode_w: wgpu::AddressMode::Repeat,
            mag_filter: wgpu::FilterMode::Nearest,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::FilterMode::Linear,
            ..the_default()
        });
        Ok(Self {
            bind_group_layout,
            pipeline,
            _shader: shader,
            sampler,
        })
    }

    pub fn create_image(&self, device: &wgpu::Device, texture: &Texture2d) -> ImageElement {
        let bind_group = ImageBindGroup {
            model_view: UniformBuffer::create_init(device, Matrix4::identity().into()),
            projection: UniformBuffer::create_init(device, Matrix4::identity().into()),
            texture_view: texture.wgpu_texture_view().clone(),
            sampler: self.sampler.clone(),
        };
        let wgpu_bind_group = bind_group.create_bind_group(&self.bind_group_layout, device);
        ImageElement {
            bind_group,
            wgpu_bind_group,
        }
    }

    pub fn draw_image(&self, render_pass: &mut wgpu::RenderPass, image: &ImageElement) {
        render_pass.set_pipeline(&self.pipeline);
        render_pass.set_bind_group(0, &image.wgpu_bind_group, &[]);
        render_pass.draw(0..6, 0..1);
    }
}
