use crate::shader::Shader;
use crate::scene_graph::{ SceneNode };

use std::collections::HashMap;
use std::ptr;
use std::mem::ManuallyDrop;
use std::pin::Pin;

use crate::mesh::{ Mesh, Terrain, Helicopter };
use crate::toolbox::{self, open_door, AnimCTX};


unsafe fn draw_scene(node: &SceneNode,
    view_projection_matrix: &glm::Mat4,
    mut model_transform: glm::Mat4,
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
    model_transform = model_transform * model_matrix;
    
    let mvp_location = shader.get_uniform_location("mvp_transform");
    gl::UniformMatrix4fv(mvp_location, 1, gl::FALSE, (view_projection_matrix * model_transform).as_ptr());
    let model_location = shader.get_uniform_location("model_transform");
    gl::UniformMatrix4fv(model_location, 1, gl::FALSE, model_transform.as_ptr());
    
    if node.index_count >= 0 {
        gl::BindVertexArray(node.vao_id);
        gl::DrawElements(gl::TRIANGLES, node.index_count, gl::UNSIGNED_INT, ptr::null());                    
    }
    
    for &child in &node.children {
        draw_scene(&*child, view_projection_matrix, model_transform, shader, elapsed);
    }
}

#[derive(Clone, Copy, Eq, PartialEq, Hash)]
pub enum Nodes {
    LunarSurface,
    HeliBody,
    HeliDoor,
    HeliMainRotor,
    HeliTailRotor,
    HeliRoot,
    SceneRoot,
}

pub struct ChaseCamera {
    pub position: glm::Vec3,
    pub target: glm::Vec3,
    pub radius: f32, // set to big number to get normal camera
    pub perspective: glm::Mat4,
    pitch: f32, 
    yaw: f32
}

impl ChaseCamera {
    pub fn new(radius: f32, perspective: glm::Mat4) -> Self {
        ChaseCamera { 
            position: glm::vec3(100.0, 35.0, 0.0), 
            target: glm::vec3(0.0, 0.0, 0.0),
            radius,
            perspective,
            pitch: 0.0,
            yaw: 0.0
        }
    }

    pub fn forward(&self) -> glm::Vec3 {
        let f_xz = glm::vec3(-self.yaw.sin(), self.yaw.cos(), 0.0);
        glm::vec3(f_xz.x, f_xz.z, f_xz.y)
    }

    pub fn right(&self) -> glm::Vec3 {
        let f_xz = glm::vec3(-self.yaw.sin(), self.yaw.cos(), 0.0);
        let r_xz = glm::rotation2d(glm::half_pi()) * f_xz;
        glm::vec3(r_xz.x, r_xz.z, r_xz.y)
    }
}
                
pub struct World {
    pub nodes: HashMap<Nodes, Vec<ManuallyDrop<Pin<Box<SceneNode>>>>>,
    pub anim_ctxs: Vec<AnimCTX>,
    pub camera: ChaseCamera,
    pub model_transformation: glm::Mat4
}

impl World {
    pub fn new(radius: f32, perspective: glm::Mat4) -> Self {
        let meshes = Self::load_models();
        let nodes = Self::setup_scene_graph(&meshes);
        let anim_ctxs:Vec<AnimCTX> = Vec::new();
        let camera = ChaseCamera::new(radius, perspective);
        World { nodes, anim_ctxs, camera, model_transformation: glm::identity() }
    }

    fn load_models() -> HashMap<Nodes, Mesh> {
        let helicopter = Helicopter::load(&"resources/helicopter.obj");
        HashMap::from([
            (Nodes::LunarSurface, Terrain::load(&"resources/lunarsurface.obj")),
            (Nodes::HeliBody, helicopter.body),
            (Nodes::HeliDoor, helicopter.door),
            (Nodes::HeliMainRotor, helicopter.main_rotor),
            (Nodes::HeliTailRotor, helicopter.tail_rotor),
        ])
    }

