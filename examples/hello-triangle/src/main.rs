use tokimu::{ClearCommand, Color, DrawMeshCommand, MeshHandle, RenderCommand, Renderer, WgpuBackend};

fn main() {
    let mut renderer = WgpuBackend::default();
    let accent = Color::rgb(0.1, 0.2, 0.3);

    renderer.begin_frame();
    renderer.submit(&[
        RenderCommand::Clear(ClearCommand {
            color: renderer.clear_color(),
        }),
        RenderCommand::DrawMesh(DrawMeshCommand {
            mesh: MeshHandle(1),
            texture: None,
        }),
    ]);
    let stats = renderer.end_frame();

    println!(
        "Tokimu hello-triangle placeholder: renderer={} clear={:?} accent={:?} draw_calls={}",
        renderer.name(),
        renderer.clear_color(),
        accent,
        stats.draw_calls
    );
}
