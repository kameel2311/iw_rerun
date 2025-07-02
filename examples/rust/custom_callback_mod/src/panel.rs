use std::time::Instant;

use crate::comms::{protocol::Message, viewer::ControlViewerHandle, viewer::SharedStateHandle};
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
    pub category: String, // For the labeling tool
    pub description: String, // For the labeling tool
}

#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub enum ObjectKind {
    #[default]
    Point3d,
    Box3d,
}

#[derive(Default)]
pub struct ControlsView {
    pub category: Vec<String>,
    pub description: Vec<String>,
}

pub struct Control {
    app: re_viewer::App,
    states: ControlStates,
    handle: ControlViewerHandle,
    shared_state: SharedStateHandle,
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
        egui::SidePanel::right("labeling_panel")
            .resizable(false)
            .default_width(300.0)
            .show(ctx, |ui| {
                self.ui_side_panel(ui);
            });
        self.app.update(ctx, frame);
    }
    // let time_secs = self.app.current_timeline_time();
}

impl Control {
    pub fn new(app: re_viewer::App, handle: ControlViewerHandle, shared_state: SharedStateHandle,) -> Self {
        Control {
            app,
            states: ControlStates {
                dynamic_offset_percentage: 0.0,
                ..Default::default()
            },
            handle,
            shared_state,
        }
    }

    fn ui(&mut self, ui: &mut egui::Ui) {
        ui.spacing_mut().item_spacing.y = 9.0;
        ui.style_mut().spacing.slider_width = ui.available_width() * 0.92;

        let mut buffer_length_value = 1.0;
        let mut bag_duration_value = 1.0;
        if let Ok(state) = self.shared_state.try_lock() {
            if let Some(Message::BagAndBuffer { bag_duration, buffer_length}) = &state.last_received_message_bag_buffer {
                ui.label(format!(
                    "Received a message: Bag Duration: {:.2}s, Buffer Length: {:.2}s",
                    bag_duration, buffer_length
                ));
                // Update the states with the received values
                bag_duration_value  = *bag_duration;
                buffer_length_value = *buffer_length;
                
            } else if state.last_received_message_bag_buffer.is_some() {
                ui.label("Received a non-timeline message.");
            } else {
                ui.label("No message received yet.");
            }
        } else {
            ui.label("Receiving...");
        }

        list_item::list_item_scope(ui, "Timeline", |ui| {
            ui.spacing_mut().item_spacing.y = ui.ctx().style().spacing.item_spacing.y;
            ui.section_collapsing_header("Timeline")
                .default_open(true)
                .show(ui, |ui| {
                    dynamic_timeline_ui(ui, self.handle.clone(), &mut self.states, buffer_length_value, bag_duration_value);
                });
        });
    }

    fn ui_side_panel(&mut self, ui: &mut egui::Ui) {
        ui.spacing_mut().item_spacing.y = 9.0;
        if let Ok(mut state) = self.shared_state.try_lock() {
            if let Some(Message::LabelingTool {category, description}) = &state.last_received_message_labelling_tool {
                ui.label(format!("Loaded Label"));
                // Update the states with the received values
                self.states.category = category.clone();
                self.states.description = description.clone();
                state.last_received_message_labelling_tool = None;
            }

        list_item::list_item_scope(ui, "Labeling Tool", |ui| {
            ui.spacing_mut().item_spacing.y = ui.ctx().style().spacing.item_spacing.y;
            ui.section_collapsing_header("Labeling Tool")
                .default_open(true)
                .show(ui, |ui| {
                    labeling_tool_ui(ui, self.handle.clone(), &mut self.states);
                });
            });
        }
    }
}

