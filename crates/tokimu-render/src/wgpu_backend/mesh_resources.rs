use wgpu::util::DeviceExt;

use crate::{Mesh, MeshHandle};

use super::{GpuMesh, GpuVertex, WgpuBackend};

impl WgpuBackend {
    pub fn upload_mesh(&mut self, handle: MeshHandle, mesh: &Mesh) {
        let replaced_existing = self.meshes.contains_key(&handle);
        let gpu_mesh = create_gpu_mesh(&self._device, mesh, "tokimu-mesh-vertex-buffer");

        self.meshes.insert(handle, gpu_mesh);
        self.stats.record_mesh_upload(replaced_existing);
    }
}

pub(super) fn create_gpu_mesh(device: &wgpu::Device, mesh: &Mesh, label: &str) -> GpuMesh {
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
    let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some(label),
        contents: bytemuck::cast_slice(&vertices),
        usage: wgpu::BufferUsages::VERTEX,
    });
    GpuMesh {
        vertex_buffer,
        vertex_count: mesh.vertex_count(),
    }
}
