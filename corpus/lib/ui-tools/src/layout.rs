use crate::{UiRect, UiTheme};

/// Reports whether a layout preserved its requested geometry or required a
/// fallback. A successful layout never reports `Exact` after it has silently
/// compressed content below its measured size.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum UiLayoutFit {
    /// The requested geometry fit without adjustment.
    Exact,
    /// The layout fit only after an explicit, non-overflowing adjustment.
    Adjusted,
    /// The requested geometry was preserved and extends beyond its container.
    Overflow,
    /// The container cannot produce finite usable geometry.
    Impossible,
}

/// Selects how a stack handles content that does not fit on its main axis.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum UiOverflowPolicy {
    /// Preserve the historical contained result by proportionally compressing
    /// children. The resolved result reports `UiLayoutFit::Adjusted`.
    #[default]
    Compress,
    /// Preserve measured child sizes and report overflow for the caller to
    /// address with scrolling, clipping, or a compact presentation.
    Preserve,
}

/// A viewport-aware application frame with distinct header, body, and footer
/// regions. Consumers choose semantic content for those regions; this type
/// owns only their responsive spatial arrangement.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct UiFrameLayout {
    pub viewport: UiRect,
    pub content: UiRect,
    pub header: UiRect,
    pub body: UiRect,
    pub footer: UiRect,
    /// Capacity result for the frame regions after padding, heights, and gaps
    /// have been resolved.
    pub fit: UiLayoutFit,
}

impl UiFrameLayout {
    pub fn for_window(
        window_size: [f32; 2],
        padding: crate::UiInsets,
        header_height: f32,
        footer_height: f32,
        gap: f32,
    ) -> Self {
        let height = window_size[1].max(1.0);
        let viewport = UiRect::new([0.0, 0.0], [2.0 * window_size[0].max(1.0) / height, 2.0]);
        Self::new(viewport, padding, header_height, footer_height, gap)
    }

    pub fn new(
        viewport: UiRect,
        padding: crate::UiInsets,
        header_height: f32,
        footer_height: f32,
        gap: f32,
    ) -> Self {
        let content = viewport.inset_by(padding);
        let requested_header_height = header_height.max(0.0);
        let requested_footer_height = footer_height.max(0.0);
        let requested_gap = gap.max(0.0);
        let header_height = requested_header_height.clamp(0.0, content.size[1]);
        let footer_height =
            requested_footer_height.clamp(0.0, (content.size[1] - header_height).max(0.0));
        let gap = requested_gap
            .max(0.0)
            .min((content.size[1] - header_height - footer_height).max(0.0) * 0.5);
        let body_height = (content.size[1] - header_height - footer_height - gap * 2.0).max(0.0);
        let top = content.center[1] + content.size[1] * 0.5;
        let bottom = content.center[1] - content.size[1] * 0.5;
        let header = UiRect::new(
            [content.center[0], top - header_height * 0.5],
            [content.size[0], header_height],
        );
        let body_top = top - header_height - gap;
        let body = UiRect::new(
            [content.center[0], body_top - body_height * 0.5],
            [content.size[0], body_height],
        );
        let footer = UiRect::new(
            [content.center[0], bottom + footer_height * 0.5],
            [content.size[0], footer_height],
        );

        Self {
            viewport,
            content,
            header,
            body,
            footer,
            fit: if !usable_rect(viewport)
                || !usable_rect(content)
                || !usable_rect(header)
                || !usable_rect(body)
                || !usable_rect(footer)
            {
                // A frame has three required semantic regions. Once compaction
                // removes any one of them, callers need an explicit fallback;
                // reporting Adjusted would make unusable detail look valid.
                UiLayoutFit::Impossible
            } else if header_height != requested_header_height
                || footer_height != requested_footer_height
                || gap != requested_gap
            {
                UiLayoutFit::Adjusted
            } else {
                UiLayoutFit::Exact
            },
        }
    }
}

/// The resolved result of splitting one region into two readable panes.
///
/// `fits_minimums` is intentionally explicit: consumers can choose a compact
/// view, a scroll region, or a diagnostic instead of silently shrinking text
/// below its requested readable width.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct UiHorizontalSplitLayout {
    pub container: UiRect,
    pub leading: UiRect,
    pub trailing: UiRect,
    /// The resolved capacity result. `fits_minimums` remains for source
    /// compatibility with existing consumers.
    pub fit: UiLayoutFit,
    pub fits_minimums: bool,
}

