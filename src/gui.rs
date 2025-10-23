use std::mem;
use crate::strings::GuiStr;
use deject::injector::ModuleEntry;
use std::sync::mpsc::{self, Receiver};
use eframe::{App, Frame};
use egui::{Button, CentralPanel, Checkbox, Color32, ComboBox, Context, Label, RichText, ScrollArea, TextEdit, Vec2};
use egui_flex::{item, Flex, FlexAlign};
use windows::Win32::Foundation::{HWND, MAX_PATH};
use windows::Win32::UI::Controls::Dialogs::{GetOpenFileNameW, OPENFILENAMEW, OFN_PATHMUSTEXIST, OFN_FILEMUSTEXIST};
use windows::core::{PCWSTR, PWSTR, w};

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

    fn get_file(&self) -> Option<String> {
        let mut file_buf = [0; MAX_PATH as usize];

        let mut open_file         = OPENFILENAMEW::default();
        open_file.lStructSize     = mem::size_of::<OPENFILENAMEW>() as u32;
        open_file.hwndOwner       = self.window;
        open_file.lpstrFile       = PWSTR(file_buf.as_mut_ptr());
        open_file.nMaxFile        = MAX_PATH;
        open_file.lpstrFilter     = w!("DLL Files (*.dll)\0*.dll\0");
        open_file.nFilterIndex    = 1;
        open_file.lpstrFileTitle  = PWSTR::null();
        open_file.nMaxFileTitle   = 0;
        open_file.lpstrInitialDir = PCWSTR::null();
        open_file.Flags           = OFN_PATHMUSTEXIST | OFN_FILEMUSTEXIST;

        if unsafe{ GetOpenFileNameW(&mut open_file) }.into() {
            let buf_len   = file_buf.iter().take_while(|&&ch| ch != 0).count();
            let buf_slice = &file_buf[..buf_len];

            match String::from_utf16(buf_slice) {
                Ok(path)  => return Some(path),
                Err(_err) => return None
            }
        }

        None
    }
}

impl App for Deject {
    fn update(&mut self, ctx: &Context, _frame: &mut Frame) {
        CentralPanel::default().show(ctx, |ui| {
            // Render tabs.
            ui.add_enabled_ui(!self.injecting, |ui| {
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
            });

            ui.separator();
            ui.add(Label::new(GuiStr::TEXT_INPUT_LABEL).selectable(false));

            // Render text entry and search button.
            ui.add_enabled_ui(!self.injecting, |ui| {
                Flex::horizontal().w_full().gap([2.0, 0.0].into()).show(ui, |flex| {
                    if flex.add(item().grow(1.0), TextEdit::singleline(&mut self.in_buf)).changed() && !self.use_name {
                        self.in_buf.retain(|ch| ch.is_ascii_digit());
                    }

                    flex.add_ui(item().grow(0.0), |ui| {
                        if ui.add_enabled(self.use_name, Button::new(GuiStr::BUTTON_SEARCH)).clicked() {

                        }
                    });
                });
            });

            ui.separator();
            ui.add(Label::new(GuiStr::TEXT_LIST_LABEL).selectable(false));

            // Render process list.
            ui.add_enabled_ui(self.use_name && self.ps_list.len() > 0 && !self.injecting, |ui| {
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

                flex.add_ui(item(), |ui| {
                    if ui.add_enabled((self.mod_list.len() != 0) && !self.injecting, Button::new(GuiStr::BUTTON_RESET)).clicked() {

                    }
                });

                flex.add_ui(item(), |ui| {
                    if ui.add_enabled(self.mod_sel.is_some() && !self.injecting, Button::new(GuiStr::BUTTON_REMOVE)).clicked() {

                    }
                });

                flex.add_ui(item(), |ui| {
                    if ui.add_enabled(!self.injecting, Button::new(GuiStr::BUTTON_ADD)).clicked() {
                        
                    }
                });
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

                        if ui.add(mod_button).clicked() && !self.injecting {
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
                flex.add_ui(item(), |ui| {
                    ui.add_enabled(!self.injecting, Checkbox::without_text(&mut self.cleanup));
                });
            });

            Flex::horizontal().w_full().show(ui, |flex| {
                flex.add(item(), Label::new(GuiStr::CHECKBOX_FREE).selectable(false));
                flex.grow();
                flex.add_ui(item(), |ui| {
                    ui.add_enabled(!self.injecting, Checkbox::without_text(&mut self.do_free));
                });
            });

            Flex::horizontal().w_full().show(ui, |flex| {
                flex.add(item(), Label::new(GuiStr::CHECKBOX_UWP).selectable(false));
                flex.grow();
                flex.add_ui(item(), |ui| {
                    ui.add_enabled((self.ps_list.len() != 0) && !self.injecting, Checkbox::without_text(&mut self.is_uwp));
                });
            });

            // Render stop and inject buttons.
            ui.horizontal_centered(|ui| {
                ui.spacing_mut().item_spacing = [2.0, 0.0].into();
                let button_size = Vec2::new(ui.available_width() / 2.0 - 2.0, 0.0);

                let stop_button = Button::new(GuiStr::BUTTON_STOP).min_size(button_size);
                if ui.add_enabled(self.injecting, stop_button).clicked() {

                }                    

                let inject_button = Button::new(GuiStr::BUTTON_INJECT).min_size(button_size);
                if ui.add_enabled((self.ps_list.len() != 0) && !self.injecting, inject_button).clicked() {

                }
            });
        });
    }
}