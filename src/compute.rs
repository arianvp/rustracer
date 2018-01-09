
extern crate vulkano;
use vulkano::descriptor::descriptor_set;

use std::sync::Arc;


use shaders::mandelbrot::cs;

pub struct ComputePart<I: 'static + vulkano::image::traits::ImageViewAccess + Send + Sync> {
    pipeline: Arc<vulkano::pipeline::ComputePipelineAbstract + Send + Sync>,
    image: Arc<I>,
}

impl<I: 'static + vulkano::image::traits::ImageViewAccess + Send + Sync> ComputePart<I> {
    pub fn new(device: &Arc<vulkano::device::Device>, image: Arc<I>) -> ComputePart<I> {
        let shader = cs::Shader::load(device.clone()).expect("failed to create shader module");
        let pipeline = Arc::new(
            vulkano::pipeline::ComputePipeline::new(
                device.clone(),
                &shader.main_entry_point(),
                &(),
            ).expect("failed to create compute pipeline"),
        );

        ComputePart {
            pipeline: pipeline,
            image: image,
        }
    }

    pub fn render(
        &mut self,
        builder: vulkano::command_buffer::AutoCommandBufferBuilder,
        dimensions: [u32; 2],
        uniform: Arc<vulkano::buffer::BufferAccess + Send + Sync + 'static>,
    ) -> vulkano::command_buffer::AutoCommandBufferBuilder {
        builder.dispatch([dimensions[0] / 16, dimensions[1] / 16, 1],
                      self.pipeline.clone(),
                      self.next_set(uniform.clone()),
                      ())
            .unwrap()
    }

    fn next_set(
        &mut self,
        uniform: Arc<vulkano::buffer::BufferAccess + Send + Sync>,
    ) -> Arc<vulkano::descriptor::descriptor_set::DescriptorSet + Send + Sync> {
        Arc::new(
            descriptor_set::PersistentDescriptorSet::start(self.pipeline.clone(), 0)
                .add_image(self.image.clone())
                .unwrap()
                .add_buffer(uniform)
                .unwrap()
                .build()
                .unwrap(),
        )
    }
}
