use std::mem;
use std::path::PathBuf;
use std::thread::{self, JoinHandle};
use crate::strings::GuiStr;
use deject::error::InjectorResult;
use deject::injector::{Injector, ModuleEntry};
use eframe::{App, Frame};
use egui::{Button, CentralPanel, Checkbox, ComboBox, Context, Label, ScrollArea, TextEdit};
use egui_flex::{item, Flex, FlexAlign};
use windows::Win32::Foundation::{HWND, MAX_PATH};
use windows::Win32::UI::Controls::Dialogs::{GetOpenFileNameW, OPENFILENAMEW, OFN_PATHMUSTEXIST, OFN_FILEMUSTEXIST};
use windows::core::{PCWSTR, PWSTR, w};

/// Graphical interface implementation.
pub struct Deject {
    /// Window handle associated with this application.
    window:     HWND,
    /// Current injection status.
    status:     String,
    /// Text box input buffer.
    in_buf:     String,
    /// List of found processes.
    ps_list:    Vec<u32>,
    /// Current process index.
    ps_cur:     usize,
    /// List of added modules.
    mod_list:   Vec<ModuleEntry>,
    /// Current module selection.
    mod_sel:    Option<ModuleEntry>,
    /// Determines if we should clean up.
    cleanup:    bool,
    /// Current injection state.
    injecting:  bool,
    /// Index of next module to inject.
    cur_inject: usize,
    /// Current thread handle.
    cur_thread: Option<JoinHandle<InjectorResult<()>>>
}

impl Default for Deject {
    fn default() -> Self {
        Self {
            window:     HWND::default(),
            status:     "Idle".to_owned(),
            in_buf:     String::new(),
            ps_list:    vec![],
            ps_cur:     0,
            mod_list:   vec![],
            mod_sel:    None,
            cleanup:    false,
            injecting:  false,
            cur_inject: 0,
            cur_thread: None
        }
    }
}

impl Deject {
    /// Adds an `HWND` to the app state.
    pub fn with_hwnd(mut self, window: HWND) -> Self {
        self.window = window;
        self
    }

    /// Retrieves a file path using a dialog box.
    fn get_file(&self) -> Option<PathBuf> {
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
                Ok(path)  => return Some(PathBuf::from(path)),
                Err(_err) => return None
            }
        }

        None
    }
}

impl App for Deject {
    fn update(&mut self, ctx: &Context, _frame: &mut Frame) {
        CentralPanel::default().show(ctx, |ui| {
            // Handle injection creation.
            if self.injecting && self.cur_inject != self.mod_list.len() {
                if self.cur_thread.is_none() {
                    // Clone necessary data.
                    let pid    = self.ps_list[self.ps_cur];
                    let module = self.mod_list[self.cur_inject].get_path();
                    let clean  = self.cleanup;

                    self.status = format!("Waiting for {} to exit", self.mod_list[self.cur_inject].get_name());
                    self.cur_thread = Some(thread::spawn(move || {
                        let injector = Injector::from_pid(pid, clean)?;
                        injector.inject_dll(&module)
                    }));
                } else {
                    if let Some(handle) = self.cur_thread.take() {
                        if handle.is_finished() {
                            if let Ok(join_result) = handle.join() {
                                match join_result {
                                    Ok(_void) => self.status = "Idle".to_string(),
                                    Err(err)  => self.status = err.to_string()
                                }
                            }

                            self.cur_inject += 1;
                        } else {
                            self.cur_thread = Some(handle);
                        }
                    }
                }
            } else {
                self.injecting = false;
            }

            ui.add(Label::new(GuiStr::TEXT_INPUT_LABEL).selectable(false));

            // Render text entry and search button.
            ui.add_enabled_ui(!self.injecting, |ui| {
                Flex::horizontal().w_full().gap([2.0, 0.0].into()).show(ui, |flex| {
                    flex.add(item().grow(1.0), TextEdit::singleline(&mut self.in_buf));

                    flex.add_ui(item().grow(0.0), |ui| {
                        if ui.add_enabled(!self.in_buf.is_empty(), Button::new(GuiStr::BUTTON_SEARCH)).clicked() {
                            match Injector::find_by_name(&self.in_buf) {
                                Ok(list) => {
                                    self.status = "Idle".to_owned();
                                    self.ps_list = list;
                                },
                                Err(err) => {
                                    self.status = err.to_string();
                                    self.ps_list.clear();
                                } 
                            }
                        }
                    });
                });
            });

            ui.separator();
            ui.add(Label::new(GuiStr::TEXT_LIST_LABEL).selectable(false));

            // Render process list.
            ui.add_enabled_ui(self.ps_list.len() > 0 && !self.injecting, |ui| {
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
                        self.mod_sel = None;
                        self.mod_list.clear();
                    }
                });

                flex.add_ui(item(), |ui| {
                    if ui.add_enabled(self.mod_sel.is_some() && !self.injecting, Button::new(GuiStr::BUTTON_REMOVE)).clicked() {
                        if let Some(module) = &self.mod_sel {
                            if let Some(index) = self.mod_list.iter().position(|mo| mo == module) {
                                self.mod_sel = None;
                                self.mod_list.remove(index);
                            }
                        }
                    }
                });

                flex.add_ui(item(), |ui| {
                    if ui.add_enabled(!self.injecting, Button::new(GuiStr::BUTTON_ADD)).clicked() {
                        match self.get_file() {
                            Some(file) => self.mod_list.push(ModuleEntry::new(file)),
                            None       => {}
                        }
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
            ui.add(Label::new(self.status.clone()).selectable(false));

            ui.separator();

            // Render options.
            Flex::horizontal().w_full().show(ui, |flex| {
                flex.add(item(), Label::new(GuiStr::CHECKBOX_CLEANUP).selectable(false));
                flex.grow();
                flex.add_ui(item(), |ui| {
                    ui.add_enabled(!self.injecting, Checkbox::without_text(&mut self.cleanup));
                });
            });

            // Render stop and inject buttons.
            ui.horizontal_centered(|ui| {
                let inject_button = Button::new(GuiStr::BUTTON_INJECT).min_size([ui.available_width(), 0.0].into());
                if ui.add_enabled((self.ps_list.len() != 0) && (self.mod_list.len() != 0) && !self.injecting, inject_button).clicked() {
                    self.cur_inject = 0;
                    self.injecting  = true;
                }
            });
        });
    }
}
