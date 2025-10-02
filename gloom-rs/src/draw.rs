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

pub unsafe fn draw_scene(node: &SceneNode,
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

pub struct World {
    meshes: HashMap<Nodes, Mesh>,
    nodes: HashMap<Nodes, SceneNode>,
}

#[derive(Clone, Copy, Eq, PartialEq, Hash)]
enum Nodes {
    LunarSurface=0,
    HeliBody=1,
    HeliDoor=2,
    HeliMainRotor=3,
    HeliTailRotor=4,
    HeliRoot=69,
    SceneRoot=420,
}

pub fn load_models() -> HashMap<Nodes, Mesh> {
    let lunarsurface = Terrain::load(&"resources/lunarsurface.obj");
    let helicopter = Helicopter::load(&"resources/helicopter.obj");
    let heli_body = helicopter.body;
    let heli_door = helicopter.door;
    let heli_main_rotor = helicopter.main_rotor;
    let heli_tail_rotor = helicopter.tail_rotor;
    unsafe { 
        create_vao(&lunarsurface.vertices, &lunarsurface.indices, &lunarsurface.colors, &lunarsurface.normals, Nodes::LunarSurface as u32);
        create_vao(&heli_body.vertices, &heli_body.indices, &heli_body.colors, &heli_body.normals, Nodes::HeliBody as u32);
        create_vao(&heli_door.vertices, &heli_door.indices, &heli_door.colors, &heli_door.normals, Nodes::HeliDoor as u32);
        create_vao(&heli_main_rotor.vertices, &heli_main_rotor.indices, &heli_main_rotor.colors, &heli_main_rotor.normals, Nodes::HeliMainRotor as u32);
        create_vao(&heli_tail_rotor.vertices, &heli_tail_rotor.indices, &heli_tail_rotor.colors, &heli_tail_rotor.normals, Nodes::HeliTailRotor as u32);
    }
    return HashMap::from([ 
        (Nodes::LunarSurface, lunarsurface),
        (Nodes::HeliBody, heli_body),
        (Nodes::HeliDoor, heli_door),
        (Nodes::HeliMainRotor, heli_main_rotor),
        (Nodes::HeliTailRotor, heli_tail_rotor),
    ]);
}

pub fn setup_scene_graph(meshes: HashMap<Nodes, Mesh>) -> HashMap<Nodes, ManuallyDrop<Pin<Box<SceneNode>>>> {
    let mut nodes = HashMap::new();

    // Create all nodes
    for (node, mesh) in &meshes {
        nodes.insert(*node, SceneNode::from_vao(*node as u32, mesh.index_count));
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

