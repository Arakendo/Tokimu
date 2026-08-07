use std::{
    fmt,
    time::{Duration, Instant},
};

mod fixture_producer;
#[cfg(not(target_arch = "wasm32"))]
mod native;
mod presentation;
#[cfg(feature = "ratatui-producer")]
mod ratatui_producer;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct SurfaceExtent {
    columns: u16,
    rows: u16,
}

impl SurfaceExtent {
    fn cell_count(self) -> usize {
        usize::from(self.columns) * usize::from(self.rows)
    }

    fn contains(self, column: u16, row: u16) -> bool {
        column < self.columns && row < self.rows
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum CellContent {
    Grapheme(String),
    Continuation,
    Empty,
}

impl CellContent {
    fn grapheme(value: impl Into<String>) -> Self {
        Self::Grapheme(value.into())
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct ResolvedCell {
    column: u16,
    row: u16,
    content: CellContent,
    style: ResolvedCellStyle,
}

/// Provider-resolved appearance facts retained with a terminal cell.
///
/// This is intentionally a corpus-local record. It describes a resolved
/// surface without admitting a public terminal styling contract.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
struct ResolvedCellStyle {
    foreground: SurfaceColor,
    background: SurfaceColor,
    emphasis: SurfaceEmphasis,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
enum SurfaceColor {
    #[default]
    Default,
    Named(NamedSurfaceColor),
    Indexed(u8),
    Rgb {
        red: u8,
        green: u8,
        blue: u8,
    },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum NamedSurfaceColor {
    Black,
    Red,
    Green,
    Yellow,
    Blue,
    Magenta,
    Cyan,
    Gray,
    DarkGray,
    LightRed,
    LightGreen,
    LightYellow,
    LightBlue,
    LightMagenta,
    LightCyan,
    White,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
struct SurfaceEmphasis {
    bold: bool,
    dim: bool,
    italic: bool,
    underlined: bool,
    reversed: bool,
    hidden: bool,
    crossed_out: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct RetainedCell {
    content: CellContent,
    style: ResolvedCellStyle,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct CursorState {
    column: u16,
    row: u16,
    visible: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct FullFrame {
    epoch: u64,
    extent: SurfaceExtent,
    cells: Vec<ResolvedCell>,
    cursor: CursorState,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct ChangedCells {
    epoch: u64,
    extent: SurfaceExtent,
    cells: Vec<ResolvedCell>,
    cursor: CursorState,
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum SurfaceUpdate {
    Full(FullFrame),
    Delta(ChangedCells),
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct TerminalSurfaceObservation {
    epoch: u64,
    extent: SurfaceExtent,
    cells: Vec<Option<RetainedCell>>,
    cursor: CursorState,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
struct SurfaceDamageStatistics {
    full_frames: u64,
    delta_frames: u64,
    full_cells: u64,
    changed_cells: u64,
}

#[derive(Default)]
struct TerminalSurfaceReplica {
    current: Option<TerminalSurfaceObservation>,
    damage: SurfaceDamageStatistics,
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum SurfaceApplyError {
    MissingBaseline,
    EpochMismatch {
        expected: u64,
        received: u64,
    },
    ExtentMismatch {
        expected: SurfaceExtent,
        received: SurfaceExtent,
    },
    CellOutsideExtent {
        column: u16,
        row: u16,
    },
    CursorOutsideExtent {
        column: u16,
        row: u16,
    },
    OrphanContinuation {
        column: u16,
        row: u16,
    },
    ContinuationOverwrite {
        column: u16,
        row: u16,
    },
    DuplicateCellDamage {
        column: u16,
        row: u16,
    },
}

impl fmt::Display for SurfaceApplyError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{self:?}")
    }
}

impl TerminalSurfaceObservation {
    fn from_full(frame: FullFrame) -> Result<Self, SurfaceApplyError> {
        let mut observation = Self {
            epoch: frame.epoch,
            extent: frame.extent,
            cells: vec![None; frame.extent.cell_count()],
            cursor: frame.cursor,
        };
        observation.apply_cells(&frame.cells)?;
        observation.validate_cursor(frame.cursor)?;
        Ok(observation)
    }

    fn apply_delta(&mut self, change: ChangedCells) -> Result<(), SurfaceApplyError> {
        if self.epoch != change.epoch {
            return Err(SurfaceApplyError::EpochMismatch {
                expected: self.epoch,
                received: change.epoch,
            });
        }
        if self.extent != change.extent {
            return Err(SurfaceApplyError::ExtentMismatch {
                expected: self.extent,
                received: change.extent,
            });
        }
        self.apply_cells(&change.cells)?;
        self.validate_cursor(change.cursor)?;
        self.cursor = change.cursor;
        Ok(())
    }

    fn apply_cells(&mut self, cells: &[ResolvedCell]) -> Result<(), SurfaceApplyError> {
        let mut next_cells = self.cells.clone();
        for (cell_index, cell) in cells.iter().enumerate() {
            if !self.extent.contains(cell.column, cell.row) {
                return Err(SurfaceApplyError::CellOutsideExtent {
                    column: cell.column,
                    row: cell.row,
                });
            }
            let index = self.index(cell.column, cell.row);
            if cells[..cell_index]
                .iter()
                .any(|prior| prior.column == cell.column && prior.row == cell.row)
            {
                return Err(SurfaceApplyError::DuplicateCellDamage {
                    column: cell.column,
                    row: cell.row,
                });
            }
            if matches!(cell.content, CellContent::Grapheme(_))
                && matches!(
                    next_cells[index].as_ref().map(|retained| &retained.content),
                    Some(CellContent::Continuation)
                )
            {
                return Err(SurfaceApplyError::ContinuationOverwrite {
                    column: cell.column,
                    row: cell.row,
                });
            }
            next_cells[index] = Some(RetainedCell {
                content: cell.content.clone(),
                style: cell.style,
            });
        }
        self.validate_continuations(&next_cells)?;
        self.cells = next_cells;
        Ok(())
    }

    fn validate_continuations(
        &self,
        cells: &[Option<RetainedCell>],
    ) -> Result<(), SurfaceApplyError> {
        for row in 0..self.extent.rows {
            for column in 0..self.extent.columns {
                let index = self.index(column, row);
                if matches!(
                    cells[index].as_ref().map(|retained| &retained.content),
                    Some(CellContent::Continuation)
                ) && (column == 0
                    || !matches!(
                        cells[self.index(column - 1, row)]
                            .as_ref()
                            .map(|retained| &retained.content),
                        Some(CellContent::Grapheme(_))
                    ))
                {
                    return Err(SurfaceApplyError::OrphanContinuation { column, row });
                }
            }
        }
        Ok(())
    }

    fn validate_cursor(&self, cursor: CursorState) -> Result<(), SurfaceApplyError> {
        if cursor.visible && !self.extent.contains(cursor.column, cursor.row) {
            return Err(SurfaceApplyError::CursorOutsideExtent {
                column: cursor.column,
                row: cursor.row,
            });
        }
        Ok(())
    }

    fn index(&self, column: u16, row: u16) -> usize {
        usize::from(row) * usize::from(self.extent.columns) + usize::from(column)
    }
}

impl TerminalSurfaceReplica {
    fn apply(
        &mut self,
        update: SurfaceUpdate,
    ) -> Result<&TerminalSurfaceObservation, SurfaceApplyError> {
        match update {
            SurfaceUpdate::Full(frame) => {
                let cell_count = frame.cells.len() as u64;
                self.current = Some(TerminalSurfaceObservation::from_full(frame)?);
                self.damage.full_frames += 1;
                self.damage.full_cells += cell_count;
            }
            SurfaceUpdate::Delta(change) => {
                let cell_count = change.cells.len() as u64;
                let surface = self
                    .current
                    .as_mut()
                    .ok_or(SurfaceApplyError::MissingBaseline)?;
                surface.apply_delta(change)?;
                self.damage.delta_frames += 1;
                self.damage.changed_cells += cell_count;
            }
        }
        Ok(self
            .current
            .as_ref()
            .expect("a full frame establishes a surface"))
    }

    fn damage_statistics(&self) -> SurfaceDamageStatistics {
        self.damage
    }
}

fn independent_fixture_observation(
) -> Result<(TerminalSurfaceObservation, SurfaceDamageStatistics), SurfaceApplyError> {
    let extent = SurfaceExtent {
        columns: 24,
        rows: 6,
    };
    let first = fixture_producer::render_fixture(1, extent, "READY")
        .expect("the bounded fixture should render");
    let second = fixture_producer::render_fixture(1, extent, "DONE")
        .expect("the bounded fixture should render");
    let delta = fixture_producer::changed_cells_between(&first, &second)
        .expect("matching fixture frames should derive a delta");
    let mut replica = TerminalSurfaceReplica::default();
    replica.apply(SurfaceUpdate::Full(first))?;
    let observation = replica.apply(SurfaceUpdate::Delta(delta))?.clone();
    let damage = replica.damage_statistics();

    Ok((observation, damage))
}

fn independent_fixture_raster() -> Result<presentation::TerminalSurfaceRaster, String> {
    let (observation, _) = independent_fixture_observation().map_err(|error| error.to_string())?;
    presentation::rasterize(&observation)
}

/// Selects the corpus-local producer before the shared terminal surface is
/// rasterized. This is not a terminal-provider contract.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum FixtureProducer {
    Independent,
    Ratatui,
}

impl FixtureProducer {
    fn parse(value: &str) -> Result<Self, String> {
        match value {
            "independent" => Ok(Self::Independent),
            "ratatui" => Ok(Self::Ratatui),
            _ => Err(format!(
                "unknown terminal fixture producer `{value}`; expected `independent` or `ratatui`"
            )),
        }
    }

    const fn name(self) -> &'static str {
        match self {
            Self::Independent => "independent",
            Self::Ratatui => "ratatui",
        }
    }

    fn raster(self) -> Result<presentation::TerminalSurfaceRaster, String> {
        match self {
            Self::Independent => independent_fixture_raster(),
            Self::Ratatui => {
                #[cfg(feature = "ratatui-producer")]
                {
                    ratatui_fixture_raster()
                }
                #[cfg(not(feature = "ratatui-producer"))]
                {
                    Err(
                        "the Ratatui fixture requires the `ratatui-producer` feature; rebuild the terminal corpus with that feature enabled"
                            .to_owned(),
                    )
                }
            }
        }
    }

    fn summary(self) -> Result<String, String> {
        match self {
            Self::Independent => independent_fixture_summary().map_err(|error| error.to_string()),
            Self::Ratatui => {
                #[cfg(feature = "ratatui-producer")]
                {
                    let (observation, damage) =
                        ratatui_fixture_observation().map_err(|error| error.to_string())?;
                    let raster = presentation::rasterize(&observation)?;
                    Ok(format!(
                        "producer={}, epoch={}, extent={}x{}, retained_cells={}, cursor=({}, {}), full_frames={}, delta_frames={}, full_cells={}, changed_cells={}, cpu_raster={}x{}, cpu_fingerprint={:016x}, font=DepartureMono-Regular.otf",
                        self.name(),
                        observation.epoch,
                        observation.extent.columns,
                        observation.extent.rows,
                        observation.cells.iter().flatten().count(),
                        observation.cursor.column,
                        observation.cursor.row,
                        damage.full_frames,
                        damage.delta_frames,
                        damage.full_cells,
                        damage.changed_cells,
                        raster.width,
                        raster.height,
                        raster.fingerprint(),
                    ))
                }
                #[cfg(not(feature = "ratatui-producer"))]
                {
                    Err(
                        "the Ratatui fixture requires the `ratatui-producer` feature; rebuild the terminal corpus with that feature enabled"
                            .to_owned(),
                    )
                }
            }
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct PipelineMeasurement {
    iterations: usize,
    elapsed: Duration,
    raster_width: u32,
    raster_height: u32,
    fingerprint: u64,
}

impl PipelineMeasurement {
    fn average_micros(self) -> u128 {
        self.elapsed.as_micros() / self.iterations as u128
    }
}

fn measure_pipeline(
    iterations: usize,
    mut build_raster: impl FnMut() -> Result<presentation::TerminalSurfaceRaster, String>,
) -> Result<PipelineMeasurement, String> {
    if iterations == 0 {
        return Err("pipeline measurement requires at least one iteration".to_owned());
    }

    let started = Instant::now();
    let mut expected = None;
    for _ in 0..iterations {
        let raster = build_raster()?;
        let current = (raster.width, raster.height, raster.fingerprint());
        if let Some(expected) = expected {
            if expected != current {
                return Err(
                    "terminal surface measurement produced unstable raster evidence".to_owned(),
                );
            }
        } else {
            expected = Some(current);
        }
    }
    let (raster_width, raster_height, fingerprint) =
        expected.expect("a nonzero measurement always produces one raster");

    Ok(PipelineMeasurement {
        iterations,
        elapsed: started.elapsed(),
        raster_width,
        raster_height,
        fingerprint,
    })
}

/// Measures repeated rasterization through the corpus-local retained-frame
/// policy. This deliberately proves only CPU-frame reuse: renderer upload and
/// partial-update policies remain separate evidence.
fn measure_cached_pipeline(
    iterations: usize,
    build_observation: impl Fn() -> Result<TerminalSurfaceObservation, String>,
) -> Result<
    (
        PipelineMeasurement,
        presentation::TerminalSurfaceRasterCacheObservations,
    ),
    String,
> {
    let mut cache = presentation::TerminalSurfaceRasterCache::new()?;
    let measurement = measure_pipeline(iterations, || {
        let observation = build_observation()?;
        Ok(cache.rasterize(&observation)?.clone())
    })?;
    Ok((measurement, cache.observations()))
}

#[cfg(feature = "ratatui-producer")]
fn ratatui_fixture_observation(
) -> Result<(TerminalSurfaceObservation, SurfaceDamageStatistics), SurfaceApplyError> {
    let extent = SurfaceExtent {
        columns: 24,
        rows: 6,
    };
    let first = ratatui_producer::render_fixture(2, extent, "READY")
        .expect("the bounded Ratatui fixture should render");
    let second = ratatui_producer::render_fixture(2, extent, "DONE")
        .expect("the bounded Ratatui fixture should render");
    let delta = ratatui_producer::changed_cells_between(&first, &second)
        .expect("matching Ratatui frames should derive a delta");
    let mut replica = TerminalSurfaceReplica::default();
    replica.apply(SurfaceUpdate::Full(first))?;
    let observation = replica.apply(SurfaceUpdate::Delta(delta))?.clone();
    let damage = replica.damage_statistics();

    Ok((observation, damage))
}

#[cfg(feature = "ratatui-producer")]
fn ratatui_fixture_raster() -> Result<presentation::TerminalSurfaceRaster, String> {
    let (observation, _) = ratatui_fixture_observation().map_err(|error| error.to_string())?;
    presentation::rasterize(&observation)
}

fn independent_fixture_summary() -> Result<String, SurfaceApplyError> {
    let (observation, damage) = independent_fixture_observation()?;
    let raster = presentation::rasterize(&observation)
        .expect("the resolved fixture surface should produce bounded CPU evidence");

    Ok(format!(
        "epoch={}, extent={}x{}, retained_cells={}, cursor=({}, {}), full_frames={}, delta_frames={}, full_cells={}, changed_cells={}, cpu_raster={}x{}, cpu_fingerprint={:016x}, font=DepartureMono-Regular.otf",
        observation.epoch,
        observation.extent.columns,
        observation.extent.rows,
        observation.cells.iter().flatten().count(),
        observation.cursor.column,
        observation.cursor.row,
        damage.full_frames,
        damage.delta_frames,
        damage.full_cells,
        damage.changed_cells,
        raster.width,
        raster.height,
        raster.fingerprint(),
    ))
}

fn main() -> Result<(), String> {
    if std::env::args().any(|argument| argument == "--measure") {
        const ITERATIONS: usize = 256;
        let (independent, independent_cache) = measure_cached_pipeline(ITERATIONS, || {
            independent_fixture_observation()
                .map(|(observation, _)| observation)
                .map_err(|error| error.to_string())
        })?;
        println!(
            "hello-terminal-surface measurement: producer=independent, iterations={}, elapsed_us={}, average_us={}, cpu_raster={}x{}, cpu_fingerprint={:016x}, font_provider_loads={}, rasterizations={}, cache_hits={}, full_invalidations={}",
            independent.iterations,
            independent.elapsed.as_micros(),
            independent.average_micros(),
            independent.raster_width,
            independent.raster_height,
            independent.fingerprint,
            independent_cache.font_provider_loads,
            independent_cache.rasterizations,
            independent_cache.cache_hits,
            independent_cache.full_invalidations,
        );

        #[cfg(feature = "ratatui-producer")]
        {
            let (ratatui, ratatui_cache) = measure_cached_pipeline(ITERATIONS, || {
                ratatui_fixture_observation()
                    .map(|(observation, _)| observation)
                    .map_err(|error| error.to_string())
            })?;
            println!(
                "hello-terminal-surface measurement: producer=ratatui, iterations={}, elapsed_us={}, average_us={}, cpu_raster={}x{}, cpu_fingerprint={:016x}, font_provider_loads={}, rasterizations={}, cache_hits={}, full_invalidations={}",
                ratatui.iterations,
                ratatui.elapsed.as_micros(),
                ratatui.average_micros(),
                ratatui.raster_width,
                ratatui.raster_height,
                ratatui.fingerprint,
                ratatui_cache.font_provider_loads,
                ratatui_cache.rasterizations,
                ratatui_cache.cache_hits,
                ratatui_cache.full_invalidations,
            );
        }
        return Ok(());
    }

    #[cfg(not(target_arch = "wasm32"))]
    {
        let producer = std::env::args()
            .find_map(|argument| argument.strip_prefix("--producer=").map(str::to_owned))
            .as_deref()
            .map(FixtureProducer::parse)
            .transpose()?
            .unwrap_or(FixtureProducer::Independent);
        println!("hello-terminal-surface: {}", producer.summary()?);
        native::run(producer)?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn extent() -> SurfaceExtent {
        SurfaceExtent {
            columns: 4,
            rows: 2,
        }
    }

    fn cursor() -> CursorState {
        CursorState {
            column: 0,
            row: 0,
            visible: true,
        }
    }

    #[test]
    fn fixture_producer_selection_is_explicit() {
        assert_eq!(
            FixtureProducer::parse("independent"),
            Ok(FixtureProducer::Independent)
        );
        assert_eq!(
            FixtureProducer::parse("ratatui"),
            Ok(FixtureProducer::Ratatui)
        );
        assert!(FixtureProducer::parse("fallback").is_err());
    }

    #[cfg(not(feature = "ratatui-producer"))]
    #[test]
    fn disabled_ratatui_producer_reports_its_required_feature() {
        let error = FixtureProducer::Ratatui
            .raster()
            .expect_err("the optional producer must not silently fall back");
        assert!(error.contains("ratatui-producer"));
    }

    fn full_frame() -> FullFrame {
        FullFrame {
            epoch: 7,
            extent: extent(),
            cells: vec![ResolvedCell {
                column: 0,
                row: 0,
                content: CellContent::grapheme("A"),
                style: ResolvedCellStyle::default(),
            }],
            cursor: cursor(),
        }
    }

    #[test]
    fn full_frame_then_delta_reconstructs_the_surface() {
        let mut replica = TerminalSurfaceReplica::default();
        replica.apply(SurfaceUpdate::Full(full_frame())).unwrap();
        let surface = replica
            .apply(SurfaceUpdate::Delta(ChangedCells {
                epoch: 7,
                extent: extent(),
                cells: vec![ResolvedCell {
                    column: 1,
                    row: 0,
                    content: CellContent::grapheme("B"),
                    style: ResolvedCellStyle::default(),
                }],
                cursor: CursorState {
                    column: 1,
                    ..cursor()
                },
            }))
            .unwrap();

        assert_eq!(
            surface.cells[0].as_ref().map(|cell| &cell.content),
            Some(&CellContent::grapheme("A"))
        );
        assert_eq!(
            surface.cells[1].as_ref().map(|cell| &cell.content),
            Some(&CellContent::grapheme("B"))
        );
        assert_eq!(surface.cursor.column, 1);
    }

    #[test]
    fn style_only_damage_preserves_provider_resolved_style() {
        let mut replica = TerminalSurfaceReplica::default();
        replica.apply(SurfaceUpdate::Full(full_frame())).unwrap();

        let emphasized = ResolvedCellStyle {
            foreground: SurfaceColor::Named(NamedSurfaceColor::Cyan),
            background: SurfaceColor::Indexed(24),
            emphasis: SurfaceEmphasis {
                bold: true,
                underlined: true,
                ..SurfaceEmphasis::default()
            },
        };
        let surface = replica
            .apply(SurfaceUpdate::Delta(ChangedCells {
                epoch: 7,
                extent: extent(),
                cells: vec![ResolvedCell {
                    column: 0,
                    row: 0,
                    content: CellContent::grapheme("A"),
                    style: emphasized,
                }],
                cursor: cursor(),
            }))
            .unwrap();

        assert_eq!(surface.cells[0].as_ref().unwrap().style, emphasized);
        assert_eq!(replica.damage_statistics().changed_cells, 1);
    }

    #[test]
    fn delta_rejects_an_epoch_or_extent_change() {
        let mut replica = TerminalSurfaceReplica::default();
        let baseline_error = replica
            .apply(SurfaceUpdate::Delta(ChangedCells {
                epoch: 7,
                extent: extent(),
                cells: Vec::new(),
                cursor: cursor(),
            }))
            .unwrap_err();
        assert_eq!(baseline_error, SurfaceApplyError::MissingBaseline);

        replica.apply(SurfaceUpdate::Full(full_frame())).unwrap();
        let epoch_error = replica
            .apply(SurfaceUpdate::Delta(ChangedCells {
                epoch: 8,
                extent: extent(),
                cells: Vec::new(),
                cursor: cursor(),
            }))
            .unwrap_err();
        assert!(matches!(
            epoch_error,
            SurfaceApplyError::EpochMismatch { .. }
        ));

        let extent_error = replica
            .apply(SurfaceUpdate::Delta(ChangedCells {
                epoch: 7,
                extent: SurfaceExtent {
                    columns: 5,
                    rows: 2,
                },
                cells: Vec::new(),
                cursor: cursor(),
            }))
            .unwrap_err();
        assert!(matches!(
            extent_error,
            SurfaceApplyError::ExtentMismatch { .. }
        ));
    }

    #[test]
    fn continuation_cells_remain_layout_metadata() {
        let mut frame = full_frame();
        frame.cells.push(ResolvedCell {
            column: 1,
            row: 0,
            content: CellContent::Continuation,
            style: ResolvedCellStyle::default(),
        });
        let mut replica = TerminalSurfaceReplica::default();
        replica.apply(SurfaceUpdate::Full(frame)).unwrap();

        let error = replica
            .apply(SurfaceUpdate::Delta(ChangedCells {
                epoch: 7,
                extent: extent(),
                cells: vec![ResolvedCell {
                    column: 1,
                    row: 0,
                    content: CellContent::grapheme("x"),
                    style: ResolvedCellStyle::default(),
                }],
                cursor: cursor(),
            }))
            .unwrap_err();
        assert!(matches!(
            error,
            SurfaceApplyError::ContinuationOverwrite { .. }
        ));
    }

    #[test]
    fn clearing_a_lead_requires_clearing_its_continuation() {
        let mut frame = full_frame();
        frame.cells.push(ResolvedCell {
            column: 1,
            row: 0,
            content: CellContent::Continuation,
            style: ResolvedCellStyle::default(),
        });
        let mut replica = TerminalSurfaceReplica::default();
        replica.apply(SurfaceUpdate::Full(frame)).unwrap();

        let error = replica
            .apply(SurfaceUpdate::Delta(ChangedCells {
                epoch: 7,
                extent: extent(),
                cells: vec![ResolvedCell {
                    column: 0,
                    row: 0,
                    content: CellContent::Empty,
                    style: ResolvedCellStyle::default(),
                }],
                cursor: cursor(),
            }))
            .unwrap_err();
        assert!(matches!(
            error,
            SurfaceApplyError::OrphanContinuation { .. }
        ));

        replica
            .apply(SurfaceUpdate::Delta(ChangedCells {
                epoch: 7,
                extent: extent(),
                cells: vec![
                    ResolvedCell {
                        column: 1,
                        row: 0,
                        content: CellContent::Empty,
                        style: ResolvedCellStyle::default(),
                    },
                    ResolvedCell {
                        column: 0,
                        row: 0,
                        content: CellContent::Empty,
                        style: ResolvedCellStyle::default(),
                    },
                ],
                cursor: cursor(),
            }))
            .unwrap();
    }

    #[test]
    fn resize_requires_a_complete_replacement_surface() {
        let mut replica = TerminalSurfaceReplica::default();
        replica.apply(SurfaceUpdate::Full(full_frame())).unwrap();

        let resized_extent = SurfaceExtent {
            columns: 6,
            rows: 3,
        };
        let resized = replica
            .apply(SurfaceUpdate::Full(FullFrame {
                epoch: 8,
                extent: resized_extent,
                cells: vec![ResolvedCell {
                    column: 5,
                    row: 2,
                    content: CellContent::grapheme("R"),
                    style: ResolvedCellStyle::default(),
                }],
                cursor: CursorState {
                    column: 5,
                    row: 2,
                    visible: true,
                },
            }))
            .unwrap();

        assert_eq!(resized.epoch, 8);
        assert_eq!(resized.extent, resized_extent);
        assert_eq!(resized.cells[0], None);
        assert_eq!(
            resized.cells[17].as_ref().map(|cell| &cell.content),
            Some(&CellContent::grapheme("R"))
        );

        let stale_delta = replica
            .apply(SurfaceUpdate::Delta(ChangedCells {
                epoch: 7,
                extent: extent(),
                cells: Vec::new(),
                cursor: cursor(),
            }))
            .unwrap_err();
        assert!(matches!(
            stale_delta,
            SurfaceApplyError::EpochMismatch { .. }
        ));

        assert_eq!(
            replica.damage_statistics(),
            SurfaceDamageStatistics {
                full_frames: 2,
                delta_frames: 0,
                full_cells: 2,
                changed_cells: 0,
            }
        );
    }

    #[test]
    fn invalid_damage_is_rejected_without_mutating_the_surface() {
        let mut replica = TerminalSurfaceReplica::default();
        replica.apply(SurfaceUpdate::Full(full_frame())).unwrap();
        let before = replica.current.clone().unwrap();

        let error = replica
            .apply(SurfaceUpdate::Delta(ChangedCells {
                epoch: 7,
                extent: extent(),
                cells: vec![
                    ResolvedCell {
                        column: 1,
                        row: 0,
                        content: CellContent::grapheme("B"),
                        style: ResolvedCellStyle::default(),
                    },
                    ResolvedCell {
                        column: 1,
                        row: 0,
                        content: CellContent::grapheme("C"),
                        style: ResolvedCellStyle::default(),
                    },
                ],
                cursor: cursor(),
            }))
            .unwrap_err();

        assert!(matches!(
            error,
            SurfaceApplyError::DuplicateCellDamage { column: 1, row: 0 }
        ));
        assert_eq!(replica.current.as_ref(), Some(&before));
    }

    #[test]
    fn independent_measurement_repeats_the_existing_pipeline() {
        let measurement = measure_pipeline(3, independent_fixture_raster).unwrap();
        assert_eq!(measurement.iterations, 3);
        assert_eq!(
            (measurement.raster_width, measurement.raster_height),
            (288, 120)
        );
        assert_ne!(measurement.fingerprint, 0);
    }

    #[cfg(feature = "ratatui-producer")]
    #[test]
    fn ratatui_measurement_repeats_the_existing_pipeline() {
        let measurement = measure_pipeline(3, ratatui_fixture_raster).unwrap();
        assert_eq!(measurement.iterations, 3);
        assert_eq!(
            (measurement.raster_width, measurement.raster_height),
            (288, 120)
        );
        assert_ne!(measurement.fingerprint, 0);
    }
}
