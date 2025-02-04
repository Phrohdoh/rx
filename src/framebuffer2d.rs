use rgx::core;
use rgx::core::*;
use rgx::kit;

use cgmath::{Matrix4, Vector4};

pub struct Pipeline {
    pub pipeline: core::Pipeline,

    width: u32,
    height: u32,
    ortho: Matrix4<f32>,
    bindings: core::BindingGroup,
    buf: core::UniformBuffer,
}

impl<'a> core::AbstractPipeline<'a> for Pipeline {
    type PrepareContext = ();
    type Uniforms = Matrix4<f32>;

    fn description() -> core::PipelineDescription<'a> {
        core::PipelineDescription {
            vertex_layout: &[
                core::VertexFormat::Float4,
                core::VertexFormat::Float2,
            ],
            pipeline_layout: &[
                Set(&[Binding {
                    binding: BindingType::UniformBuffer,
                    stage: ShaderStage::Vertex,
                }]),
                Set(&[Binding {
                    binding: BindingType::UniformBufferDynamic,
                    stage: ShaderStage::Vertex,
                }]),
                Set(&[
                    Binding {
                        binding: BindingType::SampledTexture,
                        stage: ShaderStage::Fragment,
                    },
                    Binding {
                        binding: BindingType::Sampler,
                        stage: ShaderStage::Fragment,
                    },
                ]),
            ],
            // TODO: Use `env("CARGO_MANIFEST_DIR")`
            vertex_shader: include_bytes!("data/framebuffer.vert.spv"),
            fragment_shader: include_bytes!("data/framebuffer.frag.spv"),
        }
    }

    fn setup(
        pipeline: core::Pipeline,
        dev: &core::Device,
        width: u32,
        height: u32,
    ) -> Self {
        let ortho = kit::ortho(width, height);
        let buf = dev.create_uniform_buffer(&[ortho]);
        let bindings =
            dev.create_binding_group(&pipeline.layout.sets[0], &[&buf]);

        Self {
            pipeline,
            ortho,
            buf,
            bindings,
            width,
            height,
        }
    }

    fn apply(&self, pass: &mut core::Pass) {
        pass.set_pipeline(&self.pipeline);
        pass.set_binding(&self.bindings, &[]);
    }

    fn prepare(
        &'a self,
        _ctx: (),
    ) -> Option<(&'a core::UniformBuffer, Vec<Matrix4<f32>>)> {
        Some((&self.buf, vec![self.ortho]))
    }

    fn resize(&mut self, w: u32, h: u32) {
        self.width = w;
        self.height = h;
        self.ortho = kit::ortho(w, h);
    }

    fn width(&self) -> u32 {
        self.width
    }

    fn height(&self) -> u32 {
        self.height
    }
}

impl Pipeline {
    pub fn binding(
        &self,
        renderer: &core::Renderer,
        framebuffer: &core::Framebuffer,
        sampler: &core::Sampler,
    ) -> core::BindingGroup {
        renderer.device.create_binding_group(
            &self.pipeline.layout.sets[2],
            &[framebuffer, sampler],
        )
    }

    pub fn vertex_buffer(
        width: u32,
        height: u32,
        r: &core::Renderer,
    ) -> core::VertexBuffer {
        let (w, h) = (width as f32, height as f32);

        #[rustfmt::skip]
        let vertices: Vec<(Vector4<f32>, f32, f32)> = vec![
            (0.0, 0.0, 0.0, 1.0),
            (w,   0.0, 1.0, 1.0),
            (w,   h,   1.0, 0.0),
            (0.0, 0.0, 0.0, 1.0),
            (0.0, h,   0.0, 0.0),
            (w,   h,   1.0, 0.0),
        ].iter().map(|(x, y, s, t)| {
            (Vector4::new(*x, *y, 0., 1.), *s, *t)
        }).collect();

        r.vertex_buffer(vertices.as_slice())
    }
}
