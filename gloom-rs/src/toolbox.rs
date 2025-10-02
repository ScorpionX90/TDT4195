extern crate nalgebra_glm as glm;
use std::f64::consts::PI;

use glm::Vec3;
use tobj::Mesh;

pub struct Heading {
    pub x     : f32,
    pub z     : f32,
    pub roll  : f32, // measured in radians
    pub pitch : f32, // measured in radians
    pub yaw   : f32, // measured in radians
}

pub fn simple_heading_animation(time: f32) -> Heading {
    let t             = time as f64;
    let step          = 0.05f64;
    let path_size     = 15f64;
    let circuit_speed = 0.8f64;

    let xpos      = path_size * (2.0 * (t+ 0.0) * circuit_speed).sin();
    let xpos_next = path_size * (2.0 * (t+step) * circuit_speed).sin();
    let zpos      = 3.0 * path_size * ((t+ 0.0) * circuit_speed).cos();
    let zpos_next = 3.0 * path_size * ((t+step) * circuit_speed).cos();

    let delta_pos = glm::vec2(xpos_next - xpos, zpos_next - zpos);

    let roll  = (t * circuit_speed).cos() * 0.5;
    let pitch = -0.175 * glm::length(&delta_pos);
    let yaw   = PI + delta_pos.x.atan2(delta_pos.y);

    Heading {
        x     : xpos  as f32,
        z     : zpos  as f32,
        roll  : roll  as f32,
        pitch : pitch as f32,
        yaw   : yaw   as f32,
    }
}


pub struct FullHeading {
    pub x     : f32,
    pub y     : f32,
    pub z     : f32,
    pub roll  : f32,
    pub pitch : f32,
    pub yaw   : f32,
}

pub struct AnimCTX {
    pub stime : f32,
    pub rand_seed: f32,
    pub anim_func: fn(f32, &AnimCTX) -> FullHeading,
    pub start_pos: Vec3,
    pub start_rot: Vec3,
}

pub fn open_door(time: f32, ctx: &AnimCTX) -> FullHeading {
    
    let t = time - ctx.stime;

    let x_speed = 5.0f32;
    let y_speed = 3.0f32;
    let sim_speed = 5.0f32;

    let xpos: f32 = ctx.start_pos.x + 5.0f32 * ctx.rand_seed * x_speed * t;
    let ypos: f32 = ctx.start_pos.y + -0.5* 9.81f32 * y_speed * t.powf(2.0f32);
    let zpos: f32 = ctx.start_pos.z + ctx.rand_seed * 5.0f32 * t;

    let tilt = ctx.start_rot.x + ctx.rand_seed * sim_speed * y_speed / 12.0f32 * t;
    let roll = ctx.start_rot.y + ctx.rand_seed * sim_speed * (x_speed * t) / 5.0f32; 
    let yaw = ctx.start_rot.z + tilt;

    FullHeading {
        x    : xpos as f32,
        y    : ypos as f32,
        z    : zpos as f32,
        roll : roll as f32,
        pitch: tilt as f32,
        yaw  : yaw as f32
    }
}
