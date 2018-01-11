
extern crate vulkano;
use tracer;
use std::sync::Arc;
use vulkano::buffer::BufferAccess;
use vulkano::command_buffer::AutoCommandBufferBuilder;
use vulkano::descriptor::descriptor_set::DescriptorSet;
use vulkano::descriptor::descriptor_set::PersistentDescriptorSet;
use vulkano::device::Device;
use vulkano::image::traits::ImageViewAccess;
use vulkano::pipeline::ComputePipeline;
use vulkano::pipeline::ComputePipelineAbstract;

pub struct ComputePart<I: 'static + ImageViewAccess + Send + Sync> {
    pipeline: Arc<ComputePipelineAbstract + Send + Sync>,
    image: Arc<I>,
}

impl<I: 'static + ImageViewAccess + Send + Sync> ComputePart<I> {
    pub fn new(device: &Arc<Device>, image: Arc<I>) -> ComputePart<I> {
        let shader = tracer::Shader::load(device.clone()).expect("failed to create shader module");
        let pipeline = Arc::new(
            ComputePipeline::new(device.clone(), &shader.main_entry_point(), &())
                .expect("failed to create compute pipeline"),
        );

        ComputePart {
            pipeline: pipeline,
            image: image,
        }
    }

    pub fn render(
        &mut self,
        builder: AutoCommandBufferBuilder,
        dimensions: [u32; 2],
        uniform: Arc<BufferAccess + Send + Sync + 'static>,
    ) -> AutoCommandBufferBuilder {
        builder.dispatch([dimensions[0] / 16, dimensions[1] / 16, 1],
                      self.pipeline.clone(),
                      self.next_set(uniform.clone()),
                      ())
            .unwrap()
    }

    fn next_set(
        &mut self,
        uniform: Arc<BufferAccess + Send + Sync>,
    ) -> Arc<DescriptorSet + Send + Sync> {
        Arc::new(
            PersistentDescriptorSet::start(self.pipeline.clone(), 0)
                .add_image(self.image.clone())
                .unwrap()
                .add_buffer(uniform)
                .unwrap()
                .build()
                .unwrap(),
        )
    }
}
