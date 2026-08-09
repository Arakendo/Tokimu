use wgpu::util::DeviceExt;

use crate::{Mesh, MeshHandle};

use super::{GpuMesh, GpuVertex, WgpuBackend};

impl WgpuBackend {
    pub fn upload_mesh(&mut self, handle: MeshHandle, mesh: &Mesh) {
        let replaced_existing = self.meshes.contains_key(&handle);
        let vertices: Vec<GpuVertex> = mesh
            .positions
            .iter()
            .copied()
            .zip(mesh.normals.iter().copied())
            .enumerate()
            .map(|(index, (position, normal))| GpuVertex {
                position,
                normal,
                texture_coordinates: mesh
                    .texture_coordinates
                    .get(index)
                    .copied()
                    .unwrap_or([0.0, 0.0]),
            })
            .collect();
        let vertex_buffer = self
            ._device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("tokimu-mesh-vertex-buffer"),
                contents: bytemuck::cast_slice(&vertices),
                usage: wgpu::BufferUsages::VERTEX,
            });

        self.meshes.insert(
            handle,
            GpuMesh {
                vertex_buffer,
                vertex_count: mesh.vertex_count(),
            },
        );
        self.stats.record_mesh_upload(replaced_existing);
    }
}