impl UiHorizontalSplitLayout {
    pub fn new(
        container: UiRect,
        leading_fraction: f32,
        gap: f32,
        leading_min_width: f32,
        trailing_min_width: f32,
    ) -> Self {
        let gap = gap.max(0.0).min(container.size[0]);
        let available_width = (container.size[0] - gap).max(0.0);
        let leading_min_width = leading_min_width.max(0.0);
        let trailing_min_width = trailing_min_width.max(0.0);
        let fits_minimums = leading_min_width + trailing_min_width <= available_width;
        let preferred_leading = available_width * leading_fraction.clamp(0.0, 1.0);
        let leading_width = if fits_minimums {
            preferred_leading.clamp(
                leading_min_width,
                (available_width - trailing_min_width).max(leading_min_width),
            )
        } else {
            preferred_leading
        };
        let trailing_width = (available_width - leading_width).max(0.0);
        let left = container.center[0] - container.size[0] * 0.5;
        let leading = UiRect::new(
            [left + leading_width * 0.5, container.center[1]],
            [leading_width, container.size[1]],
        );
        let trailing = UiRect::new(
            [
                left + leading_width + gap + trailing_width * 0.5,
                container.center[1],
            ],
            [trailing_width, container.size[1]],
        );

        Self {
            container,
            leading,
            trailing,
            fit: if !usable_rect(container) || !usable_rect(leading) || !usable_rect(trailing) {
                UiLayoutFit::Impossible
            } else if fits_minimums {
                UiLayoutFit::Exact
            } else {
                UiLayoutFit::Adjusted
            },
            fits_minimums,
        }
    }
}

/// A row-major grid of equal-sized cells contained within one region.
///
/// This intentionally models only the recurring uniform-grid case. Content
/// measurement, spanning, and implicit column selection remain outside this
/// contract until independent consumers prove they are needed.
#[derive(Clone, Debug, PartialEq)]
pub struct UiUniformGridLayout {
    pub container: UiRect,
    pub columns: usize,
    pub rows: usize,
    pub gap: [f32; 2],
    pub cells: Vec<UiRect>,
    pub fit: UiLayoutFit,
}

