use vulkano;
use vulkano::descriptor::descriptor_set;
use vulkano_win;
use std::sync::Arc;
use std::boxed::Box;
use std::marker::{Sync, Send};
use std::mem;

use types::Vec2;

pub struct GraphicsPart {
    pub dimensions: [u32; 2],
    pub swapchain: Arc<vulkano::swapchain::Swapchain>,
    pub recreate_swapchain: bool,
    pub images: Vec<Arc<vulkano::image::swapchain::SwapchainImage>>,
    pub texture: Arc<vulkano::image::StorageImage<vulkano::format::R8G8B8A8Unorm>>,
    pipeline: Arc<vulkano::pipeline::GraphicsPipeline<
        vulkano::pipeline::vertex::SingleBufferDefinition<Vec2>,
        Box<vulkano::descriptor::PipelineLayoutAbstract + Sync + Send>,
        Arc<vulkano::framebuffer::RenderPassAbstract + Send + Sync>>>,
    set: Arc<descriptor_set::DescriptorSet + Send +  Sync>,
    renderpass: Arc<vulkano::framebuffer::RenderPassAbstract + Send + Sync>,
    framebuffers: Option<
            Vec<Arc<vulkano::framebuffer::Framebuffer<
            Arc<vulkano::framebuffer::RenderPassAbstract + Send + Sync>,
            ((), Arc<vulkano::image::SwapchainImage>)>>>
        >,
    vertex_buffer: Arc<vulkano::buffer::cpu_access::CpuAccessibleBuffer<[Vec2]>>,
}

impl GraphicsPart {
    pub fn new(
        device: Arc<vulkano::device::Device>,
        window: &vulkano_win::Window,
        physical: vulkano::instance::PhysicalDevice,
        queue: Arc<vulkano::device::Queue>,
    ) -> GraphicsPart {

        let vs = vs::Shader::load(device.clone()).expect("failed to create shader module");
        let fs = fs::Shader::load(device.clone()).expect("failed to create shader module");

        let sampler = create_sampler(device.clone());

        let dimensions = {
            let (width, height) = window.window().get_inner_size_pixels().unwrap();
            [width, height]
        };

        let (swapchain, images) = create_swapchain(
            device.clone(),
            window.clone(),
            dimensions,
            physical.clone(),
            queue.clone(),
        );

        let renderpass = Arc::new(
            single_pass_renderpass!(device.clone(),
            attachments: {
                color: {
                    load: Clear,
                    store: DontCare,
                    format: swapchain.format(),
                    samples: 1,
                }
            },
            pass: {
                color: [color],
                depth_stencil: {}
            }
        ).unwrap(),
        );

        let pipeline = Arc::new(
            vulkano::pipeline::GraphicsPipeline::start()
                .vertex_input_single_buffer::<Vec2>()
                .vertex_shader(vs.main_entry_point(), ())
                .triangle_strip()
                .viewports_dynamic_scissors_irrelevant(1)
                .fragment_shader(fs.main_entry_point(), ())
                .blend_alpha_blending()
                .render_pass(
                    vulkano::framebuffer::Subpass::from(
                        renderpass.clone() as
                            Arc<vulkano::framebuffer::RenderPassAbstract + Send + Sync>,
                        0,
                    ).unwrap(),
                )
                .build(device.clone())
                .unwrap(),
        );

        let texture = vulkano::image::StorageImage::new(
            device.clone(),
            vulkano::image::Dimensions::Dim2d {
                width: dimensions[0],
                height: dimensions[1],
            },
            vulkano::format::R8G8B8A8Unorm,
            Some(queue.family()),
        ).unwrap();

        let set = Arc::new(
            descriptor_set::PersistentDescriptorSet::start(pipeline.clone(), 0)
                .add_sampled_image(texture.clone(), sampler.clone())
                .unwrap()
                .build()
                .unwrap(),
        );

        // Change to ImmutableBuffer
        let vertex_buffer = vulkano::buffer::cpu_access::CpuAccessibleBuffer::from_iter(
            device.clone(),
            vulkano::buffer::BufferUsage::all(),
            [
                Vec2 { position: [-1.0, -1.0] },
                Vec2 { position: [-1.0, 1.0] },
                Vec2 { position: [1.0, -1.0] },
                Vec2 { position: [1.0, 1.0] },
            ].iter()
                .cloned(),
        ).expect("failed to create buffer");

        GraphicsPart {
            pipeline: pipeline,
            dimensions: dimensions,
            swapchain: swapchain,
            recreate_swapchain: false,
            images: images,
            set: set,
            renderpass: renderpass,
            framebuffers: None,
            texture: texture,
            vertex_buffer: vertex_buffer,
        }
    }

    pub fn recreate_swapchain(&mut self, window: &vulkano_win::Window) -> bool {
        if !self.recreate_swapchain {
            return false;
        }

        self.dimensions = {
            let (new_width, new_height) = window.window().get_inner_size_pixels().unwrap();
            [new_width, new_height]
        };

        println!("{:?}", self.dimensions);

        let (new_swapchain, new_images) =
            match self.swapchain.recreate_with_dimension(self.dimensions) {
                Ok(r) => r,
                Err(vulkano::swapchain::SwapchainCreationError::UnsupportedDimensions) => {
                    return true;
                }
                Err(err) => panic!("{:?}", err),
            };

        mem::replace(&mut self.swapchain, new_swapchain);
        mem::replace(&mut self.images, new_images);

        // TODO: recreate texture here

        self.framebuffers = None;
        self.recreate_swapchain = false;
        false
    }

