#[macro_use]
extern crate vulkano_shader_derive;
#[macro_use]
extern crate vulkano;
extern crate vulkano_win;
extern crate winit;
extern crate image;


mod shaders;

use std::mem;
use std::sync::Arc;
use vulkano::buffer::BufferUsage;
use vulkano::buffer::cpu_access::CpuAccessibleBuffer;
use vulkano::command_buffer::AutoCommandBufferBuilder;  
use vulkano::command_buffer::CommandBuffer;
use vulkano::command_buffer::DynamicState;
use vulkano::descriptor::descriptor_set::PersistentDescriptorSet;
use vulkano::device::Device;
use vulkano::device::DeviceExtensions;
use vulkano::format::Format;
use vulkano::framebuffer::Framebuffer;
use vulkano::framebuffer::Subpass;
use vulkano::image::Dimensions;
use vulkano::image::StorageImage;
use vulkano::instance::Instance;
use vulkano::instance::PhysicalDevice;
use vulkano::pipeline::ComputePipeline;
use vulkano::pipeline::GraphicsPipeline;
use vulkano::pipeline::viewport::Viewport;
use vulkano::swapchain::AcquireError;
use vulkano::swapchain::Swapchain;
use vulkano::swapchain::SwapchainCreationError;
use vulkano::sync::GpuFuture;
use vulkano_win::VkSurfaceBuild;
use vulkano_win::Window;
use winit::Event;
use winit::EventsLoop;
use winit::WindowBuilder;
use winit::WindowEvent;




fn init_window(instance : Arc<Instance>) -> (EventsLoop, Window) {
    let events_loop = EventsLoop::new();
    let window = WindowBuilder::new()
        .with_decorations(true)
        .with_dimensions(1024, 768)
        .build_vk_surface(&events_loop, instance.clone())
        .expect("failed to build window");
    (events_loop, window)
}