fn dynamic_timeline_ui(ui: &mut egui::Ui, handle: ControlViewerHandle, states: &mut ControlStates, buffer_length_value: f32, bag_duration_value: f32) {
    ui.add_space(5.0);
    ui.horizontal(|ui| {
        // Add a Backward button
        if ui.add(egui::Button::new("Backward 10%")).clicked(){
            // Send a message to move the timeline backward
            handle
                .send(Message::Timeline {
                    offset_percentage: states.dynamic_offset_percentage - 0.1,
                })
                .warn_on_err_once("Failed to send timeline update");
            states.dynamic_offset_percentage -= 0.1;
            // Ensure the percentage does not go below 0.0
            if states.dynamic_offset_percentage < 0.0 {
                states.dynamic_offset_percentage = 0.0;
            }
            states.category.clear();
            states.description.clear();
        }
        // Add a Forward button
        if ui.add(egui::Button::new("Forward 10%")).clicked() {
            // Send a message to move the timeline forward
            handle
                .send(Message::Timeline {
                    offset_percentage: states.dynamic_offset_percentage + 0.1,
                })
                .warn_on_err_once("Failed to send timeline update");
            states.dynamic_offset_percentage += 0.1;
            // Ensure the percentage does not exceed 1.0
            if states.dynamic_offset_percentage > 1.0 {
                states.dynamic_offset_percentage = 1.0;
            }
            states.category.clear();
            states.description.clear();
        }
    });
    ui.horizontal(|ui| {
        if ui.add(egui::Button::new("Previous Buffer")).clicked() {
            // Send a message to move the timeline backward
            handle
                .send(Message::Timeline {
                    offset_percentage: states.dynamic_offset_percentage - buffer_length_value / bag_duration_value,
                })
                .warn_on_err_once("Failed to send timeline update");
            states.dynamic_offset_percentage -= buffer_length_value / bag_duration_value;
            // Ensure the percentage does not go below 0.0
            if states.dynamic_offset_percentage < 0.0 {
                states.dynamic_offset_percentage = 0.0;
            }
            states.category.clear();
            states.description.clear();
        }
        if ui.add(egui::Button::new("Next Buffer")).clicked() {
            // Send a message to move the timeline forward
            handle
                .send(Message::Timeline {
                    offset_percentage: states.dynamic_offset_percentage + buffer_length_value / bag_duration_value,
                })
                .warn_on_err_once("Failed to send timeline update");
            states.dynamic_offset_percentage += buffer_length_value / bag_duration_value;
            // Ensure the percentage does not exceed 1.0
            if states.dynamic_offset_percentage > 1.0 {
                states.dynamic_offset_percentage = 1.0;
            }
            states.category.clear();
            states.description.clear();
        }
    });
    ui.add_space(5.0);

    // Use horizontal layout for better width control
    ui.horizontal(|ui| {
        ui.label("Timeline:");
        
        // Get the percentage value before borrowing
        let percentage_text = format!("{:.0}%", states.dynamic_offset_percentage * 100.0);
        // Timeline slider with percentage display

        ui.label(percentage_text);
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
            states.category.clear();
            states.description.clear();
        }
    });

    ui.add_space(5.0);
}

fn labeling_tool_ui(ui: &mut egui::Ui, handle: ControlViewerHandle, states: &mut ControlStates) {
    ui.spacing_mut().item_spacing.y = 9.0;

    // Add UI elements for the labeling tool here
    ui.label("Label the currently loaded buffer");
    // Example: Add a text input for labels
    ui.label("Category: (from the listed options or custom input, keep empty and submit to delete label)");

    // Select a category from presaved list
    let response = ui.add(egui::Button::new("Dynamic Object"));
    if response.clicked() {
        states.category = "Dynamic Object".to_string();
    }
    let response = ui.add(egui::Button::new("Unmapped Map Changes"));
    if response.clicked() {
        states.category = "Unmapped Map Changes".to_string();
    }
    let response = ui.add(egui::Button::new("Blocked FOV"));
    if response.clicked() {
        states.category = "Blocked FOV".to_string();
    }   
    ui.text_edit_singleline(&mut states.category);
    ui.label("Description:");
    ui.add(egui::TextEdit::multiline(&mut states.description));
    let response = ui.add(egui::Button::new("Submit Label"));
    if response.clicked() {
        // Handle the label submission
        println!("Label submitted: {} {}", states.category ,states.description);
        // Here you can send the label to the server or process it as needed
        handle
            .send(Message::LabelingTool {
                category: states.category.clone(),
                description: states.description.clone(),
            })
            .warn_on_err_once("Failed to send timeline update");
    }
}