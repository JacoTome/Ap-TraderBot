use crate::{
    ACTIVE_COLOR, BG_COLOR, DETAILS_COLOR, HOVERED_COLOR, INACTIVE_COLOR, PLOT_BG_COLOR, TEXT_COLOR,
};
use egui::style::{Selection, Spacing};
use egui::{style::WidgetVisuals, Stroke};
use egui::{Rounding, Vec2, Visuals};
pub fn get_widget_visual_settings() -> egui::style::Widgets {
    let non_interactive_style = WidgetVisuals {
        fg_stroke: Stroke::new(2.0, DETAILS_COLOR),
        bg_stroke: Stroke::new(2.0, DETAILS_COLOR),
        bg_fill: BG_COLOR,
        rounding: Rounding::none(),
        expansion: 10.0,
    };

    let inactive = WidgetVisuals {
        fg_stroke: Stroke::new(1.0, TEXT_COLOR),
        bg_stroke: Stroke::new(1.0, INACTIVE_COLOR),
        bg_fill: INACTIVE_COLOR,
        rounding: Rounding {
            nw: 5.0,
            ne: 5.0,
            se: 5.0,
            sw: 5.0,
        },
        expansion: 1.0,
    };

    let hovered = WidgetVisuals {
        fg_stroke: Stroke::new(1.0, TEXT_COLOR),
        bg_stroke: Stroke::new(1.0, HOVERED_COLOR),
        bg_fill: HOVERED_COLOR,
        rounding: Rounding {
            nw: 5.0,
            ne: 5.0,
            se: 5.0,
            sw: 5.0,
        },
        expansion: 1.5,
    };

    let active = WidgetVisuals {
        fg_stroke: Stroke::new(1.0, TEXT_COLOR),
        bg_stroke: Stroke::new(1.0, ACTIVE_COLOR),
        bg_fill: ACTIVE_COLOR,
        rounding: Rounding {
            nw: 5.0,
            ne: 5.0,
            se: 5.0,
            sw: 5.0,
        },
        expansion: 1.5,
    };

    let open = WidgetVisuals {
        fg_stroke: Stroke::new(1.0, TEXT_COLOR),
        bg_stroke: Stroke::new(1.0, TEXT_COLOR),
        bg_fill: ACTIVE_COLOR,
        rounding: Rounding {
            nw: 5.0,
            ne: 5.0,
            se: 5.0,
            sw: 5.0,
        },
        expansion: 1.5,
    };

    let widgets = egui::style::Widgets {
        noninteractive: non_interactive_style,
        inactive: inactive,
        hovered: hovered,
        active: active,
        open: open,
    };
    widgets
}
fn get_app_spacing() -> Spacing {
    let spacing = Spacing {
        item_spacing: Vec2::new(10.0, 10.0),
        ..Default::default()
    };
    spacing
}

fn get_app_selection() -> Selection {
    let selection = Selection {
        bg_fill: HOVERED_COLOR,
        stroke: Stroke::new(1.0, TEXT_COLOR),
    };
    selection
}
fn get_app_visuals() -> Visuals {
    let visuals = Visuals {
        widgets: get_widget_visual_settings(),
        override_text_color: Some(TEXT_COLOR),
        window_fill: BG_COLOR,
        panel_fill: BG_COLOR,
        selection: get_app_selection(),
        extreme_bg_color: PLOT_BG_COLOR,
        ..Default::default()
    };
    visuals
}
pub fn get_app_style() -> egui::Style {
    let mut style = egui::Style::default();

    style.visuals.widgets = get_widget_visual_settings();
    style.spacing = get_app_spacing();
    style.visuals = get_app_visuals();

    style
}
