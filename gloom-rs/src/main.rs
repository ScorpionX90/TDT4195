// Uncomment these following global attributes to silence most warnings of "low" interest:
/*
#![allow(dead_code)]
#![allow(non_snake_case)]
#![allow(unreachable_code)]
#![allow(unused_mut)]
#![allow(unused_unsafe)]
#![allow(unused_variables)]
*/
extern crate nalgebra_glm as glm;
use std::{ mem, ptr, os::raw::c_void };
use std::thread;
use std::sync::{Mutex, Arc, RwLock};

mod shader;
mod util;
mod mesh;
mod scene_graph;
mod toolbox;

use glm::identity;
use glutin::event::{Event, WindowEvent, DeviceEvent, KeyboardInput, ElementState::{Pressed, Released}, VirtualKeyCode::{self, *}};
use glutin::event_loop::ControlFlow;
use scene_graph::SceneNode;

use crate::mesh::{Helicopter, Terrain};

// initial window size
const INITIAL_SCREEN_W: u32 = 800;
const INITIAL_SCREEN_H: u32 = 600;

// == // Helper functions to make interacting with OpenGL a little bit prettier. You *WILL* need these! // == //

// Get the size of an arbitrary array of numbers measured in bytes
// Example usage:  byte_size_of_array(my_array)
fn byte_size_of_array<T>(val: &[T]) -> isize {
    std::mem::size_of_val(&val[..]) as isize
}

// Get the OpenGL-compatible pointer to an arbitrary array of numbers
// Example usage:  pointer_to_array(my_array)
fn pointer_to_array<T>(val: &[T]) -> *const c_void {
    &val[0] as *const T as *const c_void
}

// Get the size of the given type in bytes
// Example usage:  size_of::<u64>()
fn size_of<T>() -> i32 {
    mem::size_of::<T>() as i32
}

// Get an offset in bytes for n units of type T, represented as a relative pointer
// Example usage:  offset::<u64>(4)
// Get an offset in bytes for n units of type T, represented as a relative pointer
// Example usage:  offset::<u64>(4)
fn offset<T>(n: u32) -> *const c_void {
    (n * mem::size_of::<T>() as u32) as *const c_void
}
// ptr::null()


// == // Generate your VAO here
unsafe fn create_vao(vertices: &Vec<f32>, indices: &Vec<u32>, colors: &Vec<f32>, normals: &Vec<f32>, id:u32) -> u32 {
    // generate a VAO and bind it
    let mut vao_id: u32 = id;
    gl::GenVertexArrays(1, &mut vao_id);
    gl::BindVertexArray(vao_id);

    // -- vertex buffer --

    // generate a VBO and bind it
    let mut vertices_vbo_id: u32 = 0;
    gl::GenBuffers(1, &mut vertices_vbo_id);
    gl::BindBuffer(gl::ARRAY_BUFFER, vertices_vbo_id);

    // fill it with data
    gl::BufferData(
        gl::ARRAY_BUFFER, 
        byte_size_of_array(vertices),
        pointer_to_array(vertices), 
        gl::STATIC_DRAW
    );

    // configure a VAP for the data and enable it
    let vertices_index = 0;
    gl::VertexAttribPointer(vertices_index, 3, gl::FLOAT, gl::FALSE, size_of::<f32>()*3, ptr::null());
    gl::EnableVertexAttribArray(vertices_index);

    // -- color buffer --

    // generate a VBO and bind it
    let mut colors_vbo_id: u32 = 1;
    gl::GenBuffers(1, &mut colors_vbo_id);
    gl::BindBuffer(gl::ARRAY_BUFFER, colors_vbo_id);

    // fill it with data
    gl::BufferData(
        gl::ARRAY_BUFFER, 
        byte_size_of_array(colors),
        pointer_to_array(colors), 
        gl::STATIC_DRAW
    );

    // configure a VAP for the data and enable it
    let colors_index = 1;
    gl::VertexAttribPointer(colors_index, 4, gl::FLOAT, gl::FALSE, size_of::<f32>()*4, ptr::null());
    gl::EnableVertexAttribArray(colors_index);

    // -- normal buffer --

    // generate a VBO and bind it
    let mut normals_vbo_id: u32 = 2;
    gl::GenBuffers(1, &mut normals_vbo_id);
    gl::BindBuffer(gl::ARRAY_BUFFER, normals_vbo_id);

    // fill it with data
    gl::BufferData(
        gl::ARRAY_BUFFER,
        byte_size_of_array(normals),
        pointer_to_array(normals),
        gl::STATIC_DRAW
    );

    // configure a VAP for the normal data and enable it
    let normals_index = 2;
    gl::VertexAttribPointer(normals_index, 3, gl::FLOAT, gl::FALSE, size_of::<f32>()*3, ptr::null());
    gl::EnableVertexAttribArray(normals_index);

    // -- index buffer --

    // generate a IBO and bind it
    let mut ibo_id: u32 = 0;
    gl::GenBuffers(1, &mut ibo_id);
    gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, ibo_id);

    // fill it with data
    gl::BufferData(
        gl::ELEMENT_ARRAY_BUFFER, 
        byte_size_of_array(indices), 
        pointer_to_array(indices), 
        gl::STATIC_DRAW
    );

    return vao_id;
}


