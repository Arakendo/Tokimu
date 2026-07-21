use crate::{UiRect, UiTheme};

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
    pub children: Vec<UiLayoutResult>,
}

pub trait UiMeasurable {
    fn measure(&self, context: &UiMeasureContext<'_>) -> [f32; 2];
}

impl UiLayoutResult {
    pub fn new(rect: UiRect) -> Self {
        Self {
            rect,
            children: Vec::new(),
        }
    }

    pub fn with_children(rect: UiRect, children: Vec<UiLayoutResult>) -> Self {
        Self { rect, children }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct UiHorizontalStack<T> {
    pub children: Vec<T>,
    pub gap: f32,
    pub cross_axis_alignment: UiCrossAxisAlignment,
    pub main_axis_allocation: UiMainAxisAllocation,
}

impl<T: UiMeasurable> UiHorizontalStack<T> {
    pub fn new(children: Vec<T>, gap: f32) -> Self {
        Self {
            children,
            gap: gap.max(0.0),
            cross_axis_alignment: UiCrossAxisAlignment::Center,
            main_axis_allocation: UiMainAxisAllocation::Intrinsic,
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
        let child_constraints = UiConstraints::new([0.0, 0.0], [rect.size[0], rect.size[1]]);
        let child_context = context.with_constraints(child_constraints);
        let mut child_sizes: Vec<[f32; 2]> = self
            .children
            .iter()
            .map(|child| child.measure(&child_context))
            .collect();
        let effective_gap = if child_sizes.len() > 1 {
            self.gap.min(rect.size[0] / (child_sizes.len() - 1) as f32)
        } else {
            0.0
        };
        let total_width: f32 = child_sizes.iter().map(|size| size[0]).sum::<f32>()
            + effective_gap * child_sizes.len().saturating_sub(1) as f32;
        if total_width < rect.size[0]
            && self.main_axis_allocation == UiMainAxisAllocation::Fill
            && !child_sizes.is_empty()
        {
            let extra_width = (rect.size[0] - total_width) / child_sizes.len() as f32;
            for size in &mut child_sizes {
                size[0] += extra_width;
            }
        } else if total_width > rect.size[0] && total_width > 0.0 {
            let scale = (rect.size[0] - effective_gap * child_sizes.len().saturating_sub(1) as f32)
                .max(0.0)
                / child_sizes.iter().map(|size| size[0]).sum::<f32>().max(1.0);
            for size in &mut child_sizes {
                size[0] *= scale;
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
        UiLayoutResult::with_children(rect, children)
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct UiVerticalStack<T> {
    pub children: Vec<T>,
    pub gap: f32,
    pub cross_axis_alignment: UiCrossAxisAlignment,
    pub main_axis_allocation: UiMainAxisAllocation,
}

impl<T: UiMeasurable> UiVerticalStack<T> {
    pub fn new(children: Vec<T>, gap: f32) -> Self {
        Self {
            children,
            gap: gap.max(0.0),
            cross_axis_alignment: UiCrossAxisAlignment::Center,
            main_axis_allocation: UiMainAxisAllocation::Intrinsic,
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
        let child_constraints = UiConstraints::new([0.0, 0.0], [rect.size[0], rect.size[1]]);
        let child_context = context.with_constraints(child_constraints);
        let mut child_sizes: Vec<[f32; 2]> = self
            .children
            .iter()
            .map(|child| child.measure(&child_context))
            .collect();
        let effective_gap = if child_sizes.len() > 1 {
            self.gap.min(rect.size[1] / (child_sizes.len() - 1) as f32)
        } else {
            0.0
        };
        let total_height: f32 = child_sizes.iter().map(|size| size[1]).sum::<f32>()
            + effective_gap * child_sizes.len().saturating_sub(1) as f32;
        if total_height < rect.size[1]
            && self.main_axis_allocation == UiMainAxisAllocation::Fill
            && !child_sizes.is_empty()
        {
            let extra_height = (rect.size[1] - total_height) / child_sizes.len() as f32;
            for size in &mut child_sizes {
                size[1] += extra_height;
            }
        } else if total_height > rect.size[1] && total_height > 0.0 {
            let scale = (rect.size[1] - effective_gap * child_sizes.len().saturating_sub(1) as f32)
                .max(0.0)
                / child_sizes.iter().map(|size| size[1]).sum::<f32>().max(1.0);
            for size in &mut child_sizes {
                size[1] *= scale;
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
        UiLayoutResult::with_children(rect, children)
    }
}