    pub fn recreate_framebuffers(&mut self) {
        if self.framebuffers.is_some() {
            return;
        }

        let new_framebuffers = Some(
            self.images
                .iter()
                .map(|image| {
                    Arc::new(
                        vulkano::framebuffer::Framebuffer::start(self.renderpass.clone())
                            .add(image.clone())
                            .unwrap()
                            .build()
                            .unwrap(),
                    )
                })
                .collect::<Vec<_>>(),
        );

        mem::replace(&mut self.framebuffers, new_framebuffers);
    }

    pub fn draw(
        &mut self,
        builder: vulkano::command_buffer::AutoCommandBufferBuilder,
        image_num: usize,
    ) -> vulkano::command_buffer::AutoCommandBufferBuilder {
        builder.begin_render_pass(
            self.framebuffers.as_ref().unwrap()[image_num].clone(), false,
            vec![[0.0, 0.0, 1.0, 1.0].into()])
            .unwrap()
            .draw(self.pipeline.clone(),
            vulkano::command_buffer::DynamicState {
                line_width: None,
                viewports: Some(vec![vulkano::pipeline::viewport::Viewport {
                    origin: [0.0, 0.0],
                    dimensions: [self.dimensions[0] as f32, self.dimensions[1] as f32],
                    depth_range: 0.0 .. 1.0,
                }]),
                scissors: None,
            },
            self.vertex_buffer.clone(),
            self.set.clone(), ())
            .unwrap().end_render_pass().unwrap()
    }

    pub fn acquire_next_image(
        &mut self,
    ) -> Result<(usize, vulkano::swapchain::SwapchainAcquireFuture), vulkano::swapchain::AcquireError> {
        match vulkano::swapchain::acquire_next_image(self.swapchain.clone(), None) {
            Ok(r) => Ok(r),
            Err(vulkano::swapchain::AcquireError::OutOfDate) => {
                self.recreate_swapchain = true;
                Err(vulkano::swapchain::AcquireError::OutOfDate)
            }
            err => err,
        }
    }
}

fn create_sampler(device: Arc<vulkano::device::Device>) -> Arc<vulkano::sampler::Sampler> {
    vulkano::sampler::Sampler::new(
        device,
        vulkano::sampler::Filter::Nearest,
        vulkano::sampler::Filter::Nearest,
        vulkano::sampler::MipmapMode::Nearest,
        vulkano::sampler::SamplerAddressMode::ClampToEdge,
        vulkano::sampler::SamplerAddressMode::ClampToEdge,
        vulkano::sampler::SamplerAddressMode::ClampToEdge,
        0.0,
        1.0,
        0.0,
        0.0,
    ).unwrap()
}

fn create_swapchain(
    device: Arc<vulkano::device::Device>,
    window: &vulkano_win::Window,
    dimensions: [u32; 2],
    physical: vulkano::instance::PhysicalDevice,
    queue: Arc<vulkano::device::Queue>,
) -> (Arc<vulkano::swapchain::Swapchain>, Vec<Arc<vulkano::image::SwapchainImage>>) {
    let caps = window.surface().capabilities(physical).expect(
        "failed to get surface capabilities",
    );

    let usage = caps.supported_usage_flags;
    let alpha = caps.supported_composite_alpha.iter().next().unwrap();
    let format = caps.supported_formats[0].0;

    vulkano::swapchain::Swapchain::new(
        device,
        window.surface().clone(),
        caps.min_image_count,
        format,
        dimensions,
        1,
        usage,
        &queue,
        vulkano::swapchain::SurfaceTransform::Identity,
        alpha,
        vulkano::swapchain::PresentMode::Fifo,
        true,
        None,
    ).expect("failed to create swapchain")
}

pub mod vs {
    #[derive(VulkanoShader)]
    #[ty = "vertex"]
    #[src = "
#version 450

layout (location = 0) in vec2 position;
layout (location = 0) out vec2 out_position;

out gl_PerVertex
{
	vec4 gl_Position;
};

void main() 
{
	out_position = position;
    gl_Position = vec4(position.xy, 0.0, 1.0);
}
"]
    #[allow(dead_code)]
    struct Dummy;
}

pub mod fs {
    #[derive(VulkanoShader)]
    #[ty = "fragment"]
    #[src = "

#version 450

#extension GL_ARB_separate_shader_objects : enable
#extension GL_ARB_shading_language_420pack : enable

layout(set = 0, binding = 0) uniform sampler2D tex;

layout (location = 0) in vec2 position;
// layout (binding = 0, rgba8) uniform readonly image2D to_draw;

layout (location = 0) out vec4 f_color;

void main() 
{
    f_color = texture(tex, (position+1.0)/2.0);

}"]
    #[allow(dead_code)]
    struct Dummy;
}
