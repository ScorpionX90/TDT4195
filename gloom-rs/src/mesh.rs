use tobj;

use crate::util;
use std::ptr;

// internal helper
fn generate_color_vec(color: [f32; 4], num: usize) -> Vec<f32> {
    color.iter().cloned().cycle().take(num*4).collect()
}

// Mesh

pub struct Mesh {
    pub vertices    : Vec<f32>,
    pub normals     : Vec<f32>,
    pub colors      : Vec<f32>,
    pub indices     : Vec<u32>,
    pub index_count : i32,
    pub vao_id : u32,
}

impl Mesh {
    pub fn from(mesh: tobj::Mesh, color: [f32; 4]) -> Self {
        let num_verts = mesh.positions.len() / 3;
        let index_count = mesh.indices.len() as i32;
        let mut m = Mesh {
            vertices: mesh.positions,
            normals: mesh.normals,
            indices: mesh.indices,
            colors: generate_color_vec(color, num_verts),
            index_count,
            vao_id: 0,
        };
        unsafe { m.vao_id = m.create_vao(); }
        m
    }

    unsafe fn create_vao(&self) -> u32 {
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
            util::byte_size_of_array(&self.vertices),
            util::pointer_to_array(&self.vertices), 
            gl::STATIC_DRAW
        );

        let vertices_index = 0;
        gl::VertexAttribPointer(vertices_index, 3, gl::FLOAT, gl::FALSE, util::size_of::<f32>()*3, ptr::null());
        gl::EnableVertexAttribArray(vertices_index);

        // -- color buffer --
        let mut colors_vbo_id: u32 = 0;
        gl::GenBuffers(1, &mut colors_vbo_id);
        gl::BindBuffer(gl::ARRAY_BUFFER, colors_vbo_id);

        gl::BufferData(
            gl::ARRAY_BUFFER, 
            util::byte_size_of_array(&self.colors),
            util::pointer_to_array(&self.colors), 
            gl::STATIC_DRAW
        );

        let colors_index = 1;
        gl::VertexAttribPointer(colors_index, 4, gl::FLOAT, gl::FALSE, util::size_of::<f32>()*4, ptr::null());
        gl::EnableVertexAttribArray(colors_index);

        // -- normal buffer --
        let mut normals_vbo_id: u32 = 0;
        gl::GenBuffers(1, &mut normals_vbo_id);
        gl::BindBuffer(gl::ARRAY_BUFFER, normals_vbo_id);

        gl::BufferData(
            gl::ARRAY_BUFFER,
            util::byte_size_of_array(&self.normals),
            util::pointer_to_array(&self.normals),
            gl::STATIC_DRAW
        );

        let normals_index = 2;
        gl::VertexAttribPointer(normals_index, 3, gl::FLOAT, gl::FALSE, util::size_of::<f32>()*3, ptr::null());
        gl::EnableVertexAttribArray(normals_index);

        // -- index buffer --
        let mut ibo_id: u32 = 0;
        gl::GenBuffers(1, &mut ibo_id);
        gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, ibo_id);

        gl::BufferData(
            gl::ELEMENT_ARRAY_BUFFER, 
            util::byte_size_of_array(&self.indices), 
            util::pointer_to_array(&self.indices), 
            gl::STATIC_DRAW
        );

        return vao_id;
    }
}


// Lunar terrain

pub struct Terrain;
impl Terrain {
    pub fn load(path: &str) -> Mesh {
        println!("Loading terrain model...");
        let before = std::time::Instant::now();
        let (models, _materials)
            = tobj::load_obj(path,
                &tobj::LoadOptions{
                    triangulate: true,
                    single_index: true,
                    ..Default::default()
                }
            ).expect("Failed to load terrain model");
        let after = std::time::Instant::now();
        println!("Done in {:.3}ms.", after.duration_since(before).as_micros() as f32 / 1e3);

        if models.len() > 1 || models.len() == 0 {
            panic!("Please use a model with a single mesh!")
            // You could try merging the vertices and indices
            // of the separate meshes into a single mesh.
            // I'll leave that as an optional exercise. ;)
        }

        let terrain = models[0].to_owned();
        println!("Loaded {} with {} points and {} triangles.",
            terrain.name,
            terrain.mesh.positions.len() /3,
            terrain.mesh.indices.len() / 3,
        );

        Mesh::from(terrain.mesh, [1.0, 1.0, 1.0, 1.0])
    }
}


// Helicopter

pub struct Helicopter {
    pub body       : Mesh,
    pub door       : Mesh,
    pub main_rotor : Mesh,
    pub tail_rotor : Mesh,
}

// You can use square brackets to access the components of the helicopter, if you want to use loops!
use std::ops::Index;
impl Index<usize> for Helicopter {
    type Output = Mesh;
    fn index<'a>(&'a self, i: usize) -> &'a Mesh {
        match i {
            0 => &self.body,
            1 => &self.main_rotor,
            2 => &self.tail_rotor,
            3 => &self.door,
            _ => panic!("Invalid index, try [0,3]"),
        }
    }
}

impl Helicopter {
    pub fn load(path: &str) -> Self {
        println!("Loading helicopter model...");
        let before = std::time::Instant::now();
        let (models, _materials)
            = tobj::load_obj(path,
                &tobj::LoadOptions{
                    triangulate: true,
                    single_index: true,
                    ..Default::default()
                }
            ).expect("Failed to load helicopter model");
        let after = std::time::Instant::now();
        println!("Done in {:.3}ms!", after.duration_since(before).as_micros() as f32 / 1e3);

        for model in &models {
            println!("Loaded {} with {} points and {} triangles.", model.name, model.mesh.positions.len() / 3, model.mesh.indices.len() / 3);
        }

        let body_model = models.iter().find(|m| m.name == "Body_body").expect("Incorrect model file!").to_owned();
        let door_model = models.iter().find(|m| m.name == "Door_door").expect("Incorrect model file!").to_owned();
        let main_rotor_model = models.iter().find(|m| m.name == "Main_Rotor_main_rotor").expect("Incorrect model file!").to_owned();
        let tail_rotor_model = models.iter().find(|m| m.name == "Tail_Rotor_tail_rotor").expect("Incorrect model file!").to_owned();

        Helicopter {
            body:       Mesh::from(body_model.mesh,         [0.3, 0.3, 0.3, 1.0]),
            door:       Mesh::from(door_model.mesh,         [0.1, 0.1, 0.3, 1.0]),
            main_rotor: Mesh::from(main_rotor_model.mesh,   [0.3, 0.1, 0.1, 1.0]),
            tail_rotor: Mesh::from(tail_rotor_model.mesh,   [0.1, 0.3, 0.1, 1.0]),
        }
    }
}
