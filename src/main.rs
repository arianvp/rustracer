#[macro_use]
extern crate vulkano_shader_derive;
#[macro_use]
extern crate vulkano;
extern crate vulkano_win;
extern crate winit;


mod shaders;

use vulkano::sync::GpuFuture;
use vulkano::buffer::BufferUsage;
use vulkano::buffer::cpu_access::CpuAccessibleBuffer;
use vulkano::command_buffer::AutoCommandBufferBuilder;
use vulkano::command_buffer::DynamicState;
use vulkano::device::Device;
use vulkano::device::DeviceExtensions;
use vulkano::framebuffer::Framebuffer;
use vulkano::framebuffer::Subpass;
use vulkano::instance::Instance;
use vulkano::instance::PhysicalDevice;
use vulkano::pipeline::GraphicsPipeline;
use vulkano::pipeline::viewport::Viewport;
use vulkano::swapchain::Swapchain;
use vulkano_win::VkSurfaceBuild;
use winit::EventsLoop;
use winit::WindowBuilder;
use std::sync::Arc;

use shaders::vs;
use shaders::fs;

#[derive(Copy, Clone)]
struct Vertex {
    position: [f32; 2],
}

impl_vertex!(Vertex, position);

fn main () {
    // find an instance of Vulkan that allows us to draw to a surface
    let instance = Instance::new(
        None, 
        &vulkano_win::required_extensions(),
        None
    ).expect("No instance with surface extension");

    // we select the first graphics device that we find.
    // Perhaps we should do better
    let physical = PhysicalDevice::enumerate(&instance)
        .next()
        .expect("no graphics device");

    // set up an event loop for resize events and window close events 
    // button presses and what not
    let mut events_loop = EventsLoop::new();


    let window = WindowBuilder::new()
        .with_decorations(true)
        .with_dimensions(1024, 768)
        .build_vk_surface(&events_loop, instance.clone())
        .expect("failed to build window");


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


    let vertex1 = Vertex { position: [-0.5, -0.5] };
    let vertex2 = Vertex { position: [ 0.0,  0.5] };
    let vertex3 = Vertex { position: [ 0.5, -0.25] };


    let vertex_buffer = CpuAccessibleBuffer::from_iter(graphics_device.clone(), BufferUsage::all(),
                                                    vec![vertex1, vertex2, vertex3].into_iter()).unwrap();

    let vs = vs::Shader::load(graphics_device.clone()).expect("failed to create shader module");
    let fs = fs::Shader::load(graphics_device.clone()).expect("failed to create shader module");
    // TODO recreate swapchain on image resize with 
    // https://docs.rs/vulkano/0.7/vulkano/swapchain/index.html
    let (swapchain, images) = {
        use vulkano::swapchain::SurfaceTransform;
        use vulkano::swapchain::CompositeAlpha;
        let caps = window.surface()
            .capabilities(graphics_device.physical_device())
            .expect("failure to get surface capabilities");
        let format = caps.supported_formats[0].0;
        let dimensions = caps.current_extent.unwrap_or([1024, 768]);
        let usage = caps.supported_usage_flags;
        let present = caps.present_modes.iter().next().unwrap();

        Swapchain::new(
            graphics_device.clone(),
            window.surface().clone(),
            caps.min_image_count,
            format,
            dimensions,
            1,
            usage,
            &graphics_queue,
            SurfaceTransform::Identity,
            CompositeAlpha::Opaque,
            present,
            true,
            None,
        ).expect("failed to create swapchain")
    };

    let renderpass = Arc::new(
        single_pass_renderpass!(
            graphics_device.clone(), attachments: {
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

   let pipeline = Arc::new(GraphicsPipeline::start()
        .vertex_input_single_buffer::<Vertex>()
        .vertex_shader(vs.main_entry_point(), ())
        .viewports_dynamic_scissors_irrelevant(1)
        .fragment_shader(fs.main_entry_point(), ())
        .render_pass(Subpass::from(renderpass.clone(), 0).unwrap())
        .build(graphics_device.clone())
    .unwrap());


   let framebuffers: Vec<_> = images
        .iter()
        .map(|image| Arc::new(
            Framebuffer::start(renderpass.clone())
                .add(image.clone()).unwrap()
                .build().unwrap(),
        ))
        .collect();

    loop {
        events_loop.poll_events(|event| {
            match event {
                // TODO: handle events so that we can control the camera
                _ => {},
            }
        });
        let dynamic_state = DynamicState {
            viewports: Some(vec![Viewport {
                origin: [0.0, 0.0],
                dimensions: [1024.0, 768.0],
                depth_range: 0.0 .. 1.0,
            }]),
            .. DynamicState::none()
        };
        let (image_num, acquire_future) =
            vulkano::swapchain::acquire_next_image(
                swapchain.clone(),
                None,
            ).expect("failed to acquire swapchain in time");

        let command_buffer = AutoCommandBufferBuilder
            ::new(
                graphics_device.clone(),
                graphics_queue.family(),
            ).unwrap()
            .begin_render_pass(
                framebuffers[image_num].clone(),
                false,
                vec![[0.0, 0.0, 0.0, 1.0].into(), 1.0.into()],
            ).unwrap()
            .draw(
                pipeline.clone(),
                dynamic_state,
                vertex_buffer.clone(),
                (),
                (),
            ).unwrap()
            .end_render_pass().unwrap()
            .build().unwrap();

        acquire_future
            .then_execute(graphics_queue.clone(), command_buffer).unwrap()
            .then_swapchain_present(graphics_queue.clone(), swapchain.clone(), image_num)
            .then_signal_fence_and_flush().unwrap()
            .wait(None).unwrap();
    }
}
