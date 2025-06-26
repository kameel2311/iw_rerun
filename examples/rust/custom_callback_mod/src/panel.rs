use std::time::Instant;

use crate::comms::{protocol::Message, viewer::ControlViewerHandle};
use rerun::external::{
    eframe,
    egui::{self},
    re_log::ResultExt,
    re_ui::{UiExt, list_item},
    re_viewer,
};

#[derive(Default)]
pub struct ControlStates {
    pub last_resource_update: Option<Instant>,
    pub controls_view: ControlsView,
    pub message_kind: ObjectKind,
    pub dynamic_offset_percentage: f32,
}

#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub enum ObjectKind {
    #[default]
    Point3d,
    Box3d,
}

#[derive(Default)]
pub struct ControlsView {
    pub key_sequence: Vec<String>,
}

pub struct Control {
    app: re_viewer::App,
    states: ControlStates,
    handle: ControlViewerHandle,
}

impl eframe::App for Control {
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        // Store viewer state on disk
        self.app.save(storage);
    }

    /// Called whenever we need repainting, which could be 60 Hz.
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        // First add our panel(s):
        egui::TopBottomPanel::bottom("timeline_panel")
            .resizable(false)
            .default_height(60.0)
            .show(ctx, |ui| {
                self.ui(ui);
            });
        self.app.update(ctx, frame);
    }
    // let time_secs = self.app.current_timeline_time();
}

impl Control {
    pub fn new(app: re_viewer::App, handle: ControlViewerHandle) -> Self {
        Control {
            app,
            states: ControlStates {
                dynamic_offset_percentage: 0.0,
                ..Default::default()
            },
            handle,
        }
    }

    fn ui(&mut self, ui: &mut egui::Ui) {
        ui.spacing_mut().item_spacing.y = 9.0;
        ui.style_mut().spacing.slider_width = ui.available_width() * 0.92;

        list_item::list_item_scope(ui, "Timeline", |ui| {
            ui.spacing_mut().item_spacing.y = ui.ctx().style().spacing.item_spacing.y;
            ui.section_collapsing_header("Timeline")
                .default_open(true)
                .show(ui, |ui| {
                    dynamic_timeline_ui(ui, self.handle.clone(), &mut self.states);
                });
        });
    }
}

fn dynamic_timeline_ui(ui: &mut egui::Ui, handle: ControlViewerHandle, states: &mut ControlStates) {
    ui.add_space(5.0);

    // Use horizontal layout for better width control
    ui.horizontal(|ui| {
        ui.label("Timeline:");
        ui.add_space(5.0);

        // Get the percentage value before borrowing
        let percentage_text = format!("{:.0}%", states.dynamic_offset_percentage * 100.0);
        // Timeline slider with percentage display

        ui.label(percentage_text);
        // let changed = ui
        //     .add(
        //         egui::Slider::new(
        //             &mut states.dynamic_offset_percentage,
        //             0.0_f32..=1.0_f32,
        //         )
        //         .show_value(false)
        //         .clamp_to_range(true)
        //         .step_by(0.01)
        //         .fixed_decimals(2)
        //     )
        //     .changed();

        // if changed {
        //     handle
        //         .send(Message::Timeline {
        //             offset_percentage: states.dynamic_offset_percentage,
        //         })
        //         .warn_on_err_once("Failed to send timeline update");
        // }
        
        let slider_response = ui.add(
            egui::Slider::new(
                &mut states.dynamic_offset_percentage,
                0.0_f32..=1.0_f32,
            )
            .show_value(false)
            .clamp_to_range(true)
            .step_by(0.01)
            .fixed_decimals(2),
        );

        // Only send if user released the mouse after interacting with slider
        if slider_response.drag_stopped() {
            handle
                .send(Message::Timeline {
                    offset_percentage: states.dynamic_offset_percentage,
                })
                .warn_on_err_once("Failed to send timeline update");
        }
    });

    ui.add_space(5.0);
}