    fn setup_scene_graph(meshes: &HashMap<Nodes, Mesh>) -> HashMap<Nodes, Vec<ManuallyDrop<Pin<Box<SceneNode>>>>> {
        let num_helicopters = 1;
        let mut nodes: HashMap<Nodes, Vec<ManuallyDrop<Pin<Box<SceneNode>>>>> = HashMap::new();

        nodes.insert(Nodes::SceneRoot, vec![SceneNode::new()]);

        let lunar_mesh = &meshes[&Nodes::LunarSurface];
        let mut lunar_node = SceneNode::from_vao(lunar_mesh.vao_id, lunar_mesh.index_count);
        lunar_node.reference_point = glm::vec3(0.0, 0.0, 0.0);
        nodes.insert(Nodes::LunarSurface, vec![lunar_node]);

        nodes.insert(Nodes::HeliRoot, Vec::new());
        nodes.insert(Nodes::HeliBody, Vec::new());
        nodes.insert(Nodes::HeliDoor, Vec::new());
        nodes.insert(Nodes::HeliMainRotor, Vec::new());
        nodes.insert(Nodes::HeliTailRotor, Vec::new());

        for _i in 0..num_helicopters {
            let mut heli_root = SceneNode::new();
            heli_root.reference_point = glm::vec3(0.0, 0.0, 0.0);

            let mut heli_body = SceneNode::from_vao(meshes[&Nodes::HeliBody].vao_id, meshes[&Nodes::HeliBody].index_count);
            heli_body.reference_point = glm::vec3(0.0, 0.0, 0.0);

            let mut heli_door = SceneNode::from_vao(meshes[&Nodes::HeliDoor].vao_id, meshes[&Nodes::HeliDoor].index_count);
            heli_door.reference_point = glm::vec3(0.0, 0.0, 0.0);

            let mut heli_main_rotor = SceneNode::from_vao(meshes[&Nodes::HeliMainRotor].vao_id, meshes[&Nodes::HeliMainRotor].index_count);
            heli_main_rotor.reference_point = glm::vec3(0.0, 2.0, 0.0);
            heli_main_rotor.rotation = glm::vec3(0.0, 2.0, 0.0);

            let mut heli_tail_rotor = SceneNode::from_vao(meshes[&Nodes::HeliTailRotor].vao_id, meshes[&Nodes::HeliTailRotor].index_count);
            heli_tail_rotor.reference_point = glm::vec3(0.35, 2.3, 10.4);
            heli_tail_rotor.rotation = glm::vec3(1.0, 0.0, 0.0);

            nodes.get_mut(&Nodes::HeliRoot).unwrap().push(heli_root);
            nodes.get_mut(&Nodes::HeliBody).unwrap().push(heli_body);
            nodes.get_mut(&Nodes::HeliDoor).unwrap().push(heli_door);
            nodes.get_mut(&Nodes::HeliMainRotor).unwrap().push(heli_main_rotor);
            nodes.get_mut(&Nodes::HeliTailRotor).unwrap().push(heli_tail_rotor);
        }

        for i in 0..num_helicopters {
            let heli_body_ptr = nodes[&Nodes::HeliBody][i].as_ref().get_ref() as *const SceneNode;
            let heli_door_ptr = nodes[&Nodes::HeliDoor][i].as_ref().get_ref() as *const SceneNode;
            let heli_main_rotor_ptr = nodes[&Nodes::HeliMainRotor][i].as_ref().get_ref() as *const SceneNode;
            let heli_tail_rotor_ptr = nodes[&Nodes::HeliTailRotor][i].as_ref().get_ref() as *const SceneNode;
            let heli_root_ptr = nodes[&Nodes::HeliRoot][i].as_ref().get_ref() as *const SceneNode;

            unsafe {
                nodes.get_mut(&Nodes::HeliRoot).unwrap()[i].add_child(&*heli_body_ptr);
                nodes.get_mut(&Nodes::HeliRoot).unwrap()[i].add_child(&*heli_door_ptr);
                nodes.get_mut(&Nodes::HeliRoot).unwrap()[i].add_child(&*heli_main_rotor_ptr);
                nodes.get_mut(&Nodes::HeliRoot).unwrap()[i].add_child(&*heli_tail_rotor_ptr);

                nodes.get_mut(&Nodes::LunarSurface).unwrap()[0].add_child(&*heli_root_ptr);
            }
        }

        let lunar_ptr = nodes[&Nodes::LunarSurface][0].as_ref().get_ref() as *const SceneNode;
        nodes.get_mut(&Nodes::SceneRoot).unwrap()[0].add_child(unsafe { &*lunar_ptr });

        return nodes;
    }

