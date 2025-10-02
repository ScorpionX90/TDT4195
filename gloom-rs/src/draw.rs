use crate::shader::Shader;
use crate::scene_graph::SceneNode;

use std::collections::HashMap;
use std::{ mem, ptr, os::raw::c_void };
use std::mem::ManuallyDrop;
use std::pin::Pin;

use crate::mesh::{ Mesh, Terrain, Helicopter };
use crate::toolbox;

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

unsafe fn draw_scene(node: &SceneNode,
    view_projection_matrix: &glm::Mat4,
    mut transformation_so_far: glm::Mat4,
    shader: &Shader,
    elapsed: f32
) {
    
    let mut model_matrix = glm::identity();
    // then apply node’s actual translation
    model_matrix = glm::translate(&model_matrix, &node.position);

    // translate to pivot, rotate, translate back
    model_matrix = glm::translate(&model_matrix, &node.reference_point);
    model_matrix = glm::rotate_z(&model_matrix, node.rotation.z);
    model_matrix = glm::rotate_y(&model_matrix, node.rotation.y);
    model_matrix = glm::rotate_x(&model_matrix, node.rotation.x);
    model_matrix = glm::translate(&model_matrix, &-node.reference_point);

    // combine with parent
    transformation_so_far = transformation_so_far * model_matrix;
    
    let loc = shader.get_uniform_location("transform");
    gl::UniformMatrix4fv(loc, 1, gl::FALSE, (view_projection_matrix * transformation_so_far).as_ptr());
    
    if node.index_count >= 0 {
        gl::BindVertexArray(node.vao_id);
        gl::DrawElements(gl::TRIANGLES, node.index_count, gl::UNSIGNED_INT, ptr::null());                    
    }
    
    for &child in &node.children {
        draw_scene(&*child, view_projection_matrix, transformation_so_far, shader, elapsed);
    }
}
                
// draw.rs - Replace create_vao function
unsafe fn create_vao(vertices: &Vec<f32>, indices: &Vec<u32>, colors: &Vec<f32>, normals: &Vec<f32>) -> u32 {
    // Let OpenGL generate the VAO ID
    let mut vao_id: u32 = 0;
    gl::GenVertexArrays(1, &mut vao_id);
    gl::BindVertexArray(vao_id);

    // -- vertex buffer --
    let mut vertices_vbo_id: u32 = 0;
    gl::GenBuffers(1, &mut vertices_vbo_id);
    gl::BindBuffer(gl::ARRAY_BUFFER, vertices_vbo_id);

    gl::BufferData(
        gl::ARRAY_BUFFER, 
        byte_size_of_array(vertices),
        pointer_to_array(vertices), 
        gl::STATIC_DRAW
    );

    let vertices_index = 0;
    gl::VertexAttribPointer(vertices_index, 3, gl::FLOAT, gl::FALSE, size_of::<f32>()*3, ptr::null());
    gl::EnableVertexAttribArray(vertices_index);

    // -- color buffer --
    let mut colors_vbo_id: u32 = 0;
    gl::GenBuffers(1, &mut colors_vbo_id);
    gl::BindBuffer(gl::ARRAY_BUFFER, colors_vbo_id);

    gl::BufferData(
        gl::ARRAY_BUFFER, 
        byte_size_of_array(colors),
        pointer_to_array(colors), 
        gl::STATIC_DRAW
    );

    let colors_index = 1;
    gl::VertexAttribPointer(colors_index, 4, gl::FLOAT, gl::FALSE, size_of::<f32>()*4, ptr::null());
    gl::EnableVertexAttribArray(colors_index);

    // -- normal buffer --
    let mut normals_vbo_id: u32 = 0;
    gl::GenBuffers(1, &mut normals_vbo_id);
    gl::BindBuffer(gl::ARRAY_BUFFER, normals_vbo_id);

    gl::BufferData(
        gl::ARRAY_BUFFER,
        byte_size_of_array(normals),
        pointer_to_array(normals),
        gl::STATIC_DRAW
    );

    let normals_index = 2;
    gl::VertexAttribPointer(normals_index, 3, gl::FLOAT, gl::FALSE, size_of::<f32>()*3, ptr::null());
    gl::EnableVertexAttribArray(normals_index);

    // -- index buffer --
    let mut ibo_id: u32 = 0;
    gl::GenBuffers(1, &mut ibo_id);
    gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, ibo_id);

    gl::BufferData(
        gl::ELEMENT_ARRAY_BUFFER, 
        byte_size_of_array(indices), 
        pointer_to_array(indices), 
        gl::STATIC_DRAW
    );

    return vao_id;
}

pub struct World {
    pub meshes: HashMap<Nodes, Mesh>,
    pub nodes: HashMap<Nodes, ManuallyDrop<Pin<Box<SceneNode>>>>,
}

#[derive(Clone, Copy, Eq, PartialEq, Hash)]
pub enum Nodes {
    LunarSurface=0,
    HeliBody=1,
    HeliDoor=2,
    HeliMainRotor=3,
    HeliTailRotor=4,
    HeliRoot=69,
    SceneRoot=420,
}

