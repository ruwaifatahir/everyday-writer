//! Where the popup opens: near the mouse, fully inside one monitor's work
//! area. Pure math in physical pixels, so it is tested without a screen.

/// A rectangle in physical desktop pixels. `x` and `y` can be negative for
/// monitors left of or above the primary one.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Rect {
    /// Left edge.
    pub x: i32,
    /// Top edge.
    pub y: i32,
    /// Width.
    pub width: i32,
    /// Height.
    pub height: i32,
}

/// How far the popup sits from the mouse pointer, in physical pixels at
/// 100 % scale.
pub const GAP: i32 = 12;

/// The top-left corner for a popup of `width` x `height` opened at
/// `cursor`, so it lies fully inside `work_area`.
///
/// The popup opens below and to the right of the pointer, flips to the
/// other side when it would cross the right or bottom edge, and is finally
/// clamped into the work area. A popup larger than the work area is pinned
/// to its top-left corner.
pub fn place_near_cursor(
    cursor: (i32, i32),
    (width, height): (i32, i32),
    gap: i32,
    work_area: Rect,
) -> (i32, i32) {
    let (cursor_x, cursor_y) = cursor;
    let right = work_area.x + work_area.width;
    let bottom = work_area.y + work_area.height;
    let mut x = cursor_x + gap;
    if x + width > right {
        x = cursor_x - gap - width;
    }
    let mut y = cursor_y + gap;
    if y + height > bottom {
        y = cursor_y - gap - height;
    }
    (
        x.clamp(work_area.x, (right - width).max(work_area.x)),
        y.clamp(work_area.y, (bottom - height).max(work_area.y)),
    )
}

#[cfg(test)]
mod tests {
    use super::{Rect, place_near_cursor};

    const SIZE: (i32, i32) = (400, 200);

    /// A 1920 x 1040 work area (a 1080p screen minus the taskbar).
    const PRIMARY: Rect = Rect {
        x: 0,
        y: 0,
        width: 1920,
        height: 1040,
    };

    /// A monitor to the left of the primary one, with negative coordinates.
    const LEFT: Rect = Rect {
        x: -1280,
        y: 0,
        width: 1280,
        height: 984,
    };

    #[test]
    fn opens_below_and_right_of_the_pointer() {
        assert_eq!(place_near_cursor((500, 300), SIZE, 12, PRIMARY), (512, 312));
    }

    #[test]
    fn flips_left_at_the_right_edge() {
        assert_eq!(
            place_near_cursor((1900, 300), SIZE, 12, PRIMARY),
            (1488, 312)
        );
    }

    #[test]
    fn flips_up_at_the_bottom_edge() {
        assert_eq!(
            place_near_cursor((500, 1030), SIZE, 12, PRIMARY),
            (512, 818)
        );
    }

    #[test]
    fn stays_inside_a_monitor_with_negative_coordinates() {
        assert_eq!(place_near_cursor((-10, 970), SIZE, 12, LEFT), (-422, 758));
    }

    #[test]
    fn never_crosses_into_the_next_monitor() {
        let (x, _) = place_near_cursor((-1, 300), SIZE, 12, LEFT);

        assert!(x + SIZE.0 <= LEFT.x + LEFT.width);
    }

    #[test]
    fn is_clamped_when_flipping_would_leave_the_work_area() {
        let narrow = Rect {
            x: 0,
            y: 0,
            width: 500,
            height: 300,
        };

        assert_eq!(place_near_cursor((250, 150), SIZE, 12, narrow), (0, 0));
    }

    #[test]
    fn a_popup_larger_than_the_work_area_is_pinned_to_its_corner() {
        let tiny = Rect {
            x: 100,
            y: 50,
            width: 300,
            height: 100,
        };

        assert_eq!(place_near_cursor((200, 80), SIZE, 12, tiny), (100, 50));
    }
}
