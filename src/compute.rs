
extern crate vulkano;
use tracer;
use std::sync::Arc;
use vulkano::buffer::BufferAccess;
use vulkano::buffer::CpuBufferPool;
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
    input_pool: CpuBufferPool<tracer::ty::Input>,

}

impl<I: 'static + ImageViewAccess + Send + Sync> ComputePart<I> {
    pub fn new(device: &Arc<Device>, image: Arc<I>) -> ComputePart<I> {
        let shader = tracer::Shader::load(device.clone()).expect("failed to create shader module");
        let pipeline = Arc::new(
            ComputePipeline::new(device.clone(), &shader.main_entry_point(), &())
                .expect("failed to create compute pipeline"),
        );

        let input_pool = CpuBufferPool::uniform_buffer(device.clone());

        ComputePart {
            pipeline,
            image,
            input_pool,
        }
    }

    pub fn render(
        &mut self,
        builder: AutoCommandBufferBuilder,
        dimensions: [u32; 2],
        input: tracer::ty::Input,
    ) -> AutoCommandBufferBuilder {
        builder.dispatch([dimensions[0] / 16, dimensions[1] / 16, 1],
                      self.pipeline.clone(),
                      self.next_set(input),
                      ())
            .unwrap()
    }

    fn next_set(
        &mut self,
        input: tracer::ty::Input,
    ) -> Arc<DescriptorSet + Send + Sync> {
        Arc::new(
            PersistentDescriptorSet::start(self.pipeline.clone(), 0)
                .add_image(self.image.clone())
                .unwrap()
                .add_buffer(self.input_pool.next(input).unwrap())
                .unwrap()
                .build()
                .unwrap(),
        )
    }
}
