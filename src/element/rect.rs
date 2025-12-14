use std::fmt::Debug;

use cgmath::*;

use crate::{
    element::CameraBindGroup, resources::{AppResources, LoadResourceError}, utils::*, wgpu_utils::{AsBindGroup, UniformBuffer}, Bounds, CanvasFormat, LineWidth, Rgba
};

#[derive(Debug, Clone, AsBindGroup)]
struct RectBindGroup {
    #[binding(0)]
    #[uniform]
    model_view: UniformBuffer<[[f32; 4]; 4]>,

    #[binding(1)]
    #[uniform]
    fill_color: UniformBuffer<Rgba>,

    #[binding(2)]
    #[uniform]
    line_color: UniformBuffer<Rgba>,

    #[binding(3)]
    #[uniform]
    line_width: UniformBuffer<[f32; 4]>,
}

#[derive(Debug, Clone)]
pub struct RectRenderer<'cx> {
    pipeline: wgpu::RenderPipeline,
    bind_group_layout: wgpu::BindGroupLayout,
    _shader: &'cx wgpu::ShaderModule,
}

impl<'cx> RectRenderer<'cx> {
    pub fn create(
        device: &wgpu::Device,
        resources: &'cx AppResources,
        canvas_format: CanvasFormat,
    ) -> Result<Self, LoadResourceError> {
        let shader = resources.load_shader("shaders/rect.wgsl", device)?;
        let bind_group_layout = RectBindGroup::create_bind_group_layout(device);
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: None,
            bind_group_layouts: &[
                &CameraBindGroup::create_bind_group_layout(device),
                &bind_group_layout,
            ],
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
        Ok(Self {
            bind_group_layout,
            pipeline,
            _shader: shader,
        })
    }

    pub fn create_rect(&self, device: &wgpu::Device) -> RectElement {
        let bind_group = RectBindGroup {
            model_view: UniformBuffer::create_init(device, Matrix4::identity().into()),
            fill_color: UniformBuffer::create_init(device, Rgba::from_hex(0xFFFFFFFF)),
            line_color: UniformBuffer::create_init(device, Rgba::from_hex(0xFFFFFFFF)),
            line_width: UniformBuffer::create_init(device, [0., 0., 0., 0.]),
        };
        let wgpu_bind_group = bind_group.create_bind_group(&self.bind_group_layout, device);
        RectElement {
            bind_group,
            wgpu_bind_group,
        }
    }

    pub fn draw_rect(&self, render_pass: &mut wgpu::RenderPass, rect: &RectElement) {
        render_pass.set_pipeline(&self.pipeline);
        render_pass.set_bind_group(1, &rect.wgpu_bind_group, &[]);
        render_pass.draw(0..6, 0..1);
    }
}

#[derive(Debug, Clone)]
pub struct RectElement {
    bind_group: RectBindGroup,
    wgpu_bind_group: wgpu::BindGroup,
}

impl RectElement {
    pub fn set_model_view(&self, queue: &wgpu::Queue, model_view: Matrix4<f32>) {
        self.bind_group.model_view.write(model_view.into(), queue);
    }

    /// Convenience function over `set_model_view` and `set_normalized_line_width`.
    /// Sets `model_view` and normalized `line_width` according to the bounds and line width
    /// provided.
    pub fn set_parameters(
        &self,
        queue: &wgpu::Queue,
        bounds: Bounds<f32>,
        line_width: impl Into<LineWidth>,
    ) {
        let model_view = Matrix4::from_translation(bounds.origin.to_vec().extend(0.))
            * Matrix4::from_nonuniform_scale(bounds.size.width, bounds.size.height, 1.);
        self.set_model_view(queue, model_view);
        self.set_normalized_line_width(queue, line_width.into().normalized_in(bounds.size));
    }

    pub fn set_fill_color(&self, queue: &wgpu::Queue, fill_color: impl Into<Rgba>) {
        self.bind_group.fill_color.write(fill_color.into(), queue);
    }

    pub fn set_line_color(&self, queue: &wgpu::Queue, line_color: impl Into<Rgba>) {
        self.bind_group.line_color.write(line_color.into(), queue);
    }

    pub fn set_normalized_line_width(&self, queue: &wgpu::Queue, line_width: impl Into<LineWidth>) {
        self.bind_group
            .line_width
            .write(line_width.into().to_array(), queue);
    }
}
