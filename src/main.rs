
#[macro_use]
extern crate vulkano_shader_derive;
#[macro_use]
extern crate vulkano;
extern crate vulkano_win;
extern crate winit;
extern crate image;
extern crate cgmath;

extern crate simd;
extern crate half;

mod shaders;
mod tracer;
mod vec;

use tracer::camera::Camera;
use tracer::scene::Scene;

use std::time::{Duration, Instant};
use std::mem;
use std::sync::Arc;
use vulkano::sync;
use vulkano::buffer::BufferUsage;
use vulkano::buffer::cpu_access::CpuAccessibleBuffer;
use vulkano::buffer::CpuBufferPool;
use vulkano::command_buffer::AutoCommandBufferBuilder;
use vulkano::command_buffer::DynamicState;
use vulkano::descriptor::descriptor_set::PersistentDescriptorSet;
use vulkano::device::Device;
use vulkano::device::DeviceExtensions;
use vulkano::device::Queue;
use vulkano::format::Format;
use vulkano::framebuffer::Framebuffer;
use vulkano::framebuffer::Subpass;
use vulkano::image::Dimensions;
use vulkano::image::StorageImage;
use vulkano::instance::Instance;
use vulkano::instance::PhysicalDevice;
use vulkano::pipeline::GraphicsPipeline;
use vulkano::pipeline::viewport::Viewport;
use vulkano::sampler::Filter;
use vulkano::sampler::MipmapMode;
use vulkano::sampler::Sampler;
use vulkano::sampler::SamplerAddressMode;
use vulkano::swapchain;
use vulkano::swapchain::AcquireError;
use vulkano::swapchain::CompositeAlpha;
use vulkano::swapchain::SurfaceTransform;
use vulkano::swapchain::Swapchain;
use vulkano::swapchain::SwapchainCreationError;
use vulkano::sync::GpuFuture;
use vulkano_win::VkSurfaceBuild;
use vulkano_win::Window;
use winit::Event;
use winit::EventsLoop;
use winit::WindowBuilder;
use winit::WindowEvent;




fn init_window(instance: Arc<Instance>) -> (EventsLoop, Window) {
    let events_loop = EventsLoop::new();
    let window = WindowBuilder::new()
        .with_decorations(true)
        .with_dimensions(1024, 1024)
        .build_vk_surface(&events_loop, instance.clone())
        .expect("failed to build window");
    (events_loop, window)
}

fn get_device(physical: &PhysicalDevice, window: &Window) -> (Arc<Device>, Arc<Queue>) {
    // find a graphics device that supports drawing to a window surface
    let (graphics_device, mut queues) = {
        let graphical_queue_family = physical
            .queue_families()
            .find(|&q| {
                q.supports_graphics() && window.surface().is_supported(q).unwrap_or(false)
            })
            .expect("couldn't find a graphic queue family");

        // find a device with a swapchain
        let device_ext = DeviceExtensions {
            khr_swapchain: true,
            ..DeviceExtensions::none()
        };
        Device::new(
            physical.clone(),
            physical.supported_features(),
            &device_ext,
            [(graphical_queue_family, 0.5)].iter().cloned(),
        ).expect("failed to create device")
    };

    // we just take the first queue we found. We should do something proper here in the future
    let graphics_queue = queues.next().unwrap();
    (graphics_device, graphics_queue)
}


