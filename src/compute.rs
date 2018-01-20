
use tracer;
use std::iter;
use std::sync::Arc;
use vulkano::buffer::{BufferAccess, BufferUsage, CpuBufferPool, CpuAccessibleBuffer, DeviceLocalBuffer};
use vulkano::command_buffer::AutoCommandBufferBuilder;
use vulkano::descriptor::descriptor_set::DescriptorSet;
use vulkano::descriptor::descriptor_set::PersistentDescriptorSet;
use vulkano::device::Device;
use vulkano::image::traits::ImageViewAccess;
use vulkano::instance::QueueFamily;
use vulkano::pipeline::ComputePipeline;
use vulkano::pipeline::ComputePipelineAbstract;

pub struct ComputePart<I: 'static + ImageViewAccess + Send + Sync> {
    pipeline: Arc<ComputePipelineAbstract + Send + Sync>,
    image: Arc<I>,
    input_pool: CpuBufferPool<tracer::ty::Input>,
    spheres: Arc<CpuAccessibleBuffer<[tracer::ty::Sphere]>>,
    planes: Arc<CpuAccessibleBuffer<[tracer::ty::Plane]>>,
    triangles: Arc<CpuAccessibleBuffer<[tracer::ty::Triangle]>>,
    accum: Arc<CpuAccessibleBuffer<[[f32;4]]>>,
}

impl<I: 'static + ImageViewAccess + Send + Sync> ComputePart<I> {
    pub fn new(device: &Arc<Device>, image: Arc<I>, spheres: Vec<tracer::ty::Sphere>, planes: Vec<tracer::ty::Plane>, triangles: Vec<tracer::ty::Triangle>, family: QueueFamily) -> ComputePart<I> {
        let shader = tracer::Shader::load(device.clone()).expect("failed to create shader module");
        let pipeline = Arc::new(
            ComputePipeline::new(device.clone(), &shader.main_entry_point(), &())
                .expect("failed to create compute pipeline"),
        );

        let input_pool = CpuBufferPool::uniform_buffer(device.clone());
        let spheres = CpuAccessibleBuffer::from_iter(device.clone(), BufferUsage::all(), spheres.into_iter()).unwrap();
        let planes = CpuAccessibleBuffer::from_iter(device.clone(), BufferUsage::all(), planes.into_iter()).unwrap();
        let triangles = CpuAccessibleBuffer::from_iter(device.clone(), BufferUsage::all(), triangles.into_iter()).unwrap();

        let accum = CpuAccessibleBuffer::from_iter(device.clone(), BufferUsage::all(), (0..512*512).map(|_|[0.;4])).unwrap();

        ComputePart {
            pipeline,
            image,
            input_pool,
            spheres,
            planes,
            triangles,
            accum,
        }
    }
    pub fn calculate_energy(&self, framenum: u32) -> f32 {
        let content = self.accum.read().unwrap();
        let x: f32 = content.into_iter().map(|x| x[0] + x[1] + x[2]).sum();
        x / (framenum as f32)
    }
    /// when `scene` is not None, a new scene will be uploaded
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
                .add_image(self.image.clone()).unwrap()
                .add_buffer(self.input_pool.next(input).unwrap()).unwrap()
                .add_buffer(self.spheres.clone()).unwrap()
                .add_buffer(self.planes.clone()).unwrap()
                .add_buffer(self.triangles.clone()).unwrap()
                .add_buffer(self.accum.clone()).unwrap()
                .build()
                .unwrap(),
        )
    }
}
