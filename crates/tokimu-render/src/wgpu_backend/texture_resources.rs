use std::sync::Arc;

use crate::{Rgba8TextureColorSpace, Rgba8TextureDescriptor, Texture, TextureHandle};

use super::texture_support::{
    rgba8_texture_format, validate_legacy_texture_replacement,
    validate_rgba8_render_target_creation, validate_rgba8_texture_creation,
    validate_rgba8_texture_update, write_rgba8_texture,
};
use super::{GpuTexture, GpuTextureRole, WgpuBackend, WgpuBackendError};

impl WgpuBackend {
    /// Creates one RGBA8 texture with explicit color interpretation.
    ///
    /// Creation rejects an existing handle and stores a stable texture/view
    /// identity. Callers that need changing pixels without material rebinding
    /// should create once and use [`Self::update_texture_rgba8`] thereafter.
    pub fn create_texture_rgba8(
        &mut self,
        handle: TextureHandle,
        descriptor: Rgba8TextureDescriptor,
        rgba8: &[u8],
    ) -> Result<(), WgpuBackendError> {
        validate_rgba8_texture_creation(
            handle,
            self.textures.contains_key(&handle),
            descriptor,
            rgba8,
        )?;

        self.allocate_texture_rgba8(handle, descriptor, rgba8, false);
        Ok(())
    }

    /// Allocates one sampleable, renderer-owned RGBA8 render target.
    ///
    /// The returned identity is intentionally the existing opaque
    /// [`TextureHandle`]. It can be sampled by a later material pass, but this
    /// method neither accepts pixel data nor exposes WGPU textures or views.
    /// Render-pass routing remains a separate concern.
    pub fn create_render_target_rgba8(
        &mut self,
        handle: TextureHandle,
        descriptor: Rgba8TextureDescriptor,
    ) -> Result<(), WgpuBackendError> {
        validate_rgba8_render_target_creation(
            handle,
            self.textures.contains_key(&handle),
            descriptor,
        )?;
        self.allocate_render_target_rgba8(handle, descriptor);
        Ok(())
    }

    /// Rewrites all pixels in an existing RGBA8 texture without replacing its identity.
    ///
    /// The requested dimensions must exactly match the original descriptor and
    /// `rgba8` must contain the complete payload. Resizing, partial writes, and
    /// color-space changes are intentionally not inferred by this operation.
    pub fn update_texture_rgba8(
        &mut self,
        handle: TextureHandle,
        width: u32,
        height: u32,
        rgba8: &[u8],
    ) -> Result<(), WgpuBackendError> {
        let existing = self.textures.get(&handle);
        if matches!(
            existing.map(|texture| texture.role),
            Some(GpuTextureRole::RenderTarget)
        ) {
            return Err(WgpuBackendError::TextureIsRenderTarget(handle.0));
        }
        let expected = existing.map(|texture| texture.descriptor);
        let expected = validate_rgba8_texture_update(handle, expected, width, height, rgba8)?;
        let gpu_texture = self
            .textures
            .get(&handle)
            .expect("validated texture handle must remain registered");
        write_rgba8_texture(
            &self._queue,
            &gpu_texture.texture,
            expected.width,
            expected.height,
            rgba8,
        );
        self.stats.record_texture_write();
        Ok(())
    }

    /// Compatibility create-or-replace upload using the historical sRGB interpretation.
    ///
    /// New callers that need stable resource identity should use
    /// [`Self::create_texture_rgba8`] followed by [`Self::update_texture_rgba8`].
    /// This bridge remains for existing immutable callers; it may replace a
    /// source texture and therefore does not preserve existing material views.
    /// It panics if asked to replace a renderer-owned render target. New code
    /// should prefer [`Self::try_upload_texture`] to receive that condition as
    /// an explicit error.
    pub fn upload_texture(&mut self, handle: TextureHandle, texture: &Texture) {
        self.try_upload_texture(handle, texture)
            .expect("compatibility texture upload requires a replaceable source texture");
    }

    /// Fallible compatibility create-or-replace upload using the historical
    /// sRGB interpretation.
    ///
    /// This operation deliberately cannot replace renderer-owned render
    /// targets. Their views may already be captured by material bind groups,
    /// so target replacement requires an explicit dependency-rebinding
    /// lifecycle rather than an implicit source-pixel upload.
    pub fn try_upload_texture(
        &mut self,
        handle: TextureHandle,
        texture: &Texture,
    ) -> Result<(), WgpuBackendError> {
        let descriptor = Rgba8TextureDescriptor::new(
            texture.width,
            texture.height,
            Rgba8TextureColorSpace::Srgb,
        );
        descriptor.validate_payload(&texture.rgba8)?;
        validate_legacy_texture_replacement(
            handle,
            self.textures.get(&handle).map(|existing| existing.role),
        )?;
        let replaced_existing = self.textures.contains_key(&handle);
        self.allocate_texture_rgba8(handle, descriptor, &texture.rgba8, replaced_existing);
        Ok(())
    }

    fn allocate_texture_rgba8(
        &mut self,
        handle: TextureHandle,
        descriptor: Rgba8TextureDescriptor,
        rgba8: &[u8],
        replaced_existing: bool,
    ) {
        let gpu_texture = self._device.create_texture(&wgpu::TextureDescriptor {
            label: Some("tokimu-texture"),
            size: wgpu::Extent3d {
                width: descriptor.width,
                height: descriptor.height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: rgba8_texture_format(descriptor.color_space),
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });
        write_rgba8_texture(
            &self._queue,
            &gpu_texture,
            descriptor.width,
            descriptor.height,
            rgba8,
        );
        let view = Arc::new(gpu_texture.create_view(&wgpu::TextureViewDescriptor::default()));
        self.textures.insert(
            handle,
            GpuTexture {
                texture: gpu_texture,
                view,
                descriptor,
                role: GpuTextureRole::Source,
            },
        );
        self.stats.record_texture_allocation(replaced_existing);
        self.stats.record_texture_write();
    }

    fn allocate_render_target_rgba8(
        &mut self,
        handle: TextureHandle,
        descriptor: Rgba8TextureDescriptor,
    ) {
        let gpu_texture = self._device.create_texture(&wgpu::TextureDescriptor {
            label: Some("tokimu-render-target"),
            size: wgpu::Extent3d {
                width: descriptor.width,
                height: descriptor.height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: rgba8_texture_format(descriptor.color_space),
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::RENDER_ATTACHMENT,
            view_formats: &[],
        });
        let view = Arc::new(gpu_texture.create_view(&wgpu::TextureViewDescriptor::default()));
        self.textures.insert(
            handle,
            GpuTexture {
                texture: gpu_texture,
                view,
                descriptor,
                role: GpuTextureRole::RenderTarget,
            },
        );
        self.stats.record_texture_allocation(false);
    }
}