const WIDTH: usize = 1024;
const HEIGHT: usize = 1024;
fn main() {
    // find an instance of Vulkan that allows us to draw to a surface
    let instance = Instance::new(None, &vulkano_win::required_extensions(), None)
        .expect("No instance with surface extension");

    // we select the first graphics device that we find.
    // TODO Perhaps we should do better
    let physical = PhysicalDevice::enumerate(&instance).next().expect(
        "no graphics device",
    );
    let (mut events_loop, window) = init_window(instance.clone());
    let (device, queue) = get_device(&physical, &window);

    // find a device with a swapchain
    let mut dimensions = [1024, 768];

    let (mut swapchain, mut images) = {
        let caps = window
            .surface()
            .capabilities(device.physical_device())
            .expect("failure to get surface capabilities");
        let format = caps.supported_formats[0].0;
        println!("{:?}", caps.current_extent);
        // dimensions = caps.current_extent.unwrap_or(dimensions);
        let usage = caps.supported_usage_flags;
        let present = caps.present_modes.iter().next().unwrap();

        Swapchain::new(
            device.clone(),
            window.surface().clone(),
            caps.min_image_count,
            format,
            dimensions,
            1,
            usage,
            &queue,
            SurfaceTransform::Identity,
            CompositeAlpha::Opaque,
            present,
            true,
            None,
        ).expect("failed to create swapchain")
    };

    let renderpass = Arc::new(
        single_pass_renderpass!(
            device.clone(), attachments: {
                color: {
                    load: Clear,
                    store: Store,
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

    use shaders::plane::vs;
    use shaders::plane::fs;


    let indices: [u16; 6] = [0, 1, 2, 2, 3, 0];

    #[derive(Copy, Clone)]
    struct Vertex {
        position: [f32; 2],
    }
    impl_vertex!(Vertex, position);
    let vertices = [
        Vertex { position: [1.0, 1.0] },
        Vertex { position: [-1.0, 1.0] },
        Vertex { position: [-1.0, -1.0] },
        Vertex { position: [1.0, -1.0] },
    ];

    let index_buffer =
        CpuAccessibleBuffer::from_iter(device.clone(), BufferUsage::all(), indices.iter().cloned())
            .unwrap();
    let vertex_buffer = CpuAccessibleBuffer::from_iter(
        device.clone(),
        BufferUsage::all(),
        vertices.iter().cloned(),
    ).unwrap();

    let vs = vs::Shader::load(device.clone()).expect("failed to create shader module");
    let fs = fs::Shader::load(device.clone()).expect("failed to create shader module");


    let graphics_pipeline = Arc::new(
        GraphicsPipeline::start()
            .vertex_input_single_buffer::<Vertex>()
            .vertex_shader(vs.main_entry_point(), ())
            .viewports_dynamic_scissors_irrelevant(1)
            .fragment_shader(fs.main_entry_point(), ())
            .render_pass(Subpass::from(renderpass.clone(), 0).unwrap())
            .build(device.clone())
            .unwrap(),
    );


    let image = StorageImage::new(
        device.clone(),
        Dimensions::Dim2d {
            width: WIDTH as u32,
            height: HEIGHT as u32,
        },
        Format::R16G16B16A16Sfloat,
        Some(queue.family()),
    ).unwrap();



    let sampler = Sampler::new(
        device.clone(),
        Filter::Linear,
        Filter::Linear,
        MipmapMode::Nearest,
        SamplerAddressMode::Repeat,
        SamplerAddressMode::Repeat,
        SamplerAddressMode::Repeat,
        0.0,
        1.0,
        0.0,
        0.0,
    ).unwrap();

    // add image to the set
    let graphics_set = Arc::new(
        PersistentDescriptorSet::start(graphics_pipeline.clone(), 0)
            .add_sampled_image(image.clone(), sampler.clone())
            .unwrap()
            .build()
            .unwrap(),
    );


    let mut framebuffers: Option<Vec<Arc<Framebuffer<_, _>>>> = None;
    let mut recreate_swapchain = false;
    let mut previous_frame_end = Box::new(sync::now(Arc::clone(&device))) as Box<GpuFuture>;

    let mut white_buffer: Vec<[half::f16; 4]> = vec![[half::f16::from_f32(0.0), half::f16::from_f32(0.0),half::f16::from_f32(0.0),half::f16::from_f32(0.0)]; WIDTH * HEIGHT];
    let buffer_pool = CpuBufferPool::upload(Arc::clone(&device));

    let scene = Scene::new();
    let mut camera = Camera::new(WIDTH, HEIGHT);


    loop {
        let begin_time = Instant::now();
        previous_frame_end.cleanup_finished();

        tracer::tracer(&camera, &scene, &mut white_buffer);


        let dynamic_state = DynamicState {
            viewports: Some(vec![
                Viewport {
                    origin: [0.0, 0.0],
                    dimensions: [dimensions[0] as f32, dimensions[1] as f32],
                    depth_range: 0.0..1.0,
                },
            ]),
            ..DynamicState::none()
        };

        if recreate_swapchain {
            // dimensions = window.surface().capabilities(physical)
            //            .expect("failed to get surface capabilities").current_extent.unwrap();

            let (new_swapchain, new_images) = match swapchain.recreate_with_dimension(dimensions) {
                Ok(r) => r,
                // TODO, this only happens sometimes. Why?
                // This error tends to happen when the user is manually resizing the window.
                // Simply restarting the loop is the easiest way to fix this issue.
                Err(SwapchainCreationError::UnsupportedDimensions) => {
                    continue;
                }
                Err(err) => panic!("{:?}", err),
            };

            mem::replace(&mut swapchain, new_swapchain);
            mem::replace(&mut images, new_images);
            framebuffers = None;
            recreate_swapchain = false;
        }

        if framebuffers.is_none() {
            let new_framebuffers = Some(
                images
                    .iter()
                    .map(|image| {
                        Arc::new(
                            Framebuffer::start(graphics_pipeline.render_pass().clone())
                                .add(image.clone())
                                .unwrap()
                                .build()
                                .unwrap(),
                        )
                    })
                    .collect::<Vec<_>>(),
            );
            mem::replace(&mut framebuffers, new_framebuffers);
        }

        let (image_num, acquire_future) =
            match swapchain::acquire_next_image(swapchain.clone(), None) {
                Ok(r) => r,
                Err(AcquireError::OutOfDate) => {
                    println!("out of date? lol no");
                    recreate_swapchain = true;
                    continue;
                }
                Err(err) => panic!("{:?}", err),
            };

        // here we do something interesting. We can not  just use `white_buffer`, as
        // its ownership would be moved to the chunk function. However, we can not
        // also borrow white_buffer, as then we also borrow its elements &u8, and
        // there is no AcceptsPixels<&u8> instance. Luckily u8 is Copy, and thus
        // we can iterate over the buffer by reference, but copy the underlying
        // elements, yielding a [u8] instead of an [&u8]
        let sub_buffer = buffer_pool.chunk(white_buffer.iter().cloned()).unwrap();


        let command_buffer =
            AutoCommandBufferBuilder ::new(device.clone(), queue.family()).unwrap()
            .copy_buffer_to_image(sub_buffer.clone(), Arc::clone(&image)).unwrap()
            .begin_render_pass(framebuffers.as_ref().unwrap()[image_num].clone(), false, vec![[0.0, 0.0, 0.0, 1.0].into(), 1.0.into()],).unwrap()
            .draw_indexed(graphics_pipeline.clone(), dynamic_state, vertex_buffer.clone(), index_buffer.clone(), graphics_set.clone(), ()).unwrap()
            .end_render_pass().unwrap()
            .build().unwrap();

        let future = previous_frame_end
            .join(acquire_future)
            .then_execute(queue.clone(), command_buffer)
            .unwrap()
            .then_swapchain_present(queue.clone(), swapchain.clone(), image_num)
            .then_signal_fence_and_flush()
            .unwrap();
        previous_frame_end = Box::new(future) as Box<_>;

        // TODO this is probably wrong
        events_loop.poll_events(|event| {
            match event {
                Event::WindowEvent { event, .. } => {
                    match event {
                        WindowEvent::Resized(width, height) => {
                            dimensions = [width, height];
                            recreate_swapchain = true;
                        }
                        WindowEvent::KeyboardInput { input, .. } => {
                            camera.handle_input(input.virtual_keycode.unwrap());
                        }
                        _ => {}
                    }
                }
                // TODO: handle events so that we can control the camera
                _ => {}
            }
        });

        let frame_time = Instant::now().duration_since(begin_time);
        let frame_time_seconds = frame_time.as_secs() as f64 +
            frame_time.subsec_nanos() as f64 / 1_000_000_000.0;
        println!("{:?}", frame_time_seconds);



    }
}
