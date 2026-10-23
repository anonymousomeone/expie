use egui::{Button, Color32, Context, CornerRadius, Frame, Layout, Margin, Panel, ScrollArea, Stroke, Ui};

use crate::State;

pub struct Editor {
    ui_state: EditorUiState,
}

#[derive(Default)]
pub struct EditorUiState {
    pub selected_entity_id: Option<usize>,
}

impl Editor {
    pub fn new() -> Editor {
        Editor {
            ui_state: EditorUiState::default(),
        }
    }

    pub fn ui(&mut self, ctx: &Context, ui: &mut Ui, state: &mut State) {
        Panel::left("Workspads")
            .default_size(300.0)
            .resizable(true)
            .frame(
                Frame::side_top_panel(&ctx.global_style())
                    .inner_margin(Margin::same(1))
            )
            .show(ui, |ui| {
                ui.take_available_width();
                ui.label("Workspads");

                self.ui_entity_viewer(ctx, ui, state);
            });

        Panel::right("Editor")
            .default_size(300.0)
            .resizable(true)
            .show(ui, |ui| {
                ui.take_available_width();
                ui.label("Resizable side panel");
            });
    }

    fn ui_entity_viewer(&mut self, ctx: &Context, ui: &mut Ui, state: &mut State) {
        Frame::new()
            .stroke(Stroke::new(1.0, Color32::DARK_GRAY))
            .show(ui, |ui| {
                let row_height = ui.spacing().interact_size.y;

                ui.with_layout(Layout::top_down_justified(egui::Align::LEFT), |ui| {
                    ScrollArea::vertical()
                        .auto_shrink([false; 2])
                        .max_height(500.0)
                        .show_rows(ui, row_height, state.entities.len(), |ui, row_range| {
                            for id in row_range {
                                let button = Button::new(format!("Entity {}", id))
                                    .corner_radius(CornerRadius::ZERO);

                                if ui.add(button).clicked() {
                                    match self.ui_state.selected_entity_id {
                                        Some(selected_id) => {
                                            let entity = state.entities.get_mut(selected_id).unwrap();
                                            entity.set_outline(false);
                                        },
                                        None => {
                                            self.ui_state.selected_entity_id = Some(id);
                                        }
                                    }

                                    state.entities.get_mut(id).unwrap().set_outline(true);
                                }
                            }
                        });
                });
            });
    }
}