use egui::{vec2, Widget};

#[derive(Clone, Debug)]
pub struct ControlPoint {
    pub softness:   Option<f32>,
    pub tangent:    Option<egui::Vec2>,
    pub position:   egui::Vec2,
    pub guard_rail: bool,
    pub default:    Option<Box<ControlPoint>>,
}
impl ControlPoint {
    pub fn new(position: egui::Vec2) -> Self {
        let mut point = Self {
            softness: None,
            tangent: None,
            position,
            guard_rail: false,
            default: None,
        };
        point.default = Some(Box::new(point.clone()));
        point
    }

    pub fn reset(&mut self) {
        if let Some(default) = &self.default {
            self.softness = default.softness;
            self.tangent = default.tangent;
            self.position = default.position;
            self.guard_rail = default.guard_rail;
        }
    }
}

pub struct Envelope<'a> {
    pub control_points: &'a mut Vec<ControlPoint>,
}
impl<'a> Envelope<'a> {
    pub fn new(control_points: &'a mut Vec<ControlPoint>) -> Self { Self { control_points } }
}
impl<'a> Widget for Envelope<'a> {
    fn ui(self, ui: &mut egui::Ui) -> egui::Response {
        let (rect, response) = ui.allocate_at_least(
            egui::vec2(ui.available_width(), 200.0),
            egui::Sense::click_and_drag(),
        );

        let to_normalized = |p: egui::Pos2| (p.to_vec2() - rect.min.to_vec2()) / rect.size();
        let from_normalized = |p: egui::Vec2| (p * rect.size() + rect.min.to_vec2()).to_pos2();

        let painter = ui.painter();
        let visuals = ui.style().interact(&response);
        painter.rect(rect, visuals.rounding, visuals.bg_fill, visuals.bg_stroke);

        for point in self.control_points.windows(2) {
            painter.line_segment(
                [
                    from_normalized(point[0].position),
                    from_normalized(point[1].position),
                ],
                visuals.fg_stroke,
            );
        }

        if response.hovered() || response.dragged() || response.double_clicked() {
            let cursor_position = response
                .interact_pointer_pos()
                .or(response.hover_pos())
                .map(to_normalized);

            let closest_point: Option<usize> = cursor_position.and_then(|cursor_position| {
                self.control_points
                    .iter()
                    .enumerate()
                    .min_by_key(|(_, point)| {
                        let distance = (from_normalized(point.position)
                            - from_normalized(cursor_position))
                        .length_sq();
                        (distance * 10000.0).clamp(0.0, usize::MAX as f32) as usize
                    })
                    .map(|(i, _)| i)
            });

            if response.double_clicked() {
                if let Some(closest) = closest_point {
                    self.control_points[closest].reset();
                }
            }

            for (i, point) in self.control_points.iter_mut().enumerate() {
                let is_closest = closest_point == Some(i);

                if is_closest {
                    if response.dragged() {
                        point.position = (point.position + response.drag_delta() / rect.size())
                            .clamp(vec2(0.0, 0.0), vec2(1.0, 1.0));
                    }

                    painter.circle(
                        from_normalized(point.position),
                        10.0,
                        visuals.bg_fill,
                        (1.0, ui.style().visuals.error_fg_color),
                    );
                }
                else {
                    painter.circle(
                        from_normalized(point.position),
                        3.0,
                        visuals.bg_fill,
                        visuals.bg_stroke,
                    );
                }
            }
        }

        response
    }
}