fn get_device(physical: &PhysicalDevice,  window: &Window) -> (std::sync::Arc<vulkano::device::Device>, std::sync::Arc<vulkano::device::Queue>) {
    // find a graphics device that supports drawing to a window surface
    let (graphics_device, mut queues) = {
        let graphical_queue_family = physical
            .queue_families()
            .find(|&q| q.supports_graphics() && window.surface().is_supported(q).unwrap_or(false))
            .expect("couldn't find a graphic queue family");
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

fn main () {
    // find an instance of Vulkan that allows us to draw to a surface
    let instance = Instance::new(
        None, 
        &vulkano_win::required_extensions(),
        None
    ).expect("No instance with surface extension");

    // we select the first graphics device that we find.
    // Perhaps we should do better
    let physical = PhysicalDevice::enumerate(&instance).next() .expect("no graphics device");
    let (mut events_loop, window) = init_window(instance.clone());
    let (device, queue) = get_device(&physical, &window);
    let mut dimensions = [1024, 768];

    let (mut swapchain, mut images) = {
        use vulkano::swapchain::SurfaceTransform;
        use vulkano::swapchain::CompositeAlpha;
        let caps = window.surface()
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
    use shaders::mandelbrot::cs;


    let indices : [ u16; 6 ] = [0, 1, 2, 2, 3, 0] ;

    #[derive(Copy, Clone)]
    struct Vertex {
        position: [f32; 2],
    }
    impl_vertex!(Vertex, position);
    let vertices = [ 
        Vertex { position: [ 1.0,  1.0] },
        Vertex { position: [-1.0,  1.0] },
        Vertex { position: [-1.0, -1.0] },
        Vertex { position: [ 1.0, -1.0] },
    ];

    let index_buffer = CpuAccessibleBuffer::from_iter(device.clone(), BufferUsage::all(), indices.iter().cloned()).unwrap();
    let vertex_buffer = CpuAccessibleBuffer::from_iter(device.clone(), BufferUsage::all(), vertices.iter().cloned()).unwrap();

    let vs = vs::Shader::load(device.clone()).expect("failed to create shader module");
    let fs = fs::Shader::load(device.clone()).expect("failed to create shader module");
    let cs = cs::Shader::load(device.clone()).expect("failed to create shader module");


   let graphics_pipeline = Arc::new(GraphicsPipeline::start()
        .vertex_input_single_buffer::<Vertex>()
        .vertex_shader(vs.main_entry_point(), ())
        .viewports_dynamic_scissors_irrelevant(1)
        .fragment_shader(fs.main_entry_point(), ())
        .render_pass(Subpass::from(renderpass.clone(), 0).unwrap())
        .build(device.clone())
    .unwrap());

    let compute_pipeline = Arc::new(ComputePipeline::new(device.clone(), &cs.main_entry_point(), &()).unwrap());

    let image = StorageImage::new(device.clone(), Dimensions::Dim2d { width: 1024, height: 1024 },
                                Format::R8G8B8A8Unorm, Some(queue.family())).unwrap();


    let params_buffer = CpuAccessibleBuffer::from_data(device.clone(), BufferUsage::all(), cs::ty::Input {
        center: [1.0, 0.0],
        iter: 200,
        scale: 1.0,
    }).unwrap();

    let set = Arc::new(PersistentDescriptorSet::start(compute_pipeline.clone(), 0)
        .add_image(image.clone()).unwrap()
        .add_buffer(params_buffer).unwrap()
        .build().unwrap()
    );

    let command_buffer =
        AutoCommandBufferBuilder::new(device.clone(), queue.family()).unwrap()
            .dispatch([1024 / 8, 1024 / 8, 1], compute_pipeline.clone(), set.clone(), ()).unwrap()
            .build().unwrap();

    let future = command_buffer.execute(queue.clone()).unwrap();


    let sampler = vulkano::sampler::Sampler::new(device.clone(), vulkano::sampler::Filter::Linear,
                                                 vulkano::sampler::Filter::Linear, vulkano::sampler::MipmapMode::Nearest,
                                                 vulkano::sampler::SamplerAddressMode::Repeat,
                                                 vulkano::sampler::SamplerAddressMode::Repeat,
                                                 vulkano::sampler::SamplerAddressMode::Repeat,
                                                 0.0, 1.0, 0.0, 0.0).unwrap();


    // add image to the set
    let set = Arc::new(vulkano::descriptor::descriptor_set::PersistentDescriptorSet::start(graphics_pipeline.clone(), 0)
        .add_sampled_image(image.clone(), sampler.clone()).unwrap()
        .build().unwrap()
    );


    let mut framebuffers: Option<Vec<Arc<vulkano::framebuffer::Framebuffer<_,_>>>> = None;
    let mut recreate_swapchain = false;
    let mut previous_frame_end = Box::new(future) as Box<GpuFuture>;

    loop {

        // It is important to call this function from time to time, otherwise resources will keep
        // accumulating and you will eventually reach an out of memory error.  Calling this
        // function polls various fences in order to determine what the GPU has already processed,
        // and frees the resources that are no longer needed.

        previous_frame_end.cleanup_finished();

        let dynamic_state = DynamicState {
            viewports: Some(vec![Viewport {
                origin: [0.0, 0.0],
                dimensions: [dimensions[0] as f32, dimensions[1] as f32],
                depth_range: 0.0 .. 1.0,
            }]),
            .. DynamicState::none()
        };

        if recreate_swapchain {
            // dimensions = window.surface().capabilities(physical)
            //            .expect("failed to get surface capabilities").current_extent.unwrap();

            let (new_swapchain, new_images) = match swapchain.recreate_with_dimension(dimensions) {
                Ok(r) => r,
                // This error tends to happen when the user is manually resizing the window.
                // Simply restarting the loop is the easiest way to fix this issue.
                Err(SwapchainCreationError::UnsupportedDimensions) => {
                    continue;
                },
                Err(err) => panic!("{:?}", err)
            };

            mem::replace(&mut swapchain, new_swapchain);
            mem::replace(&mut images, new_images);
            framebuffers = None;
            recreate_swapchain = false;
        }

        if framebuffers.is_none() {
            let new_framebuffers = Some(images.iter().map(|image| {
                Arc::new(Framebuffer::start(graphics_pipeline.render_pass().clone())
                         .add(image.clone()).unwrap()
                         .build().unwrap())
            }).collect::<Vec<_>>());
            mem::replace(&mut framebuffers, new_framebuffers);
        }

        let (image_num, acquire_future) =
            match vulkano::swapchain::acquire_next_image( swapchain.clone(), None) {
                Ok(r) => r,
                Err(AcquireError::OutOfDate) => {
                    println!("out of date? lol no");
                    recreate_swapchain = true;
                    continue;
                },
                Err(err) => panic!("{:?}", err),
            };

        let command_buffer = AutoCommandBufferBuilder
            ::new(
                device.clone(),
                queue.family(),
            ).unwrap()
            .begin_render_pass(
                framebuffers.as_ref().unwrap()[image_num].clone(),
                false,
                vec![[0.0, 0.0, 0.0, 1.0].into(), 1.0.into()],
            ).unwrap()
            .draw_indexed(
                graphics_pipeline.clone(),
                dynamic_state,
                vertex_buffer.clone(),
                index_buffer.clone(),
                set.clone(),
                (),
            ).unwrap()
            .end_render_pass().unwrap()
            .build().unwrap();

        let future = previous_frame_end.join(acquire_future)
            .then_execute(queue.clone(), command_buffer).unwrap()

            // The color output is now expected to contain our triangle. But in order to show it on
            // the screen, we have to *present* the image by calling `present`.
            //
            // This function does not actually present the image immediately. Instead it submits a
            // present command at the end of the queue. This means that it will only be presented once
            // the GPU has finished executing the command buffer that draws the triangle.
            .then_swapchain_present(queue.clone(), swapchain.clone(), image_num)
            .then_signal_fence_and_flush().unwrap();
        previous_frame_end = Box::new(future) as Box<_>;

         // TODO this is probably wrong
        events_loop.poll_events(|event| {
            match event {
                Event::WindowEvent{event, ..} => match event {
                    WindowEvent::Resized(width, height) => {
                        dimensions = [width, height];
                        recreate_swapchain = true;
                    },
                    WindowEvent::KeyboardInput{input,..} => {
                    },
                    _ => {},
                },
                // TODO: handle events so that we can control the camera
                _ => {},
            }
        });
    }
}
