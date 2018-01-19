#![feature(fused)]
#![feature(exact_size_is_empty)]

extern crate bvh;
extern crate fps_counter;
#[macro_use]
extern crate vulkano_shader_derive;
#[macro_use]
extern crate vulkano;
extern crate vulkano_win;
extern crate winit;
extern crate image;
extern crate nalgebra;
extern crate scoped_threadpool;

extern crate half;

mod tracer;
mod types;
mod graphics;
mod compute;
mod scene;
use fps_counter::FPSCounter;
use nalgebra::Vector3;
use std::collections::HashSet;
use std::sync::Arc;
use vulkano::buffer::CpuBufferPool;
use vulkano::command_buffer::AutoCommandBufferBuilder;
use vulkano::device::{Device, DeviceExtensions, Queue};
use vulkano::instance::{Instance, PhysicalDevice};
use vulkano::sync::{GpuFuture, now};
use vulkano_win::{VkSurfaceBuild, Window};
use winit::{Event, EventsLoop, WindowBuilder, WindowEvent};


fn init_window(instance: Arc<Instance>) -> (EventsLoop, Window) {
    let events_loop = EventsLoop::new();
    let window = WindowBuilder::new()
        .with_decorations(true)
        .with_dimensions(512, 512)
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

    let mut graphics =
        graphics::GraphicsPart::new(device.clone(), &window, physical.clone(), queue.clone());


    let triangles = vec![
        tracer::ty::Triangle {

            p1: [3.9,  3.4, 3.0],
            p2: [4.9,  3.4, 3.0],
            p3: [3.13, 3.4, 2.0],
            material: tracer::ty::Material {
                diffuse: [17., 17., 17.],
                refl: 0.0,
                emissive: 1,
                _dummy0: [0; 8],
            },
            _dummy0: [0;4],
            _dummy1: [0;4],
            _dummy2: [0;4],
            _dummy3: [0;4],
        },
    ];

    let num_triangles = triangles.len() as u32;

    let planes = vec![
        tracer::ty::Plane {
            normal: [0., 1., 0.],
            d: 0.,
            material: tracer::ty::Material {
                diffuse: [0.7, 0.7, 0.7],
                refl: 0.0,
                emissive: 0,
                _dummy0: [0; 8],
            },
            _dummy0: [0; 4],
        },
        tracer::ty::Plane {
            normal: [0., 0., -1.],
            d: 8.,
            material: tracer::ty::Material {
                diffuse: [0.7, 0.7, 0.7],
                refl: 0.0,
                emissive: 0,
                _dummy0: [0; 8],
            },
            _dummy0: [0; 4],
        },
        tracer::ty::Plane {
            normal: [1., 0., 0.],
            d: 0.,
            material: tracer::ty::Material {
                diffuse: [0.6, 0.0555, 0.062],
                refl: 0.0,
                emissive: 0,
                _dummy0: [0; 8],
            },
            _dummy0: [0; 4],
        },
        tracer::ty::Plane {
            normal: [0., -1., 0.],
            d: 5.49,
            material: tracer::ty::Material {
                diffuse: [0.7, 0.7, 0.7],
                refl: 0.0,
                emissive: 0,
                _dummy0: [0; 8],
            },
            _dummy0: [0; 4],
        },
        tracer::ty::Plane {
            normal: [-1., 0., 0.],
            d: 5.49,
            material: tracer::ty::Material {
                diffuse: [0.0, 0.7, 0.0],
                refl: 0.0,
                emissive: 0,
                _dummy0: [0; 8],
            },
            _dummy0: [0; 4],
        },
    ];

    let num_planes = planes.len() as u32;

    // TODO we should probably arc this?
    let spheres = vec![
        tracer::ty::Sphere {
            position: [4.9, 1.0, 0.3],
            radius: 0.5,
            material: tracer::ty::Material {
                diffuse: [0.7, 0.8, 0.1],
                refl: 0.0,
                emissive: 0,
                _dummy0: [0; 8],
            },
            _dummy0: [0; 4],
        },
        /*tracer::ty::Sphere {
            position: [1.4, 3.9, 1.2],
            radius: 0.7,
            material: tracer::ty::Material {
                diffuse: [27.0, 13.0, 12.0],
                refl: 0.0,
                emissive: 1,
                _dummy0: [0; 8],
            },
            _dummy0: [0; 4],
        },*/
    ];

    let num_spheres = spheres.len() as u32;

    let mut compute = compute::ComputePart::new(
        &device,
        graphics.texture.clone(),
        spheres,
        planes,
        triangles,
        queue.family(),
    );



    let mut previous_frame_end = Box::new(now(device.clone())) as Box<GpuFuture>;

    let mut camera = tracer::ty::Camera::new(
        Vector3::new(2.78, 2.73, -8.0),
        Vector3::new(2.73, 2.73, 0.),
        5.,
    );

    let mut keycodes = HashSet::new();
    let mut frame_num = 1;
    let mut fps_counter = FPSCounter::new();

    loop {
        previous_frame_end.cleanup_finished();

        if graphics.recreate_swapchain(&window) {
            continue;
        }

        graphics.recreate_framebuffers();

        let (image_num, acquire_future) = match graphics.acquire_next_image() {
            Ok(r) => r,
            Err(vulkano::swapchain::AcquireError::OutOfDate) => {
                continue;
            }
            Err(err) => panic!("{:?}", err),
        };


        let cb = {
            let mut cbb = AutoCommandBufferBuilder::new(device.clone(), queue.family()).unwrap();
            cbb = compute.render(
                cbb,
                graphics.dimensions,
                tracer::ty::Input {
                    camera,
                    num_spheres,
                    num_planes,
                    num_triangles,
                    frame_num,
                },
            );
            cbb = graphics.draw(cbb, image_num);
            cbb.build().unwrap()
        };

        let future = previous_frame_end
            .join(acquire_future)
            .then_execute(queue.clone(), cb)
            .unwrap()
            .then_swapchain_present(queue.clone(), graphics.swapchain.clone(), image_num)
            .then_signal_fence_and_flush()
            .unwrap();

        // lets wait until the frame is done, so we can access the buffer
        // and calculate its energy.
        future.wait(None).unwrap();
        let energy = compute.calculate_energy(frame_num);
        println!("{:?}", energy);
        println!("{:?}", fps_counter.tick());

        previous_frame_end = Box::new(future) as Box<_>;


        // TODO this is probably wrong
        events_loop.poll_events(|event| {
            match event {
                Event::WindowEvent { event, .. } => {
                    match event {
                        WindowEvent::Resized(_width, _height) => graphics.recreate_swapchain = true,
                        WindowEvent::KeyboardInput { input, .. } => {
                            match input.state {
                                winit::ElementState::Pressed => {
                                    keycodes.insert(input.virtual_keycode.unwrap());
                                }
                                winit::ElementState::Released => {
                                    keycodes.remove(&input.virtual_keycode.unwrap());
                                }

                            }
                        }
                        _ => {}
                    }
                }
                // TODO: handle events so that we can control the camera
                _ => {}
            }
        });

        if !keycodes.is_empty() {
            camera.handle_input(&keycodes);
            frame_num = 0;
        }
        frame_num += 1;

    }
}
