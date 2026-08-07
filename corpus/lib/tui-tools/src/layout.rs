use crate::{TuiDiagnostic, TuiRect};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Axis {
    Horizontal,
    Vertical,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LayoutConstraint {
    Fixed(u16),
    Min(u16),
    Remaining,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct LayoutResult {
    pub regions: Vec<TuiRect>,
    pub diagnostics: Vec<TuiDiagnostic>,
}

pub fn split(parent: TuiRect, axis: Axis, constraints: &[LayoutConstraint]) -> LayoutResult {
    if parent.is_empty() {
        return LayoutResult {
            regions: vec![TuiRect::new(parent.x, parent.y, 0, 0); constraints.len()],
            diagnostics: vec![TuiDiagnostic::EmptyRegion { region: parent }],
        };
    }

    let available = match axis {
        Axis::Horizontal => parent.width,
        Axis::Vertical => parent.height,
    };
    let required = constraints
        .iter()
        .map(|constraint| match constraint {
            LayoutConstraint::Fixed(value) | LayoutConstraint::Min(value) => *value,
            LayoutConstraint::Remaining => 0,
        })
        .fold(0_u16, u16::saturating_add);
    let remaining_count = constraints
        .iter()
        .filter(|constraint| matches!(constraint, LayoutConstraint::Remaining))
        .count() as u16;
    let flexible = available.saturating_sub(required);
    let flexible_each = flexible.checked_div(remaining_count).unwrap_or(0);
    let mut flexible_remainder = flexible.checked_rem(remaining_count).unwrap_or(0);

    let mut cursor = 0_u16;
    let mut regions = Vec::with_capacity(constraints.len());
    for constraint in constraints {
        let requested = match constraint {
            LayoutConstraint::Fixed(value) | LayoutConstraint::Min(value) => *value,
            LayoutConstraint::Remaining => {
                let extra = u16::from(flexible_remainder > 0);
                flexible_remainder = flexible_remainder.saturating_sub(extra);
                flexible_each.saturating_add(extra)
            }
        };
        let length = requested.min(available.saturating_sub(cursor));
        let region = match axis {
            Axis::Horizontal => TuiRect::new(
                parent.x.saturating_add(cursor),
                parent.y,
                length,
                parent.height,
            ),
            Axis::Vertical => TuiRect::new(
                parent.x,
                parent.y.saturating_add(cursor),
                parent.width,
                length,
            ),
        };
        regions.push(region);
        cursor = cursor.saturating_add(length);
    }

    let diagnostics = if required > available {
        vec![TuiDiagnostic::Undersized {
            axis: match axis {
                Axis::Horizontal => "horizontal",
                Axis::Vertical => "vertical",
            },
            available,
            required,
        }]
    } else {
        Vec::new()
    };

    LayoutResult {
        regions,
        diagnostics,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn split_is_deterministic_and_bounded() {
        let parent = TuiRect::new(4, 2, 20, 5);
        let result = split(
            parent,
            Axis::Horizontal,
            &[
                LayoutConstraint::Fixed(4),
                LayoutConstraint::Remaining,
                LayoutConstraint::Min(3),
            ],
        );
        assert_eq!(
            result.regions,
            vec![
                TuiRect::new(4, 2, 4, 5),
                TuiRect::new(8, 2, 13, 5),
                TuiRect::new(21, 2, 3, 5),
            ]
        );
        assert!(result.diagnostics.is_empty());
        assert!(result
            .regions
            .iter()
            .all(|region| region.right() <= parent.right()));
    }

    #[test]
    fn undersized_split_reports_and_clamps() {
        let parent = TuiRect::new(0, 0, 5, 2);
        let result = split(
            parent,
            Axis::Horizontal,
            &[LayoutConstraint::Fixed(4), LayoutConstraint::Min(4)],
        );
        assert_eq!(result.regions[0].width, 4);
        assert_eq!(result.regions[1].width, 1);
        assert_eq!(
            result.diagnostics,
            vec![TuiDiagnostic::Undersized {
                axis: "horizontal",
                available: 5,
                required: 8,
            }]
        );
    }
}
