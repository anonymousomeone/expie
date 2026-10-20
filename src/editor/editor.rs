use egui::{Context, Ui, Panel};

use crate::State;

pub struct Editor {

}

impl Editor {
    pub fn new() -> Editor {
        Editor {
        }
    }

    pub fn ui(&mut self, ctx: &Context, ui: &mut Ui, state: &mut State) {
        Panel::right("Editor")
            .default_size(300.0)
            .resizable(true)
            .show(ui, |ui| {
                ui.take_available_width();
                ui.label("Resizable side panel");
            });
    }
}