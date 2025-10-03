extern crate nalgebra_glm as glm;
use std::f64::consts::PI;

use glm::Vec3;

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
    let x_speed = 90.0;
    let y_speed = 6.0;

    // Local transformation
    let local_offset = glm::vec3(
        ctx.rand_seed * x_speed * t,                    // forward (x)
        y_speed * t - 0.5 * 9.81 * t.powi(2),           // vertical (y)
        ctx.rand_seed * 5.0 * t                         // sideways (z)
    );

    // Locally rotated animation vector
    let rotated_offset = glm::rotate_vec3(
        &local_offset,
        ctx.start_rot.y,                       
        &glm::vec3(0.0, 1.0, 0.0)     // rotate around  world up axis (yaw of helicopter)
    );

    // Global transformation
    let pos = ctx.start_pos + rotated_offset;

    // Update rotation as a function of time (dampened for realism)
    let yaw   = ctx.start_rot.y + ctx.rand_seed  * (x_speed * t/ 4.0) / 5.0;
    let roll  = ctx.start_rot.z * t / 2.0; // adjust if needed
    let pitch = ctx.start_rot.x + ctx.rand_seed  * y_speed / 12.0 * t;

    FullHeading {
        x: pos.x,
        y: pos.y,
        z: pos.z,
        roll,
        pitch,
        yaw,
    }
}

