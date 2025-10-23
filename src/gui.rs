use crate::strings::GuiStr;
use deject::injector::ModuleEntry;
use std::sync::mpsc::{self, Receiver};
use eframe::{App, Frame};
use egui::{Button, CentralPanel, Checkbox, Color32, ComboBox, Context, Label, RichText, ScrollArea, TextEdit, Widget};
use egui_flex::{item, Flex, FlexAlign};
use windows::Win32::Foundation::HWND;

/// Graphical interface implementation.
pub struct Deject {
    /// Window handle associated with this application.
    window:    HWND,
    /// Current injection status.
    status:    String,
    /// Thread listener for status updates.
    reciever:  Option<Receiver<String>>,
    /// Determines if a process name should be used to search.
    use_name:  bool,
    /// Text box input buffer.
    in_buf:    String,
    /// List of found processes.
    ps_list:   Vec<u32>,
    /// Current process index.
    ps_cur:    usize,
    /// List of added modules.
    mod_list:  Vec<ModuleEntry>,
    /// Current module selection.
    mod_sel:   Option<ModuleEntry>,
    /// Determines if we should clean up.
    cleanup:   bool,
    /// Determines if we should free the library after.
    do_free:   bool,
    /// Determines if we should handle UWP apps.
    is_uwp:    bool,
    /// Current injection state.
    injecting: bool
}

impl Default for Deject {
    fn default() -> Self {
        Self {
            window:    HWND::default(),
            status:    "Idle".to_owned(),
            reciever:  None,
            use_name:  true,
            in_buf:    String::new(),
            ps_list:   vec![],
            ps_cur:    0,
            mod_list:  vec![],
            mod_sel:   None,
            cleanup:   false,
            do_free:   false,
            is_uwp:    false,
            injecting: false
        }
    }
}

impl Deject {
    /// Adds an `HWND` to the app state.
    pub fn with_hwnd(mut self, window: HWND) -> Self {
        self.window = window;
        self
    }
}

impl App for Deject {
    fn update(&mut self, ctx: &Context, _frame: &mut Frame) {
        CentralPanel::default().show(ctx, |ui| {
            // Render tabs.
            Flex::horizontal().w_full().grow_items(1.0).gap([2.0, 0.0].into()).show(ui, |flex| {
                let name_tab = Button::new(GuiStr::TAB_PROCESS_NAME).selected(self.use_name);
                if flex.add(item(), name_tab).clicked() && !self.use_name {
                    self.use_name = true;
                    self.in_buf.clear();
                }

                let id_tab = Button::new(GuiStr::TAB_PROCESS_ID).selected(!self.use_name);
                if flex.add(item(), id_tab).clicked() && self.use_name {
                    self.use_name = false;
                    self.in_buf.clear();
                }
            });

            ui.separator();
            ui.add(Label::new(GuiStr::TEXT_INPUT_LABEL).selectable(false));

            // Render text entry and search button.
            Flex::horizontal().w_full().gap([2.0, 0.0].into()).show(ui, |flex| {
                if flex.add(item().grow(1.0), TextEdit::singleline(&mut self.in_buf)).changed() && !self.use_name {
                    self.in_buf.retain(|ch| ch.is_ascii_digit());
                }

                flex.add_ui(item().grow(0.0), |ui| {
                    if ui.add_enabled(self.use_name, Button::new(GuiStr::BUTTON_SEARCH)).clicked() {

                    }
                });
            });

            ui.separator();
            ui.add(Label::new(GuiStr::TEXT_LIST_LABEL).selectable(false));

            // Render process list.
            ui.add_enabled_ui(self.use_name && self.ps_list.len() > 0, |ui| {
                ComboBox::from_id_salt("ps-list").width(ui.available_width()).show_index(
                    ui,
                    &mut self.ps_cur,
                    self.ps_list.len(),
                    |i| {
                        match self.ps_list.len() {
                            0 => "None".to_owned(),
                            _ => self.ps_list[i].to_string()
                        }
                    }
                );
            });

            ui.separator();

            // Render module list and buttons.
            Flex::horizontal().w_full().gap([2.0, 0.0].into()).align_items(FlexAlign::Center).show(ui, |flex| {
                flex.add(item(), Label::new(GuiStr::TEXT_MODULE_LABEL).selectable(false));
                flex.grow();

                if flex.add(item(), Button::new(GuiStr::BUTTON_RESET)).clicked() {

                }

                if flex.add(item(), Button::new(GuiStr::BUTTON_REMOVE)).clicked() {

                }

                if flex.add(item(), Button::new(GuiStr::BUTTON_ADD)).clicked() {

                }
            });

            let row_height  = ui.spacing().interact_size.y;
            ScrollArea::vertical()
                .max_height(row_height * 4.5)
                .auto_shrink(false)
                .show_rows(ui, row_height, self.mod_list.len(), |ui, range| {
                    for mod_index in range {
                        let text_data   = self.mod_list[mod_index].to_string();
                        let is_selected = match &self.mod_sel {
                            Some(inner_mod) => self.mod_list[mod_index] == *inner_mod,
                            None            => false
                        };

                        let mod_button  = Button::selectable(is_selected, text_data)
                            .min_size([ui.available_width(), 0.0].into());

                        if ui.add(mod_button).clicked() {
                            if !is_selected {
                                self.mod_sel = Some(self.mod_list[mod_index].clone());
                            } else {
                                match self.mod_sel.is_none() {
                                    true  => self.mod_sel = Some(self.mod_list[mod_index].clone()),
                                    false => self.mod_sel = None
                                }
                            }
                        }
                    }
                });

            ui.separator();
            ui.add(Label::new(GuiStr::TEXT_STATUS_LABEL).selectable(false));

            // Render status.
            ui.add(Label::new(RichText::from(self.status.clone()).color(Color32::WHITE)).selectable(false));

            ui.separator();

            // Render options.
            Flex::horizontal().w_full().show(ui, |flex| {
                flex.add(item(), Label::new(GuiStr::CHECKBOX_CLEANUP).selectable(false));
                flex.grow();
                flex.add(item(), Checkbox::without_text(&mut self.cleanup));
            });

            Flex::horizontal().w_full().show(ui, |flex| {
                flex.add(item(), Label::new(GuiStr::CHECKBOX_FREE).selectable(false));
                flex.grow();
                flex.add(item(), Checkbox::without_text(&mut self.do_free));
            });

            Flex::horizontal().w_full().show(ui, |flex| {
                flex.add(item(), Label::new(GuiStr::CHECKBOX_UWP).selectable(false));
                flex.grow();
                flex.add_ui(item(), |ui| {
                    ui.add_enabled_ui(self.ps_list.len() != 0, |ui| {
                        ui.add(Checkbox::without_text(&mut self.is_uwp));
                    })
                });
            });

            // Render inject button.
            ui.horizontal_centered(|ui| {
                ui.add_enabled_ui(self.ps_list.len() != 0, |ui| {
                    if ui.add(Button::new(GuiStr::BUTTON_INJECT).min_size([ui.available_width(), 0.0].into())).clicked() {

                    }
                });
            });
        });
    }
}