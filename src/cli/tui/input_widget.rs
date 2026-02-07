// Input Widget - Helper to render tui-textarea
//
// Note: This is not a proper Widget implementation due to tui-textarea's API.
// Instead, we provide a helper function to render the textarea.

use ratatui::{
    Frame,
    layout::Rect,
};
use tui_textarea::TextArea;

/// Render a TextArea without borders (to avoid scrollback leakage)
pub fn render_input_widget<'a>(frame: &mut Frame, textarea: &'a TextArea<'a>, area: Rect, _prompt: &str) {
    // No border - render textarea directly to avoid scrollback leakage
    // The status bar border provides sufficient visual separation
    frame.render_widget(textarea, area);
}
