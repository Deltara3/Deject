#[cfg(feature = "gui")]
mod gui;
#[cfg(any(feature = "gui"))]
mod strings;

#[cfg(feature = "gui")]
mod gui_deps {
    pub use core::ffi::c_void;
    pub use crate::gui::Deject;
    pub use deject::{catch_unwrap, pcwstr};
    pub use egui_phosphor::Variant;
    pub use eframe::{run_native, NativeOptions, CreationContext, App};
    pub use egui::{ViewportBuilder, FontData, FontDefinitions, FontFamily, TextStyle};
    pub use raw_window_handle::{HasWindowHandle, RawWindowHandle};
    pub use windows::Win32::UI::WindowsAndMessaging::{MessageBoxW, MB_OK, MB_ICONERROR};
    pub use windows::Win32::Foundation::HWND;
    pub use windows::core::{PCWSTR, HSTRING, w};
    pub use std::error::Error;
}

#[cfg(feature = "gui")]
use gui_deps::*;

fn main() {
    if cfg!(feature = "gui") {
        let viewport = ViewportBuilder::default()
            .with_inner_size([320.0, 378.0])
            .with_resizable(false)
            .with_maximize_button(false);

        let options = NativeOptions {
            viewport,
            ..Default::default()
        };

        catch_unwrap!(run_native("Deject", options, Box::new(gui_callback)), |err| {
            let _message_result = unsafe { MessageBoxW(
                None,
                pcwstr!(err.to_string()),
                w!("Unhandled Exception"),
                MB_OK | MB_ICONERROR
            ) };
        });
    }
}

#[cfg(feature = "gui")]
fn gui_callback(cc: &CreationContext) -> Result<Box<dyn App>, Box<dyn Error + Send + Sync>> {
    let mut fonts = FontDefinitions::default();
    
    // We are using monospace, so inject fonts manually.
    let raw_ubuntu_data = include_bytes!("../assets/UbuntuSansMono-Regular.ttf");
    fonts.font_data.insert("ubuntu".into(), FontData::from_static(raw_ubuntu_data).into());
    fonts.font_data.insert("phosphor".into(), Variant::Light.font_data().into());

    if let Some(font_keys) = fonts.families.get_mut(&FontFamily::Monospace) {
        font_keys.insert(0, "ubuntu".into());
        font_keys.insert(1, "phosphor".into());
    }

    cc.egui_ctx.set_fonts(fonts);
    cc.egui_ctx.all_styles_mut(|style| {
        style.override_text_style = Some(TextStyle::Monospace);
    });

    // Get window handle and pass it to the app.
    match cc.window_handle()?.as_raw() {
        RawWindowHandle::Win32(handle) => {
            let hwnd_ptr = handle.hwnd.get() as *mut c_void;
            Ok(Box::new(Deject::default().with_hwnd(HWND(hwnd_ptr))))
        },
        _ => panic!("Unsupported operating system")
    }
}