pub struct ChaseCamera {
    pub position: glm::Vec3,
    pub target: glm::Vec3,
    pub radius: f32,
    pub perspective: glm::Mat4,
}

impl ChaseCamera {
    pub fn new(radius: f32, perspective: glm::Mat4) -> Self {
        ChaseCamera { 
            position: glm::vec3(100.0, 35.0, 0.0), 
            target: glm::vec3(0.0, 0.0, 0.0),
            radius,
            perspective,
        }
    }

    pub fn view_matrix(&self) -> glm::Mat4 {
        glm::look_at(
            &self.position,
            &self.target,
            &self.up()
        )
    }

    pub fn forward(&self) -> glm::Vec3 {
        glm::normalize(&(&self.target - self.position))
    }

    pub fn up(&self) -> glm::Vec3 {
        glm::vec3(0.0, 1.0, 0.0)
    }

    pub fn right(&self) -> glm::Vec3 {
        glm::normalize(&glm::cross(&self.forward(), &self.up()))
    }
}
