use macaw::{Vec3, vec3};

pub struct Mesh {
    positions: Vec<Vec3>,
    indices: Vec<u32>,
}

impl Mesh {
    pub fn cube() -> Mesh {
        let mut mesh = Mesh {
            positions: Vec::new(),
            indices: Vec::new(),
        };
        // Pushed in pairs of back front, should be identical expect for one dimension
        // front back (Z swaps)
        mesh.push_quad(vec3(0.0, 0.0, 0.0), vec3(1.0, 0.0, 0.0), vec3(1.0, 1.0, 0.0), vec3(0.0, 1.0, 0.0));
        mesh.push_quad(vec3(0.0, 0.0, 1.0), vec3(1.0, 0.0, 1.0), vec3(1.0, 1.0, 1.0), vec3(0.0, 1.0, 1.0));

        // left right (X swaps)
        mesh.push_quad(vec3(0.0, 0.0, 0.0), vec3(0.0, 1.0, 0.0), vec3(0.0, 1.0, 1.0), vec3(0.0, 0.0, 1.0));
        mesh.push_quad(vec3(1.0, 0.0, 0.0), vec3(1.0, 1.0, 0.0), vec3(1.0, 1.0, 1.0), vec3(1.0, 0.0, 1.0));

        // top bottom (Y swaps)
        mesh.push_quad(vec3(0.0, 0.0, 0.0), vec3(0.0, 0.0, 1.0), vec3(1.0, 0.0, 1.0), vec3(1.0, 0.0, 0.0));
        mesh.push_quad(vec3(0.0, 1.0, 0.0), vec3(0.0, 1.0, 1.0), vec3(1.0, 1.0, 1.0), vec3(1.0, 1.0, 0.0));

        mesh
    }

    // points need to be defined CCW
    pub fn push_quad(&mut self, p0: Vec3, p1: Vec3, p2: Vec3, p3: Vec3) {
        let start_index = self.positions.len() as u32;
        self.positions.push(p0);
        self.positions.push(p1);
        self.positions.push(p2);
        self.positions.push(p3);

        let indices: [u32; 6] = [
            0, 1, 3,
            1, 2, 3
        ];
        for i in indices {
            self.indices.push(i + start_index)
        }
    }
}
