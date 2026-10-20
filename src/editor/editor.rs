use egui::{Context, Ui, Panel};

pub struct Editor {

}

impl Editor {
    pub fn new() -> Editor {
        Editor {
        }
    }

    pub fn ui(&mut self, ctx: &Context, ui: &mut Ui) {
        Panel::right("Editor")
            .default_size(300.0)
            .resizable(true)
            .show(ui, |ui| {
                ui.label("Resizable side panel");
            });
    }
}