pub fn load_models() -> HashMap<Nodes, Mesh> {
    let mut lunarsurface = Terrain::load(&"resources/lunarsurface.obj");
    let helicopter = Helicopter::load(&"resources/helicopter.obj");
    let mut heli_body = helicopter.body;
    let mut heli_door = helicopter.door;
    let mut heli_main_rotor = helicopter.main_rotor;
    let mut heli_tail_rotor = helicopter.tail_rotor;

    unsafe { 
        lunarsurface.vao_id = create_vao(&lunarsurface.vertices, &lunarsurface.indices, &lunarsurface.colors, &lunarsurface.normals);
        heli_body.vao_id = create_vao(&heli_body.vertices, &heli_body.indices, &heli_body.colors, &heli_body.normals);
        heli_door.vao_id = create_vao(&heli_door.vertices, &heli_door.indices, &heli_door.colors, &heli_door.normals);
        heli_main_rotor.vao_id = create_vao(&heli_main_rotor.vertices, &heli_main_rotor.indices, &heli_main_rotor.colors, &heli_main_rotor.normals);
        heli_tail_rotor.vao_id = create_vao(&heli_tail_rotor.vertices, &heli_tail_rotor.indices, &heli_tail_rotor.colors, &heli_tail_rotor.normals);
    }

    return HashMap::from([ 
        (Nodes::LunarSurface, lunarsurface),
        (Nodes::HeliBody, heli_body),
        (Nodes::HeliDoor, heli_door),
        (Nodes::HeliMainRotor, heli_main_rotor),
        (Nodes::HeliTailRotor, heli_tail_rotor),
    ]);
}


pub fn setup_scene_graph(meshes: &HashMap<Nodes, Mesh>) -> HashMap<Nodes, ManuallyDrop<Pin<Box<SceneNode>>>> {
    let mut nodes = HashMap::new();

    // Create all nodes with correct VAO IDs from meshes
    for (node, mesh) in meshes {
        nodes.insert(*node, SceneNode::from_vao(mesh.vao_id, mesh.index_count));
    }

    nodes.insert(Nodes::HeliRoot, SceneNode::new());
    nodes.insert(Nodes::SceneRoot, SceneNode::new());

    // Set reference points and initial state
    nodes.get_mut(&Nodes::LunarSurface).unwrap().reference_point = glm::vec3(0.0, 0.0, 0.0);
    nodes.get_mut(&Nodes::LunarSurface).unwrap().position.y = -10.0;

    nodes.get_mut(&Nodes::HeliRoot).unwrap().reference_point = glm::vec3(0.0, 0.0, 0.0);
    nodes.get_mut(&Nodes::HeliBody).unwrap().reference_point = glm::vec3(0.0, 0.0, 0.0);
    nodes.get_mut(&Nodes::HeliDoor).unwrap().reference_point = glm::vec3(0.0, 0.0, 0.0);
    nodes.get_mut(&Nodes::HeliMainRotor).unwrap().reference_point = glm::vec3(0.0, 2.0, 0.0);
    nodes.get_mut(&Nodes::HeliMainRotor).unwrap().rotation = glm::vec3(0.0, 2.0, 0.0);
    nodes.get_mut(&Nodes::HeliTailRotor).unwrap().reference_point = glm::vec3(0.35, 2.3, 10.4);
    nodes.get_mut(&Nodes::HeliTailRotor).unwrap().rotation = glm::vec3(1.0, 0.0, 0.0);

    // BUILD THE SCENE GRAPH HIERARCHY
    let heli_body_ptr = nodes.get(&Nodes::HeliBody).unwrap().as_ref().get_ref() as *const SceneNode;
    let heli_door_ptr = nodes.get(&Nodes::HeliDoor).unwrap().as_ref().get_ref() as *const SceneNode;
    let heli_main_rotor_ptr = nodes.get(&Nodes::HeliMainRotor).unwrap().as_ref().get_ref() as *const SceneNode;
    let heli_tail_rotor_ptr = nodes.get(&Nodes::HeliTailRotor).unwrap().as_ref().get_ref() as *const SceneNode;
    let heli_root_ptr = nodes.get(&Nodes::HeliRoot).unwrap().as_ref().get_ref() as *const SceneNode;
    let lunar_ptr = nodes.get(&Nodes::LunarSurface).unwrap().as_ref().get_ref() as *const SceneNode;

    nodes.get_mut(&Nodes::HeliRoot).unwrap().add_child(unsafe { &*heli_body_ptr });
    nodes.get_mut(&Nodes::HeliRoot).unwrap().add_child(unsafe { &*heli_door_ptr });
    nodes.get_mut(&Nodes::HeliRoot).unwrap().add_child(unsafe { &*heli_main_rotor_ptr });
    nodes.get_mut(&Nodes::HeliRoot).unwrap().add_child(unsafe { &*heli_tail_rotor_ptr });

    nodes.get_mut(&Nodes::LunarSurface).unwrap().add_child(unsafe { &*heli_root_ptr });
    nodes.get_mut(&Nodes::SceneRoot).unwrap().add_child(unsafe { &*lunar_ptr });

    return nodes;
}



pub fn update(elapsed: f32, world: &mut World, perspective: glm::Mat4, transformation: glm::Mat4, shader: &Shader) {
    let transform_thus_far = perspective * transformation;
    let rotor_speed = 60.0;

    let iter_heli_heading = toolbox::simple_heading_animation(elapsed);

    let heli_root = world.nodes.get_mut(&Nodes::HeliRoot).unwrap();
    heli_root.position.x = iter_heli_heading.x;
    heli_root.position.z = iter_heli_heading.z;
    heli_root.rotation.z = iter_heli_heading.roll;
    heli_root.rotation.y = iter_heli_heading.yaw;
    heli_root.rotation.x = iter_heli_heading.pitch;

    world.nodes.get_mut(&Nodes::HeliMainRotor).unwrap().rotation.y = elapsed * rotor_speed;
    world.nodes.get_mut(&Nodes::HeliTailRotor).unwrap().rotation.x = elapsed * rotor_speed;

    // Build scene graph on the fly or store scene_root separately
    unsafe {draw_scene(&world.nodes[&Nodes::SceneRoot], &transform_thus_far, glm::identity(), &shader, elapsed);}
}

