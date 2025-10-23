#[cfg(feature = "gui")]
use const_format::formatcp;
#[cfg(feature = "gui")]
use egui_phosphor::variants::light as Icon;

/// Container for all GUI string objects.
#[cfg(feature = "gui")]
pub struct GuiStr;

#[cfg(feature = "gui")]
impl GuiStr {
    /// String used for the image name tab.
    pub const TAB_PROCESS_NAME: &str  = formatcp!("{} Image Name", Icon::NOTEPAD);
    /// String used for the Process ID tab.
    pub const TAB_PROCESS_ID: &str    = formatcp!("{} Process ID", Icon::IDENTIFICATION_BADGE);
    /// String used for the text input label.
    pub const TEXT_INPUT_LABEL: &str  = formatcp!("{} Input", Icon::TEXTBOX);
    /// String used for the process list label.
    pub const TEXT_LIST_LABEL: &str   = formatcp!("{} Processes", Icon::CPU);
    /// String used for the module list label.
    pub const TEXT_MODULE_LABEL: &str = formatcp!("{} Modules", Icon::PUZZLE_PIECE);
    pub const TEXT_STATUS_LABEL: &str = formatcp!("{} Status", Icon::SPINNER_BALL);
    /// String used for the search button.
    pub const BUTTON_SEARCH: &str     = Icon::MAGNIFYING_GLASS;
    /// String used for the add module button.
    pub const BUTTON_ADD: &str        = Icon::PLUS;
    /// String used for the remove module button.
    pub const BUTTON_REMOVE: &str     = Icon::MINUS;
    /// String used for the reset modules button.
    pub const BUTTON_RESET: &str      = Icon::ARROWS_CLOCKWISE;
    /// String used for the inject button.
    pub const BUTTON_INJECT: &str     = formatcp!("{} Inject", Icon::SYRINGE);
    /// String used for the cleanup checkbox.
    pub const CHECKBOX_CLEANUP: &str  = formatcp!("{} Cleanup", Icon::BROOM);
    /// String used for the free library checkbox.
    pub const CHECKBOX_FREE: &str     = formatcp!("{} Remove Library", Icon::TRASH);
    /// String used for the UWP checkbox.
    pub const CHECKBOX_UWP:     &str  = formatcp!("{} UWP Application", Icon::PACKAGE);
}