    fn view_matrix(&self) -> glm::Mat4 {
        glm::look_at(
            &self.camera.position,
            &self.camera.target,
            &glm::vec3(0.0, 1.0, 0.0)
        )
    }

    pub fn update(&mut self, elapsed: f32, shader: &Shader) {
        let rotor_speed = 60.0;

        // update all helicopter positions
        for i in 0..self.nodes[&Nodes::HeliRoot].len() {
            let iter_heli_heading = toolbox::simple_heading_animation(elapsed + i as f32 * 1.6f32);
            let heli_root = &mut self.nodes.get_mut(&Nodes::HeliRoot).unwrap()[i];
            heli_root.position.x = iter_heli_heading.x;
            heli_root.position.y = 15.0;
            heli_root.position.z = iter_heli_heading.z;
            heli_root.rotation.z = iter_heli_heading.roll;
            heli_root.rotation.y = iter_heli_heading.yaw;
            heli_root.rotation.x = iter_heli_heading.pitch;

            self.nodes.get_mut(&Nodes::HeliMainRotor).unwrap()[i].rotation.y = elapsed * rotor_speed;
            self.nodes.get_mut(&Nodes::HeliTailRotor).unwrap()[i].rotation.x = elapsed * rotor_speed;
        }

        for ctx in &mut self.anim_ctxs {
            let door_transform = open_door(elapsed, ctx);
            self.nodes.get_mut(&Nodes::HeliDoor).unwrap()[0].position.x = door_transform.x;
            self.nodes.get_mut(&Nodes::HeliDoor).unwrap()[0].position.y = door_transform.y;
            self.nodes.get_mut(&Nodes::HeliDoor).unwrap()[0].position.z = door_transform.y;
            self.nodes.get_mut(&Nodes::HeliDoor).unwrap()[0].rotation.x = door_transform.pitch;
            self.nodes.get_mut(&Nodes::HeliDoor).unwrap()[0].rotation.y = door_transform.yaw;
            self.nodes.get_mut(&Nodes::HeliDoor).unwrap()[0].rotation.z = door_transform.roll;
        }

        // camera chase after target
        let camera_target = &self.nodes[&Nodes::HeliRoot][0];
        self.camera.target = camera_target.position;
        let distance = glm::distance(&self.camera.position, &self.camera.target);
        let direction = glm::normalize(&(self.camera.target - self.camera.position));
        if distance > self.camera.radius {
            self.camera.position += direction * (distance - self.camera.radius);
        }

        // update camera angles
        self.camera.yaw = direction.z.atan2(direction.x) + glm::half_pi::<f32>();
        let horizontal_distance = (direction.x * direction.x + direction.z * direction.z).sqrt();
        self.camera.pitch = direction.y.atan2(horizontal_distance);


        let transform_thus_far = self.camera.perspective * self.view_matrix();

        // Build scene graph on the fly or store scene_root separately
        unsafe {
            draw_scene(&self.nodes[&Nodes::SceneRoot][0], &transform_thus_far, glm::identity(), &shader, elapsed);
        }
    }
}
