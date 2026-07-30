use crate::{
    CameraHandle, Color, Instance2d, MaterialHandle, MaterialOverride, MeshHandle, PipelineHandle,
    RenderableHandle,
};

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct ViewportRect {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ClearCommand {
    pub color: Color,
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct DrawMeshCommand {
    pub mesh: MeshHandle,
    pub material: MaterialHandle,
    pub pipeline: PipelineHandle,
    pub instance: Instance2d,
    pub camera: Option<CameraHandle>,
    pub viewport: Option<ViewportRect>,
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct DrawRenderableCommand {
    pub renderable: RenderableHandle,
    pub instance: Instance2d,
    pub camera: Option<CameraHandle>,
    pub viewport: Option<ViewportRect>,
}

/// Draws a mesh with a transient adjustment to its otherwise shared material.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct DrawMeshMaterialOverrideCommand {
    pub draw: DrawMeshCommand,
    pub material_override: MaterialOverride,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum RenderCommand {
    Clear(ClearCommand),
    DrawMesh(DrawMeshCommand),
    DrawMeshMaterialOverride(DrawMeshMaterialOverrideCommand),
    DrawRenderable(DrawRenderableCommand),
}

#[cfg(test)]
mod tests {
    use super::{DrawMeshCommand, DrawMeshMaterialOverrideCommand, RenderCommand};
    use crate::{Color, MaterialHandle, MaterialOverride};

    #[test]
    fn override_draws_keep_their_shared_source_material_explicit() {
        let source = MaterialHandle(41);
        let base_draw = DrawMeshCommand {
            material: source,
            ..DrawMeshCommand::default()
        };
        let selected = DrawMeshMaterialOverrideCommand {
            draw: base_draw,
            material_override: MaterialOverride::with_replacement_color(Color::rgb(0.9, 0.4, 0.1))
                .unwrap(),
        };
        let faded = DrawMeshMaterialOverrideCommand {
            draw: base_draw,
            material_override: MaterialOverride::default()
                .with_opacity_multiplier(0.35)
                .unwrap(),
        };

        let commands = [
            RenderCommand::DrawMeshMaterialOverride(selected),
            RenderCommand::DrawMeshMaterialOverride(faded),
        ];

        let RenderCommand::DrawMeshMaterialOverride(selected) = commands[0] else {
            unreachable!("first command is an override draw");
        };
        let RenderCommand::DrawMeshMaterialOverride(faded) = commands[1] else {
            unreachable!("second command is an override draw");
        };
        assert_eq!(selected.draw.material, source);
        assert_eq!(faded.draw.material, source);
        assert_ne!(selected.material_override, faded.material_override);
    }
}
