use thiserror::Error;

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Mesh {
    pub positions: Vec<[f32; 3]>,
    pub normals: Vec<[f32; 3]>,
    /// Optional per-vertex texture coordinates. When present, this stream is
    /// aligned one-to-one with [`Self::positions`].
    pub texture_coordinates: Vec<[f32; 2]>,
}

impl Mesh {
    pub fn new(positions: Vec<[f32; 3]>, normals: Vec<[f32; 3]>) -> Self {
        Self {
            positions,
            normals,
            texture_coordinates: Vec::new(),
        }
    }

    /// Adds one provider-neutral UV coordinate for every mesh position.
    ///
    /// Empty coordinates mean the mesh does not supply texture coordinates;
    /// they are accepted for untextured mesh use. A non-empty stream must be
    /// aligned with positions before it can reach a renderer backend.
    pub fn with_texture_coordinates(
        mut self,
        texture_coordinates: Vec<[f32; 2]>,
    ) -> Result<Self, MeshValidationError> {
        validate_texture_coordinates(self.positions.len(), &texture_coordinates)?;
        self.texture_coordinates = texture_coordinates;
        Ok(self)
    }

    /// Returns whether this mesh has a complete supplied texture-coordinate
    /// stream suitable for a shader that declares `TextureCoordinate2`.
    pub fn has_texture_coordinates(&self) -> bool {
        !self.texture_coordinates.is_empty()
            && self.texture_coordinates.len() == self.positions.len()
    }

    /// Assigns one shading normal to every position in a triangle-list mesh.
    ///
    /// Ordered positions still define geometric facing. Callers are responsible
    /// for making the supplied normal agree with that winding, or for documenting
    /// why the shading normal intentionally differs.
    pub fn uniform_normal(positions: Vec<[f32; 3]>, normal: [f32; 3]) -> Self {
        let normals = vec![normal; positions.len()];
        Self::new(positions, normals)
    }

    pub fn triangle() -> Self {
        Self::uniform_normal(
            vec![[0.0, 0.6, 0.0], [-0.6, -0.6, 0.0], [0.6, -0.6, 0.0]],
            [0.0, 0.0, 1.0],
        )
    }

    pub fn quad() -> Self {
        Self::uniform_normal(
            vec![
                [-0.5, 0.5, 0.0],
                [-0.5, -0.5, 0.0],
                [0.5, -0.5, 0.0],
                [-0.5, 0.5, 0.0],
                [0.5, -0.5, 0.0],
                [0.5, 0.5, 0.0],
            ],
            [0.0, 0.0, 1.0],
        )
    }

    pub fn diamond() -> Self {
        Self::uniform_normal(
            vec![
                [0.0, 0.6, 0.0],
                [-0.55, 0.0, 0.0],
                [0.0, -0.6, 0.0],
                [0.0, 0.6, 0.0],
                [0.0, -0.6, 0.0],
                [0.55, 0.0, 0.0],
            ],
            [0.0, 0.0, 1.0],
        )
    }

