use crate::prelude::*;

static THUMB: &'static str = "thumb";
static TRACK: &'static str = "track";

#[derive(Copy, Clone)]
enum SliderAction {
    Move { mouse_x: f64 },
}

/// The `SliderState` is used to manipulate the position of the thumb of the slider widget.
#[derive(Default, AsAny)]
pub struct SliderState {
    action: Option<SliderAction>,
    value: f64,
    thumb: Entity,
    track: Entity,
}

impl SliderState {
    fn action(&mut self, action: SliderAction) {
        self.action = Some(action);
    }

    fn adjust(&mut self, ctx: &mut Context) {}
}

impl State for SliderState {
    fn init(&mut self, _: &mut Registry, ctx: &mut Context) {
        self.thumb = ctx
            .entity_of_child(THUMB)
            .expect("SliderState.init: Thumb child could not be found.");
        self.track = ctx
            .entity_of_child(TRACK)
            .expect("SliderState.init: Track child could not be found.");
    }

    fn update_post_layout(&mut self, _: &mut Registry, ctx: &mut Context) {
        if let Some(action) = self.action {
            match action {
                SliderAction::Move { mouse_x } => {
                    if *ctx.child("thumb").get::<bool>("pressed") {
                        let thumb_width = ctx
                            .get_widget(self.thumb)
                            .get::<Rectangle>("bounds")
                            .width();
                        let track_width = ctx
                            .get_widget(self.track)
                            .get::<Rectangle>("bounds")
                            .width();
                        let slider_x = ctx.widget().get::<Point>("position").x;

                        let thumb_x =
                            calculate_thumb_x(mouse_x, thumb_width, slider_x, track_width);

                        ctx.get_widget(self.thumb)
                            .get_mut::<Thickness>("margin")
                            .set_left(thumb_x);

                        let minimum = *ctx.widget().get("minimum");
                        let maximum = *ctx.widget().get("maximum");

                        ctx.widget().set(
                            "value",
                            calculate_value(thumb_x, minimum, maximum, thumb_width, track_width),
                        );

                        ctx.push_event(ChangedEvent(ctx.entity));
                    }
                }
            }

            self.action = None;
        }
    }
}

widget!(
    /// The `Slider` allows to use a value in a range of values.
    ///
    /// **CSS element:** `Slider`
    Slider<SliderState>: MouseHandler, ChangedHandler {
        /// Sets or shares the minimum of the range.
        minimum: f64,

        /// Sets or shared the maximum of the range.
        maximum: f64,

        /// Sets or shares the current value of the range.
        value: f64,

        /// Sets or shares the background property.
        background: Brush,

        /// Sets or shares the border radius property.
        border_radius: f64,

        /// Sets or shares the border thickness property.
        border_width: Thickness,

        /// Sets or shares the border brush property.
        border_brush: Brush,

        /// Sets or shares the css selector property.
        selector: Selector
    }
);

impl Template for Slider {
    fn template(self, id: Entity, ctx: &mut BuildContext) -> Self {
        self.name("Slider")
            .selector("slider")
            .minimum(0.0)
            .maximum(100.0)
            .value(0.0)
            .height(32.0)
            .border_radius(4.0)
            .child(
                Grid::create()
                    .selector(Selector::default().id(TRACK))
                    .margin((8.0, 0.0, 8.0, 0.0))
                    .child(
                        Container::create()
                            // todo fix border radius from css
                            .border_radius(id)
                            .background(id)
                            .vertical_alignment("center")
                            .height(8.0)
                            .build(ctx),
                    )
                    .child(
                        // todo: selector default crashes
                        Button::create()
                            .selector(Selector::from("thumb").id(THUMB))
                            .vertical_alignment("center")
                            .horizontal_alignment("start")
                            .max_width(28.0)
                            .max_height(28.0)
                            .border_radius(16.0)
                            .build(ctx),
                    )
                    .build(ctx),
            )
            .on_mouse_move(move |states, p| {
                states
                    .get_mut::<SliderState>(id)
                    .action(SliderAction::Move { mouse_x: p.x });
                true
            })
    }
}

// --- Helpers --

fn adjust_value(value: f64, minimum: f64, maximum: f64) -> f64 {
    if value < minimum {
        return minimum;
    }

    if value > maximum {
        return maximum;
    }

    value
}

fn adjust_minimum(minimum: f64, maximum: f64) -> f64 {
    if minimum > maximum {
        return maximum;
    }

    minimum
}

fn adjust_maximum(minimum: f64, maximum: f64) -> f64 {
    if maximum < minimum {
        return minimum;
    }

    maximum
}

fn calculate_thumb_x(mouse_x: f64, thumb_width: f64, slider_x: f64, track_width: f64) -> f64 {
    (mouse_x - slider_x - thumb_width)
        .max(0.0)
        .min(track_width - thumb_width)
        .round()
}

fn calculate_value(
    thumb_x: f64,
    minimum: f64,
    maximum: f64,
    thumb_width: f64,
    track_width: f64,
) -> f64 {
    (thumb_x / (track_width - thumb_width) * (maximum - minimum)).round()
}

// --- Helpers --

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calculate_thumb_x() {
        assert_eq!(0.0, calculate_thumb_x(-1000.0, 32.0, 0.0, 100.0));
        assert_eq!(0.0, calculate_thumb_x(0.0, 32.0, 0.0, 100.0));
        assert_eq!(18.0, calculate_thumb_x(50.0, 32.0, 0.0, 100.0));
        assert_eq!(36.0, calculate_thumb_x(68.0, 32.0, 0.0, 100.0));
        assert_eq!(68.0, calculate_thumb_x(100.0, 32.0, 0.0, 100.0));
        assert_eq!(68.0, calculate_thumb_x(1000.0, 32.0, 0.0, 100.0));
    }

    #[test]
    fn test_calculate_value() {
        assert_eq!(0.0, calculate_value(0.0, 0.0, 100.0, 32.0, 100.0));
        assert_eq!(50.0, calculate_value(34.0, 0.0, 100.0, 32.0, 100.0));
        assert_eq!(100.0, calculate_value(68.0, 0.0, 100.0, 32.0, 100.0));
        assert_eq!(0.0, calculate_value(0.0, -50.0, 50.0, 32.0, 100.0));
        assert_eq!(50.0, calculate_value(34.0, -50.0, 50.0, 32.0, 100.0));
        assert_eq!(100.0, calculate_value(68.0, -50.0, 50.0, 32.0, 100.0));
    }
}
