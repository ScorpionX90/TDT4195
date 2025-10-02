use crate::shader::Shader;
use crate::scene_graph::SceneNode;

use std::collections::HashMap;
use std::ptr;
use std::mem::ManuallyDrop;
use std::pin::Pin;

use crate::mesh::{ Mesh, Terrain, Helicopter };
use crate::toolbox;


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
                
pub struct World {
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
    let helicopter = Helicopter::load(&"resources/helicopter.obj");
    HashMap::from([
        (Nodes::LunarSurface, Terrain::load(&"resources/lunarsurface.obj")),
        (Nodes::HeliBody, helicopter.body),
        (Nodes::HeliDoor, helicopter.door),
        (Nodes::HeliMainRotor, helicopter.main_rotor),
        (Nodes::HeliTailRotor, helicopter.tail_rotor),
    ])
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