    pub fn cube() -> Self {
        Self::new(
            vec![
                [-0.5, -0.5, 0.5],
                [0.5, -0.5, 0.5],
                [0.5, 0.5, 0.5],
                [-0.5, -0.5, 0.5],
                [0.5, 0.5, 0.5],
                [-0.5, 0.5, 0.5],
                [0.5, -0.5, -0.5],
                [-0.5, -0.5, -0.5],
                [-0.5, 0.5, -0.5],
                [0.5, -0.5, -0.5],
                [-0.5, 0.5, -0.5],
                [0.5, 0.5, -0.5],
                [-0.5, -0.5, -0.5],
                [-0.5, -0.5, 0.5],
                [-0.5, 0.5, 0.5],
                [-0.5, -0.5, -0.5],
                [-0.5, 0.5, 0.5],
                [-0.5, 0.5, -0.5],
                [0.5, -0.5, 0.5],
                [0.5, -0.5, -0.5],
                [0.5, 0.5, -0.5],
                [0.5, -0.5, 0.5],
                [0.5, 0.5, -0.5],
                [0.5, 0.5, 0.5],
                [-0.5, 0.5, 0.5],
                [0.5, 0.5, 0.5],
                [0.5, 0.5, -0.5],
                [-0.5, 0.5, 0.5],
                [0.5, 0.5, -0.5],
                [-0.5, 0.5, -0.5],
                [-0.5, -0.5, -0.5],
                [0.5, -0.5, -0.5],
                [0.5, -0.5, 0.5],
                [-0.5, -0.5, -0.5],
                [0.5, -0.5, 0.5],
                [-0.5, -0.5, 0.5],
            ],
            vec![
                [0.0, 0.0, 1.0],
                [0.0, 0.0, 1.0],
                [0.0, 0.0, 1.0],
                [0.0, 0.0, 1.0],
                [0.0, 0.0, 1.0],
                [0.0, 0.0, 1.0],
                [0.0, 0.0, -1.0],
                [0.0, 0.0, -1.0],
                [0.0, 0.0, -1.0],
                [0.0, 0.0, -1.0],
                [0.0, 0.0, -1.0],
                [0.0, 0.0, -1.0],
                [-1.0, 0.0, 0.0],
                [-1.0, 0.0, 0.0],
                [-1.0, 0.0, 0.0],
                [-1.0, 0.0, 0.0],
                [-1.0, 0.0, 0.0],
                [-1.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [0.0, 1.0, 0.0],
                [0.0, 1.0, 0.0],
                [0.0, 1.0, 0.0],
                [0.0, 1.0, 0.0],
                [0.0, 1.0, 0.0],
                [0.0, 1.0, 0.0],
                [0.0, -1.0, 0.0],
                [0.0, -1.0, 0.0],
                [0.0, -1.0, 0.0],
                [0.0, -1.0, 0.0],
                [0.0, -1.0, 0.0],
                [0.0, -1.0, 0.0],
            ],
        )
    }

    pub fn vertex_count(&self) -> u32 {
        self.positions.len() as u32
    }
}

/// Provider-neutral mesh input validation errors.
#[derive(Clone, Debug, Eq, Error, PartialEq)]
pub enum MeshValidationError {
    #[error("mesh has {positions} positions but {texture_coordinates} texture coordinates")]
    TextureCoordinateCountMismatch {
        positions: usize,
        texture_coordinates: usize,
    },
}

fn validate_texture_coordinates(
    positions: usize,
    texture_coordinates: &[[f32; 2]],
) -> Result<(), MeshValidationError> {
    if texture_coordinates.is_empty() || texture_coordinates.len() == positions {
        Ok(())
    } else {
        Err(MeshValidationError::TextureCoordinateCountMismatch {
            positions,
            texture_coordinates: texture_coordinates.len(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::{Mesh, MeshValidationError};

    #[test]
    fn built_in_mesh_normals_agree_with_triangle_winding() {
        for (name, mesh) in [
            ("triangle", Mesh::triangle()),
            ("quad", Mesh::quad()),
            ("diamond", Mesh::diamond()),
            ("cube", Mesh::cube()),
        ] {
            assert_eq!(mesh.positions.len(), mesh.normals.len());
            assert_eq!(mesh.positions.len() % 3, 0);
            for triangle_start in (0..mesh.positions.len()).step_by(3) {
                let a = mesh.positions[triangle_start];
                let b = mesh.positions[triangle_start + 1];
                let c = mesh.positions[triangle_start + 2];
                let geometric = cross(subtract(b, a), subtract(c, a));
                for normal in &mesh.normals[triangle_start..triangle_start + 3] {
                    assert!(
                        dot(geometric, *normal) > 0.0,
                        "{name} triangle {} shading normal disagrees with its winding",
                        triangle_start / 3
                    );
                }
            }
        }
    }

    #[test]
    fn texture_coordinates_must_align_with_positions_when_present() {
        let mesh = Mesh::triangle()
            .with_texture_coordinates(vec![[0.0, 0.0]; 3])
            .expect("aligned texture coordinates should be accepted");
        assert!(mesh.has_texture_coordinates());

        assert_eq!(
            Mesh::triangle().with_texture_coordinates(vec![[0.0, 0.0]; 2]),
            Err(MeshValidationError::TextureCoordinateCountMismatch {
                positions: 3,
                texture_coordinates: 2,
            })
        );
        assert!(!Mesh::triangle().has_texture_coordinates());
    }

    fn subtract(left: [f32; 3], right: [f32; 3]) -> [f32; 3] {
        [left[0] - right[0], left[1] - right[1], left[2] - right[2]]
    }

    fn cross(left: [f32; 3], right: [f32; 3]) -> [f32; 3] {
        [
            left[1] * right[2] - left[2] * right[1],
            left[2] * right[0] - left[0] * right[2],
            left[0] * right[1] - left[1] * right[0],
        ]
    }

    fn dot(left: [f32; 3], right: [f32; 3]) -> f32 {
        left[0] * right[0] + left[1] * right[1] + left[2] * right[2]
    }
}
