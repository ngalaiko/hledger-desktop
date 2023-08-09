use std::fmt;

use tauri_egui::egui::{
    epaint::RectShape, pos2, vec2, Response, Sense, Shape, Ui, WidgetInfo, WidgetType,
};

#[derive(Debug, Clone, Copy, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum CheckboxState {
    Checked,
    Unchecked,
    Indeterminate,
}

pub fn ui(ui: &mut Ui, state: &mut CheckboxState) -> Response {
    let desired_size = ui.spacing().interact_size.y * vec2(1.0, 1.0);

    let (rect, mut response) = ui.allocate_exact_size(desired_size, Sense::click());

    if response.clicked() {
        *state = match *state {
            CheckboxState::Checked => CheckboxState::Unchecked,
            CheckboxState::Unchecked => CheckboxState::Checked,
            CheckboxState::Indeterminate => CheckboxState::Checked,
        };
        response.mark_changed();
    }

    response.widget_info(|| WidgetInfo::labeled(WidgetType::Checkbox, state.to_string()));

    if ui.is_rect_visible(rect) {
        let visuals = ui.style().interact(&response);
        let (small_icon_rect, big_icon_rect) = ui.spacing().icon_rectangles(rect);
        ui.painter().add(RectShape {
            rect: big_icon_rect.expand(visuals.expansion),
            rounding: visuals.rounding,
            fill: visuals.bg_fill,
            stroke: visuals.bg_stroke,
        });

        match *state {
            CheckboxState::Checked => {
                // Check mark:
                ui.painter().add(Shape::line(
                    vec![
                        pos2(small_icon_rect.left(), small_icon_rect.center().y),
                        pos2(small_icon_rect.center().x, small_icon_rect.bottom()),
                        pos2(small_icon_rect.right(), small_icon_rect.top()),
                    ],
                    visuals.fg_stroke,
                ));
            }
            CheckboxState::Indeterminate => {
                // Minus:
                ui.painter().add(Shape::line(
                    vec![
                        pos2(small_icon_rect.left(), small_icon_rect.center().y),
                        pos2(small_icon_rect.right(), small_icon_rect.center().y),
                    ],
                    visuals.fg_stroke,
                ));
            }
            CheckboxState::Unchecked => {}
        }
    }

    response
}

impl fmt::Display for CheckboxState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CheckboxState::Checked => write!(f, "Checked"),
            CheckboxState::Unchecked => write!(f, "Unchecked"),
            CheckboxState::Indeterminate => write!(f, "Indeterminate"),
        }
    }
}