fn main() {
    // Set up the necessary objects to deal with windows and event handling
    let el = glutin::event_loop::EventLoop::new();
    let wb = glutin::window::WindowBuilder::new()
        .with_title("Gloom-rs")
        .with_transparent(false)
        .with_resizable(true)
        .with_inner_size(glutin::dpi::LogicalSize::new(INITIAL_SCREEN_W, INITIAL_SCREEN_H));
    let cb = glutin::ContextBuilder::new()
        .with_vsync(true);
    let windowed_context = cb.build_windowed(wb, &el).unwrap();
    // Uncomment these if you want to use the mouse for controls, but want it to be confined to the screen and/or invisible.
    // windowed_context.window().set_cursor_grab(true).expect("failed to grab cursor");
    // windowed_context.window().set_cursor_visible(false);

    // Set up a shared vector for keeping track of currently pressed keys
    let arc_pressed_keys = Arc::new(Mutex::new(Vec::<VirtualKeyCode>::with_capacity(10)));
    // Make a reference of this vector to send to the render thread
    let pressed_keys = Arc::clone(&arc_pressed_keys);

    // Set up shared tuple for tracking mouse movement between frames
    let arc_mouse_delta = Arc::new(Mutex::new((0f32, 0f32)));
    // Make a reference of this tuple to send to the render thread
    let mouse_delta = Arc::clone(&arc_mouse_delta);

    // Set up shared tuple for tracking changes to the window size
    let arc_window_size = Arc::new(Mutex::new((INITIAL_SCREEN_W, INITIAL_SCREEN_H, false)));
    // Make a reference of this tuple to send to the render thread
    let window_size = Arc::clone(&arc_window_size);

    // Spawn a separate thread for rendering, so event handling doesn't block rendering
    let render_thread = thread::spawn(move || {
        // Acquire the OpenGL Context and load the function pointers.
        // This has to be done inside of the rendering thread, because
        // an active OpenGL context cannot safely traverse a thread boundary
        let context = unsafe {
            let c = windowed_context.make_current().unwrap();
            gl::load_with(|symbol| c.get_proc_address(symbol) as *const _);
            c
        };

        let mut window_aspect_ratio = INITIAL_SCREEN_W as f32 / INITIAL_SCREEN_H as f32;

        // Set up openGL
        unsafe {
            gl::Enable(gl::DEPTH_TEST);
            gl::DepthFunc(gl::LESS);
            gl::Enable(gl::CULL_FACE);
            gl::Disable(gl::MULTISAMPLE);
            gl::Enable(gl::BLEND);
            gl::BlendFunc(gl::SRC_ALPHA, gl::ONE_MINUS_SRC_ALPHA);
            gl::Enable(gl::DEBUG_OUTPUT_SYNCHRONOUS);
            gl::DebugMessageCallback(Some(util::debug_callback), ptr::null());

            // Print some diagnostics
            println!("{}: {}", util::get_gl_string(gl::VENDOR), util::get_gl_string(gl::RENDERER));
            println!("OpenGL\t: {}", util::get_gl_string(gl::VERSION));
            println!("GLSL\t: {}", util::get_gl_string(gl::SHADING_LANGUAGE_VERSION));
        }

        // == // Set up your VAO around here

        let lunarsurface = Terrain::load(&"resources/lunarsurface.obj");
        let helicopter = Helicopter::load(&"resources/helicopter.obj");
        let heli_body = helicopter.body;
        let heli_door = helicopter.door;
        let heli_main_rotor = helicopter.main_rotor;
        let heli_tail_rotor = helicopter.tail_rotor;

        // 
        let lunar_vao_id = unsafe { 
            create_vao(&lunarsurface.vertices, &lunarsurface.indices, &lunarsurface.colors, &lunarsurface.normals, 1)
        };
        let heli_body_vao_id = unsafe {
            create_vao(&heli_body.vertices, &heli_body.indices, &heli_body.colors, &heli_body.normals, 2)
        };
        let heli_door_vao_id = unsafe {
            create_vao(&heli_door.vertices, &heli_door.indices, &heli_door.colors, &heli_door.normals, 2)
        };
        let heli_main_rotor_vao_id = unsafe {
            create_vao(&heli_main_rotor.vertices, &heli_main_rotor.indices, &heli_main_rotor.colors, &heli_main_rotor.normals, 2)
        };
        let heli_tail_rotor_vao_id = unsafe {
            create_vao(&heli_tail_rotor.vertices, &heli_tail_rotor.indices, &heli_tail_rotor.colors, &heli_tail_rotor.normals, 2)
        };

        // Define scene nodes
        let mut scene_node = SceneNode::new();
        let mut lunar_node = SceneNode::from_vao(lunar_vao_id, lunarsurface.index_count);
        let mut heli_root_node = SceneNode::new();
        let mut heli_body_node = SceneNode::from_vao(heli_body_vao_id, heli_body.index_count);
        let mut heli_door_node = SceneNode::from_vao(heli_door_vao_id, heli_door.index_count);
        let mut heli_main_rotor_node = SceneNode::from_vao(heli_main_rotor_vao_id, heli_main_rotor.index_count);
        let mut heli_tail_rotor_node = SceneNode::from_vao(heli_tail_rotor_vao_id, heli_tail_rotor.index_count);

        // Set scene graph hierarchy
        heli_root_node.add_child(&heli_body_node);
        heli_root_node.add_child(&heli_door_node);
        heli_root_node.add_child(&heli_main_rotor_node);
        heli_root_node.add_child(&heli_tail_rotor_node);
        lunar_node.add_child(&heli_root_node);
        scene_node.add_child(&lunar_node);

        // Set model reference points
        scene_node.reference_point = glm::vec3(0.0f32,0.0f32, 0.0f32);
        lunar_node.reference_point = glm::vec3(0.0f32,0.0f32, 0.0f32);
        heli_root_node.reference_point = glm::vec3(0.0f32, 0.0f32, 0.0f32);
        heli_body_node.reference_point = glm::vec3(0.0f32,0.0f32, 0.0f32);
        heli_door_node.reference_point = glm::vec3(0.0f32, 0.0f32, 0.0f32);
        heli_main_rotor_node.reference_point = glm::vec3(0.0f32, 2.0f32, 0.0f32);
        heli_tail_rotor_node.reference_point = glm::vec3(0.35f32, 2.3f32, 10.4f32);
        
        lunar_node.position.y = -10.0f32;
        heli_main_rotor_node.rotation = glm::vec3(0.0f32, 2.0f32, 0.0f32);
        heli_tail_rotor_node.rotation = glm::vec3(1.0f32, 0.0f32, 0.0f32);

        let mut transformation: glm::Mat4;
        let perspective: glm::Mat4 = glm::perspective(
            window_aspect_ratio,
            120.0f32,
            1.0f32,
            1000.0f32
        );

        let mut camera_position: glm::Vec3 = glm::vec3(0.0, 0.0, 0.0);
        let translation_speed = 30.0f32;
        let rotation_speed = 1.5f32;

        let mut yaw = 0.0f32;
        let mut pitch = 0.0f32;

        // == // Set up your shaders here

        // Basic usage of shader helper:
        // The example code below creates a 'shader' object.
        // It which contains the field `.program_id` and the method `.activate()`.
        // The `.` in the path is relative to `Cargo.toml`.
        // This snippet is not enough to do the exercise, and will need to be modified (outside
        // of just using the correct path), but it only needs to be called once

        let simple_shader = unsafe {
            shader::ShaderBuilder::new()
                .attach_file("./shaders/simple.vert")
                .attach_file("./shaders/simple.frag")
                .link()
        };


        // The main rendering loop
        let first_frame_time = std::time::Instant::now();
        let mut previous_frame_time = first_frame_time;
        loop {
            // Compute time passed since the previous frame and since the start of the program
            let now = std::time::Instant::now();
            let elapsed = now.duration_since(first_frame_time).as_secs_f32();
            let delta_time = now.duration_since(previous_frame_time).as_secs_f32();
            previous_frame_time = now;

            let angle_speed = rotation_speed * delta_time;
            let trans_speed: f32 = translation_speed * delta_time;

            // Handle resize events
            if let Ok(mut new_size) = window_size.lock() {
                if new_size.2 {
                    context.resize(glutin::dpi::PhysicalSize::new(new_size.0, new_size.1));
                    window_aspect_ratio = new_size.0 as f32 / new_size.1 as f32;
                    (*new_size).2 = false;
                    println!("Window was resized to {}x{}", new_size.0, new_size.1);
                    unsafe { gl::Viewport(0, 0, new_size.0 as i32, new_size.1 as i32); }
                }
            }

            // Calculate forward vector, and right vector in x-z plane for relative yaw based translation.
            let f_xz = glm::vec3(-yaw.sin(), yaw.cos(), 0.0);
            let r_xz = glm::rotation2d(glm::half_pi()) * f_xz;

            let forward = glm::vec3(f_xz.x, f_xz.z, f_xz.y);
            let right = glm::vec3(r_xz.x, r_xz.z, r_xz.y);
            // Handle keyboard input
            if let Ok(keys) = pressed_keys.lock() {
                for key in keys.iter() {
                    match key {
                        VirtualKeyCode::W => { 
                            camera_position += forward * trans_speed;
                        }
                        VirtualKeyCode::A => { 
                            camera_position -= right * trans_speed;
                        }
                        VirtualKeyCode::S => { 
                            camera_position -= forward * trans_speed;
                        }
                        VirtualKeyCode::D => { 
                            camera_position += right * trans_speed;
                        }
                        VirtualKeyCode::Space => { 
                            camera_position.y -= trans_speed;
                        }
                        VirtualKeyCode::LShift => { 
                            camera_position.y += trans_speed;
                        }

                        VirtualKeyCode::Down => { 
                            pitch += angle_speed;
                            pitch = pitch.clamp(
                                -std::f32::consts::FRAC_PI_2,
                                std::f32::consts::FRAC_PI_2
                            );
                        }
                        VirtualKeyCode::Up => { 
                            pitch -= angle_speed;
                            pitch = pitch.clamp(
                                -std::f32::consts::FRAC_PI_2,
                                std::f32::consts::FRAC_PI_2
                            );
                        }
                        VirtualKeyCode::Right => { 
                            yaw += angle_speed;
                        }
                        VirtualKeyCode::Left => { 
                            yaw -= angle_speed;
                        }

                        _ => { }
                    }
                }
            }
            // Handle mouse movement. delta contains the x and y movement of the mouse since last frame in pixels
            if let Ok(mut delta) = mouse_delta.lock() {

                // == // Optionally access the accumulated mouse movement between
                // == // frames here with `delta.0` and `delta.1`

                *delta = (0.0, 0.0); // reset when done
            }

            let yaw_matrix = glm::rotation(yaw, &glm::vec3(0.0, 1.0, 0.0));
            let pitch_matrix = glm::rotation(pitch, &glm::vec3(1.0, 0.0, 0.0));
            let translation_matrix = glm::translation(&(camera_position));
            
            // apply yaw dependent pitch first.
            let rotation_matrix = pitch_matrix * yaw_matrix;
            transformation = rotation_matrix * translation_matrix;

            unsafe {
                simple_shader.activate();

                // Clear the color and depth buffers
                gl::ClearColor(0.40, 0.55, 1.0, 1.0); // night sky
                gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
                
                // Apply perspective transformation last
                let transform_thus_far = perspective * transformation;
                let loc = simple_shader.get_uniform_location("transform");

                let rotor_speed = 60.0f32;
                
                // let iter_heli_heading = toolbox::simple_heading_animation(elapsed);
                // heli_root_node.position.x = iter_heli_heading.x;
                // heli_root_node.position.z = iter_heli_heading.z;
                // heli_root_node.rotation.z = iter_heli_heading.roll;
                // heli_root_node.rotation.y = iter_heli_heading.yaw;
                // heli_root_node.rotation.x = iter_heli_heading.pitch;

                heli_main_rotor_node.rotation.y = elapsed * rotor_speed;
                heli_tail_rotor_node.rotation.x = elapsed * rotor_speed;
                
                unsafe fn draw_scene(node: &scene_graph::SceneNode,
                    view_projection_matrix: &glm::Mat4,
                    mut transformation_so_far: glm::Mat4,
                    loc: i32,
                    elapsed: f32
                ) {
                    
                    let mut model_matrix = identity();
                    // then apply node’s actual translation
                    model_matrix = glm::translate(&model_matrix, &node.position);
                    
                    // translate to pivot, rotate, translate back
                    model_matrix = glm::translate(&model_matrix, &node.reference_point);
                    model_matrix = glm::rotate_x(&model_matrix, node.rotation.x);
                    model_matrix = glm::rotate_y(&model_matrix, node.rotation.y);
                    model_matrix = glm::rotate_z(&model_matrix, node.rotation.z);
                    model_matrix = glm::translate(&model_matrix, &-node.reference_point);

                    // combine with parent
                    transformation_so_far =  transformation_so_far * model_matrix;
                    
                    gl::UniformMatrix4fv(loc, 1, gl::FALSE, (view_projection_matrix * transformation_so_far).as_ptr());
                    
                    if node.index_count >= 0 {
                        gl::BindVertexArray(node.vao_id);
                        gl::DrawElements(gl::TRIANGLES, node.index_count, gl::UNSIGNED_INT, ptr::null());                    
                    }
                    
                    for &child in &node.children {
                        draw_scene(&*child, view_projection_matrix, transformation_so_far, loc, elapsed);
                    }
                }
                
                draw_scene(&scene_node, &transform_thus_far, identity(), loc, elapsed);
                // Display the new color buffer on the display
                context.swap_buffers().unwrap(); // we use "double buffering" to avoid artifacts
            }
        }
    });


    // == //
    // == // From here on down there are only internals.
    // == //


    // Keep track of the health of the rendering thread
    let render_thread_healthy = Arc::new(RwLock::new(true));
    let render_thread_watchdog = Arc::clone(&render_thread_healthy);
    thread::spawn(move || {
        if !render_thread.join().is_ok() {
            if let Ok(mut health) = render_thread_watchdog.write() {
                println!("Render thread panicked!");
                *health = false;
            }
        }
    });

    // Start the event loop -- This is where window events are initially handled
    el.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Wait;

        // Terminate program if render thread panics
        if let Ok(health) = render_thread_healthy.read() {
            if *health == false {
                *control_flow = ControlFlow::Exit;
            }
        }

        match event {
            Event::WindowEvent { event: WindowEvent::Resized(physical_size), .. } => {
                println!("New window size received: {}x{}", physical_size.width, physical_size.height);
                if let Ok(mut new_size) = arc_window_size.lock() {
                    *new_size = (physical_size.width, physical_size.height, true);
                }
            }
            Event::WindowEvent { event: WindowEvent::CloseRequested, .. } => {
                *control_flow = ControlFlow::Exit;
            }
            // Keep track of currently pressed keys to send to the rendering thread
            Event::WindowEvent { event: WindowEvent::KeyboardInput {
                    input: KeyboardInput { state: key_state, virtual_keycode: Some(keycode), .. }, .. }, .. } => {

                if let Ok(mut keys) = arc_pressed_keys.lock() {
                    match key_state {
                        Released => {
                            if keys.contains(&keycode) {
                                let i = keys.iter().position(|&k| k == keycode).unwrap();
                                keys.remove(i);
                            }
                        },
                        Pressed => {
                            if !keys.contains(&keycode) {
                                keys.push(keycode);
                            }
                        }
                    }
                }

                // Handle Escape and Q keys separately
                match keycode {
                    Escape => { *control_flow = ControlFlow::Exit; }
                    Q      => { *control_flow = ControlFlow::Exit; }
                    _      => { }
                }
            }
            Event::DeviceEvent { event: DeviceEvent::MouseMotion { delta }, .. } => {
                // Accumulate mouse movement
                if let Ok(mut position) = arc_mouse_delta.lock() {
                    *position = (position.0 + delta.0 as f32, position.1 + delta.1 as f32);
                }
            }
            _ => { }
        }
    });
}