impl UiUniformGridLayout {
    pub fn new(container: UiRect, item_count: usize, columns: usize, gap: [f32; 2]) -> Self {
        if !usable_rect(container) || columns == 0 {
            return Self {
                container,
                columns,
                rows: 0,
                gap: [0.0, 0.0],
                cells: Vec::new(),
                fit: UiLayoutFit::Impossible,
            };
        }
        if item_count == 0 {
            return Self {
                container,
                columns,
                rows: 0,
                gap: [0.0, 0.0],
                cells: Vec::new(),
                fit: UiLayoutFit::Exact,
            };
        }

        let rows = item_count.div_ceil(columns);
        let requested_gap = gap;
        let sanitized_gap = gap.map(|value| {
            if value.is_finite() {
                value.max(0.0)
            } else {
                0.0
            }
        });
        let column_gaps = columns.saturating_sub(1);
        let row_gaps = rows.saturating_sub(1);
        // Retain at least half of each axis for cells when a requested gap is
        // larger than the container can honestly accommodate.
        let max_horizontal_gap = if column_gaps == 0 {
            0.0
        } else {
            container.size[0] / (column_gaps as f32 * 2.0)
        };
        let max_vertical_gap = if row_gaps == 0 {
            0.0
        } else {
            container.size[1] / (row_gaps as f32 * 2.0)
        };
        let gap = [
            sanitized_gap[0].min(max_horizontal_gap),
            sanitized_gap[1].min(max_vertical_gap),
        ];
        let cell_size = [
            (container.size[0] - gap[0] * column_gaps as f32) / columns as f32,
            (container.size[1] - gap[1] * row_gaps as f32) / rows as f32,
        ];
        if !cell_size
            .iter()
            .all(|value| value.is_finite() && *value > 0.0)
        {
            return Self {
                container,
                columns,
                rows,
                gap,
                cells: Vec::new(),
                fit: UiLayoutFit::Impossible,
            };
        }

        let left = container.center[0] - container.size[0] * 0.5;
        let top = container.center[1] + container.size[1] * 0.5;
        let cells = (0..item_count)
            .map(|index| {
                let column = index % columns;
                let row = index / columns;
                UiRect::new(
                    [
                        left + column as f32 * (cell_size[0] + gap[0]) + cell_size[0] * 0.5,
                        top - row as f32 * (cell_size[1] + gap[1]) - cell_size[1] * 0.5,
                    ],
                    cell_size,
                )
            })
            .collect();
        let fit = if requested_gap != gap {
            UiLayoutFit::Adjusted
        } else {
            UiLayoutFit::Exact
        };

        Self {
            container,
            columns,
            rows,
            gap,
            cells,
            fit,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct UiConstraints {
    pub min: [f32; 2],
    pub max: [f32; 2],
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum UiSizePolicy {
    Intrinsic,
    Fill,
    Fixed([f32; 2]),
    Min([f32; 2]),
    Max([f32; 2]),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum UiCrossAxisAlignment {
    Start,
    Center,
    End,
    Fill,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum UiMainAxisAllocation {
    Intrinsic,
    Fill,
    /// Keeps child measurements intact and allocates remaining capacity
    /// between siblings. This is the semantic spacer behavior used by
    /// toolbars and command rows.
    SpaceBetween,
}

impl UiSizePolicy {
    pub fn resolve(self, intrinsic: [f32; 2], constraints: UiConstraints) -> [f32; 2] {
        let desired = match self {
            Self::Intrinsic => intrinsic,
            Self::Fill => constraints.max,
            Self::Fixed(size) => size,
            Self::Min(minimum) => [intrinsic[0].max(minimum[0]), intrinsic[1].max(minimum[1])],
            Self::Max(maximum) => [intrinsic[0].min(maximum[0]), intrinsic[1].min(maximum[1])],
        };
        constraints.constrain(desired)
    }
}

impl UiConstraints {
    pub const fn new(min: [f32; 2], max: [f32; 2]) -> Self {
        Self {
            min: [min[0].min(max[0]), min[1].min(max[1])],
            max: [min[0].max(max[0]), min[1].max(max[1])],
        }
    }

    pub const fn unbounded() -> Self {
        Self {
            min: [0.0, 0.0],
            max: [f32::INFINITY, f32::INFINITY],
        }
    }

    pub fn constrain(self, size: [f32; 2]) -> [f32; 2] {
        [
            size[0].clamp(self.min[0], self.max[0]),
            size[1].clamp(self.min[1], self.max[1]),
        ]
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct UiMeasureContext<'a> {
    pub theme: &'a UiTheme,
    pub available_space: [f32; 2],
    pub dpi_scale: f32,
    pub constraints: UiConstraints,
}

impl<'a> UiMeasureContext<'a> {
    pub fn new(theme: &'a UiTheme, available_space: [f32; 2]) -> Self {
        Self {
            theme,
            available_space,
            dpi_scale: 1.0,
            constraints: UiConstraints::new([0.0, 0.0], available_space),
        }
    }

    pub fn unbounded(theme: &'a UiTheme) -> Self {
        Self {
            theme,
            available_space: [f32::INFINITY, f32::INFINITY],
            dpi_scale: 1.0,
            constraints: UiConstraints::unbounded(),
        }
    }

    pub fn with_constraints(mut self, constraints: UiConstraints) -> Self {
        self.constraints = constraints;
        self
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct UiLayoutResult {
    pub rect: UiRect,
    /// Whether this node preserved its requested main-axis geometry.
    pub fit: UiLayoutFit,
    /// The positive main/cross-axis extent that remains outside `rect` when
    /// `fit` is `Overflow`.
    pub overflow: [f32; 2],
    pub children: Vec<UiLayoutResult>,
}

/// A provider-neutral view of resolved layout geometry.
///
/// Specialized layout helpers retain their domain-specific metadata while
/// exposing this common result to diagnostics, hit testing, and consumers that
/// only need resolved rectangles and fit evidence.
pub trait UiResolvedLayout {
    fn layout_result(&self) -> UiLayoutResult;
}

pub trait UiMeasurable {
    fn measure(&self, context: &UiMeasureContext<'_>) -> [f32; 2];
}

impl UiLayoutResult {
    pub fn new(rect: UiRect) -> Self {
        Self {
            rect,
            fit: UiLayoutFit::Exact,
            overflow: [0.0, 0.0],
            children: Vec::new(),
        }
    }

    pub fn with_children(rect: UiRect, children: Vec<UiLayoutResult>) -> Self {
        Self {
            rect,
            fit: UiLayoutFit::Exact,
            overflow: [0.0, 0.0],
            children,
        }
    }

    pub fn with_fit(
        rect: UiRect,
        fit: UiLayoutFit,
        overflow: [f32; 2],
        children: Vec<UiLayoutResult>,
    ) -> Self {
        Self {
            rect,
            fit,
            overflow,
            children,
        }
    }
}

impl UiResolvedLayout for UiLayoutResult {
    fn layout_result(&self) -> UiLayoutResult {
        self.clone()
    }
}

impl UiResolvedLayout for UiFrameLayout {
    fn layout_result(&self) -> UiLayoutResult {
        UiLayoutResult::with_fit(
            self.viewport,
            self.fit,
            [0.0, 0.0],
            vec![
                UiLayoutResult::new(self.header),
                UiLayoutResult::new(self.body),
                UiLayoutResult::new(self.footer),
            ],
        )
    }
}

impl UiResolvedLayout for UiHorizontalSplitLayout {
    fn layout_result(&self) -> UiLayoutResult {
        UiLayoutResult::with_fit(
            self.container,
            self.fit,
            [0.0, 0.0],
            vec![
                UiLayoutResult::new(self.leading),
                UiLayoutResult::new(self.trailing),
            ],
        )
    }
}

impl UiResolvedLayout for UiUniformGridLayout {
    fn layout_result(&self) -> UiLayoutResult {
        UiLayoutResult::with_fit(
            self.container,
            self.fit,
            [0.0, 0.0],
            self.cells
                .iter()
                .copied()
                .map(UiLayoutResult::new)
                .collect(),
        )
    }
}

fn usable_rect(rect: UiRect) -> bool {
    rect.center.iter().all(|value| value.is_finite())
        && rect
            .size
            .iter()
            .all(|value| value.is_finite() && *value > 0.0)
}

#[derive(Clone, Debug, PartialEq)]
pub struct UiHorizontalStack<T> {
    pub children: Vec<T>,
    pub gap: f32,
    pub cross_axis_alignment: UiCrossAxisAlignment,
    pub main_axis_allocation: UiMainAxisAllocation,
    pub overflow_policy: UiOverflowPolicy,
}

impl<T: UiMeasurable> UiHorizontalStack<T> {
    pub fn new(children: Vec<T>, gap: f32) -> Self {
        Self {
            children,
            gap: gap.max(0.0),
            cross_axis_alignment: UiCrossAxisAlignment::Center,
            main_axis_allocation: UiMainAxisAllocation::Intrinsic,
            overflow_policy: UiOverflowPolicy::default(),
        }
    }

    pub fn with_cross_axis_alignment(mut self, alignment: UiCrossAxisAlignment) -> Self {
        self.cross_axis_alignment = alignment;
        self
    }

    pub fn with_main_axis_allocation(mut self, allocation: UiMainAxisAllocation) -> Self {
        self.main_axis_allocation = allocation;
        self
    }

    pub fn with_overflow_policy(mut self, policy: UiOverflowPolicy) -> Self {
        self.overflow_policy = policy;
        self
    }

    pub fn measure(&self, context: &UiMeasureContext<'_>) -> [f32; 2] {
        let child_constraints = UiConstraints::new(
            [0.0, 0.0],
            [context.available_space[0], context.available_space[1]],
        );
        let child_context = context.with_constraints(child_constraints);
        let child_sizes = self
            .children
            .iter()
            .map(|child| child.measure(&child_context));
        let mut size = [0.0_f32, 0.0_f32];
        for (index, child_size) in child_sizes.enumerate() {
            size[0] += child_size[0];
            size[1] = size[1].max(child_size[1]);
            if index > 0 {
                size[0] += self.gap;
            }
        }
        context.constraints.constrain(size)
    }

    pub fn layout(&self, rect: UiRect, context: &UiMeasureContext<'_>) -> UiLayoutResult {
        if !usable_rect(rect) {
            return UiLayoutResult::with_fit(rect, UiLayoutFit::Impossible, [0.0, 0.0], Vec::new());
        }
        let child_constraints = UiConstraints::new([0.0, 0.0], [rect.size[0], rect.size[1]]);
        let child_context = context.with_constraints(child_constraints);
        let mut child_sizes: Vec<[f32; 2]> = self
            .children
            .iter()
            .map(|child| child.measure(&child_context))
            .collect();
        let mut effective_gap = if child_sizes.len() > 1 {
            self.gap.min(rect.size[0] / (child_sizes.len() - 1) as f32)
        } else {
            0.0
        };
        let total_width: f32 = child_sizes.iter().map(|size| size[0]).sum::<f32>()
            + effective_gap * child_sizes.len().saturating_sub(1) as f32;
        let mut fit = UiLayoutFit::Exact;
        let mut overflow = [0.0, 0.0];
        if total_width < rect.size[0] && !child_sizes.is_empty() {
            match self.main_axis_allocation {
                UiMainAxisAllocation::Fill => {
                    let extra_width = (rect.size[0] - total_width) / child_sizes.len() as f32;
                    for size in &mut child_sizes {
                        size[0] += extra_width;
                    }
                    fit = UiLayoutFit::Adjusted;
                }
                UiMainAxisAllocation::SpaceBetween if child_sizes.len() > 1 => {
                    effective_gap += (rect.size[0] - total_width) / (child_sizes.len() - 1) as f32;
                    fit = UiLayoutFit::Adjusted;
                }
                UiMainAxisAllocation::Intrinsic | UiMainAxisAllocation::SpaceBetween => {}
            }
        } else if total_width > rect.size[0] && total_width > 0.0 {
            match self.overflow_policy {
                UiOverflowPolicy::Compress => {
                    let scale = (rect.size[0]
                        - effective_gap * child_sizes.len().saturating_sub(1) as f32)
                        .max(0.0)
                        / child_sizes.iter().map(|size| size[0]).sum::<f32>().max(1.0);
                    for size in &mut child_sizes {
                        size[0] *= scale;
                    }
                    fit = UiLayoutFit::Adjusted;
                }
                UiOverflowPolicy::Preserve => {
                    fit = UiLayoutFit::Overflow;
                    overflow[0] = total_width - rect.size[0];
                }
            }
        }

        let content_width = child_sizes.iter().map(|size| size[0]).sum::<f32>()
            + effective_gap * child_sizes.len().saturating_sub(1) as f32;
        let mut cursor = rect.center[0] - content_width * 0.5;
        let children = child_sizes
            .into_iter()
            .map(|size| {
                let child_height = match self.cross_axis_alignment {
                    UiCrossAxisAlignment::Fill => rect.size[1],
                    _ => size[1].min(rect.size[1]),
                };
                let child_center_y = match self.cross_axis_alignment {
                    UiCrossAxisAlignment::Start => {
                        rect.center[1] + (rect.size[1] - child_height) * 0.5
                    }
                    UiCrossAxisAlignment::Center | UiCrossAxisAlignment::Fill => rect.center[1],
                    UiCrossAxisAlignment::End => {
                        rect.center[1] - (rect.size[1] - child_height) * 0.5
                    }
                };
                let child_rect = UiRect::new(
                    [cursor + size[0] * 0.5, child_center_y],
                    [size[0], child_height],
                );
                cursor += size[0] + effective_gap;
                UiLayoutResult::new(child_rect)
            })
            .collect();
        UiLayoutResult::with_fit(rect, fit, overflow, children)
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct UiVerticalStack<T> {
    pub children: Vec<T>,
    pub gap: f32,
    pub cross_axis_alignment: UiCrossAxisAlignment,
    pub main_axis_allocation: UiMainAxisAllocation,
    pub overflow_policy: UiOverflowPolicy,
}

impl<T: UiMeasurable> UiVerticalStack<T> {
    pub fn new(children: Vec<T>, gap: f32) -> Self {
        Self {
            children,
            gap: gap.max(0.0),
            cross_axis_alignment: UiCrossAxisAlignment::Center,
            main_axis_allocation: UiMainAxisAllocation::Intrinsic,
            overflow_policy: UiOverflowPolicy::default(),
        }
    }

    pub fn with_cross_axis_alignment(mut self, alignment: UiCrossAxisAlignment) -> Self {
        self.cross_axis_alignment = alignment;
        self
    }

    pub fn with_main_axis_allocation(mut self, allocation: UiMainAxisAllocation) -> Self {
        self.main_axis_allocation = allocation;
        self
    }

    pub fn with_overflow_policy(mut self, policy: UiOverflowPolicy) -> Self {
        self.overflow_policy = policy;
        self
    }

    pub fn measure(&self, context: &UiMeasureContext<'_>) -> [f32; 2] {
        let child_constraints = UiConstraints::new(
            [0.0, 0.0],
            [context.available_space[0], context.available_space[1]],
        );
        let child_context = context.with_constraints(child_constraints);
        let child_sizes = self
            .children
            .iter()
            .map(|child| child.measure(&child_context));
        let mut size = [0.0_f32, 0.0_f32];
        for (index, child_size) in child_sizes.enumerate() {
            size[0] = size[0].max(child_size[0]);
            size[1] += child_size[1];
            if index > 0 {
                size[1] += self.gap;
            }
        }
        context.constraints.constrain(size)
    }

    pub fn layout(&self, rect: UiRect, context: &UiMeasureContext<'_>) -> UiLayoutResult {
        if !usable_rect(rect) {
            return UiLayoutResult::with_fit(rect, UiLayoutFit::Impossible, [0.0, 0.0], Vec::new());
        }
        let child_constraints = UiConstraints::new([0.0, 0.0], [rect.size[0], rect.size[1]]);
        let child_context = context.with_constraints(child_constraints);
        let mut child_sizes: Vec<[f32; 2]> = self
            .children
            .iter()
            .map(|child| child.measure(&child_context))
            .collect();
        let mut effective_gap = if child_sizes.len() > 1 {
            self.gap.min(rect.size[1] / (child_sizes.len() - 1) as f32)
        } else {
            0.0
        };
        let total_height: f32 = child_sizes.iter().map(|size| size[1]).sum::<f32>()
            + effective_gap * child_sizes.len().saturating_sub(1) as f32;
        let mut fit = UiLayoutFit::Exact;
        let mut overflow = [0.0, 0.0];
        if total_height < rect.size[1] && !child_sizes.is_empty() {
            match self.main_axis_allocation {
                UiMainAxisAllocation::Fill => {
                    let extra_height = (rect.size[1] - total_height) / child_sizes.len() as f32;
                    for size in &mut child_sizes {
                        size[1] += extra_height;
                    }
                    fit = UiLayoutFit::Adjusted;
                }
                UiMainAxisAllocation::SpaceBetween if child_sizes.len() > 1 => {
                    effective_gap += (rect.size[1] - total_height) / (child_sizes.len() - 1) as f32;
                    fit = UiLayoutFit::Adjusted;
                }
                UiMainAxisAllocation::Intrinsic | UiMainAxisAllocation::SpaceBetween => {}
            }
        } else if total_height > rect.size[1] && total_height > 0.0 {
            match self.overflow_policy {
                UiOverflowPolicy::Compress => {
                    let scale = (rect.size[1]
                        - effective_gap * child_sizes.len().saturating_sub(1) as f32)
                        .max(0.0)
                        / child_sizes.iter().map(|size| size[1]).sum::<f32>().max(1.0);
                    for size in &mut child_sizes {
                        size[1] *= scale;
                    }
                    fit = UiLayoutFit::Adjusted;
                }
                UiOverflowPolicy::Preserve => {
                    fit = UiLayoutFit::Overflow;
                    overflow[1] = total_height - rect.size[1];
                }
            }
        }

        let content_height = child_sizes.iter().map(|size| size[1]).sum::<f32>()
            + effective_gap * child_sizes.len().saturating_sub(1) as f32;
        let mut cursor = rect.center[1] + content_height * 0.5;
        let children = child_sizes
            .into_iter()
            .map(|size| {
                let child_width = match self.cross_axis_alignment {
                    UiCrossAxisAlignment::Fill => rect.size[0],
                    _ => size[0].min(rect.size[0]),
                };
                let child_center_x = match self.cross_axis_alignment {
                    UiCrossAxisAlignment::Start => {
                        rect.center[0] - (rect.size[0] - child_width) * 0.5
                    }
                    UiCrossAxisAlignment::Center | UiCrossAxisAlignment::Fill => rect.center[0],
                    UiCrossAxisAlignment::End => {
                        rect.center[0] + (rect.size[0] - child_width) * 0.5
                    }
                };
                let child_rect = UiRect::new(
                    [child_center_x, cursor - size[1] * 0.5],
                    [child_width, size[1]],
                );
                cursor -= size[1] + effective_gap;
                UiLayoutResult::new(child_rect)
            })
            .collect();
        UiLayoutResult::with_fit(rect, fit, overflow, children)
    }
}
