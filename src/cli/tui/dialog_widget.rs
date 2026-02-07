// Dialog Widget - Ratatui Widget implementation for dialogs
//
// Renders dialogs inline with the TUI, matching the existing color scheme

use ratatui::{
    buffer::Buffer,
    layout::{Alignment, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, BorderType, Paragraph, Widget, Wrap},
};

use super::dialog::{Dialog, DialogOption, DialogType};

/// Widget for rendering dialogs
pub struct DialogWidget<'a> {
    pub dialog: &'a Dialog,
}

impl<'a> DialogWidget<'a> {
    /// Create a new dialog widget
    pub fn new(dialog: &'a Dialog) -> Self {
        Self { dialog }
    }

    /// Render a single-select dialog
    fn render_select(
        options: &[DialogOption],
        selected_index: usize,
        help: &Option<String>,
    ) -> Vec<Line<'static>> {
        let mut lines = Vec::new();

        // Add options with numbering
        for (idx, option) in options.iter().enumerate() {
            let is_selected = idx == selected_index;
            let number = idx + 1;

            // Format: "N. Label - Description"
            let prefix = if is_selected {
                Span::styled(
                    format!("{}. ", number),
                    Style::default()
                        .fg(Color::Cyan)
                        .add_modifier(Modifier::BOLD),
                )
            } else {
                Span::styled(format!("{}. ", number), Style::default().fg(Color::Gray))
            };

            let label = if is_selected {
                Span::styled(
                    option.label.clone(),
                    Style::default()
                        .fg(Color::White)
                        .bg(Color::Blue)
                        .add_modifier(Modifier::BOLD),
                )
            } else {
                Span::styled(option.label.clone(), Style::default().fg(Color::White))
            };

            let mut spans = vec![prefix, label];

            if let Some(desc) = &option.description {
                spans.push(Span::styled(
                    format!(" - {}", desc),
                    Style::default().fg(Color::DarkGray),
                ));
            }

            lines.push(Line::from(spans));
        }

        // Add help message if present
        if let Some(help_text) = help {
            lines.push(Line::from(""));
            lines.push(Line::from(Span::styled(
                help_text.clone(),
                Style::default().fg(Color::Yellow),
            )));
        }

        // Add keybindings hint
        lines.push(Line::from(""));
        lines.push(Line::from(Span::styled(
            "↑/↓ or j/k: Navigate | 1-9: Select directly | Enter: Confirm | Esc: Cancel",
            Style::default().fg(Color::DarkGray),
        )));

        lines
    }

    /// Render a multi-select dialog
    fn render_multiselect(
        options: &[DialogOption],
        selected_indices: &std::collections::HashSet<usize>,
        cursor_index: usize,
        help: &Option<String>,
    ) -> Vec<Line<'static>> {
        let mut lines = Vec::new();

        // Add options with checkboxes
        for (idx, option) in options.iter().enumerate() {
            let is_cursor = idx == cursor_index;
            let is_selected = selected_indices.contains(&idx);

            // Checkbox: [X] or [ ]
            let checkbox = if is_selected { "[X]" } else { "[ ]" };
            let checkbox_span = if is_cursor {
                Span::styled(
                    format!("{} ", checkbox),
                    Style::default()
                        .fg(Color::Cyan)
                        .add_modifier(Modifier::BOLD),
                )
            } else {
                Span::styled(format!("{} ", checkbox), Style::default().fg(Color::Gray))
            };

            let label = if is_cursor {
                Span::styled(
                    option.label.clone(),
                    Style::default()
                        .fg(Color::White)
                        .bg(Color::Blue)
                        .add_modifier(Modifier::BOLD),
                )
            } else {
                Span::styled(option.label.clone(), Style::default().fg(Color::White))
            };

            let mut spans = vec![checkbox_span, label];

            if let Some(desc) = &option.description {
                spans.push(Span::styled(
                    format!(" - {}", desc),
                    Style::default().fg(Color::DarkGray),
                ));
            }

            lines.push(Line::from(spans));
        }

        // Add help message if present
        if let Some(help_text) = help {
            lines.push(Line::from(""));
            lines.push(Line::from(Span::styled(
                help_text.clone(),
                Style::default().fg(Color::Yellow),
            )));
        }

        // Add keybindings hint
        lines.push(Line::from(""));
        lines.push(Line::from(Span::styled(
            "↑/↓ or j/k: Navigate | Space: Toggle | Enter: Confirm | Esc: Cancel",
            Style::default().fg(Color::DarkGray),
        )));

        lines
    }

    /// Render a text input dialog
    fn render_text_input(
        prompt: &str,
        input: &str,
        cursor_pos: usize,
        default: &Option<String>,
        help: &Option<String>,
    ) -> Vec<Line<'static>> {
        let mut lines = Vec::new();

        // Add prompt
        lines.push(Line::from(Span::styled(
            prompt.to_string(),
            Style::default().fg(Color::White),
        )));

        // Show default if present
        if let Some(def) = default {
            lines.push(Line::from(Span::styled(
                format!("(default: {})", def),
                Style::default().fg(Color::DarkGray),
            )));
        }

        lines.push(Line::from(""));

        // Render input field with cursor
        let mut input_spans = vec![Span::styled(
            "> ",
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )];

        // Add text before cursor
        if cursor_pos > 0 {
            input_spans.push(Span::styled(
                input[..cursor_pos].to_string(),
                Style::default().fg(Color::White),
            ));
        }

        // Add cursor
        if cursor_pos < input.len() {
            // Cursor on a character
            input_spans.push(Span::styled(
                input.chars().nth(cursor_pos).unwrap().to_string(),
                Style::default()
                    .fg(Color::Black)
                    .bg(Color::White)
                    .add_modifier(Modifier::BOLD),
            ));

            // Add text after cursor
            if cursor_pos + 1 < input.len() {
                input_spans.push(Span::styled(
                    input[cursor_pos + 1..].to_string(),
                    Style::default().fg(Color::White),
                ));
            }
        } else {
            // Cursor at end (show as block)
            input_spans.push(Span::styled(
                " ",
                Style::default()
                    .bg(Color::White)
                    .add_modifier(Modifier::BOLD),
            ));
        }

        lines.push(Line::from(input_spans));

        // Add help message if present
        if let Some(help_text) = help {
            lines.push(Line::from(""));
            lines.push(Line::from(Span::styled(
                help_text.clone(),
                Style::default().fg(Color::Yellow),
            )));
        }

        // Add keybindings hint
        lines.push(Line::from(""));
        lines.push(Line::from(Span::styled(
            "Type to enter text | Backspace: Delete | ←/→: Move cursor | Enter: Confirm | Esc: Cancel",
            Style::default().fg(Color::DarkGray),
        )));

        lines
    }

    /// Render a confirmation dialog
    fn render_confirm(
        prompt: &str,
        default: bool,
        selected: bool,
        help: &Option<String>,
    ) -> Vec<Line<'static>> {
        let mut lines = Vec::new();

        // Add prompt
        lines.push(Line::from(Span::styled(
            prompt.to_string(),
            Style::default().fg(Color::White),
        )));

        lines.push(Line::from(""));

        // Render Yes/No options
        let yes_style = if selected {
            Style::default()
                .fg(Color::White)
                .bg(Color::Blue)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Color::Gray)
        };

        let no_style = if !selected {
            Style::default()
                .fg(Color::White)
                .bg(Color::Blue)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Color::Gray)
        };

        let yes_label = if default { "Yes (default)" } else { "Yes" };
        let no_label = if !default { "No (default)" } else { "No" };

        lines.push(Line::from(vec![
            Span::styled(format!("  {}  ", yes_label), yes_style),
            Span::raw("  "),
            Span::styled(format!("  {}  ", no_label), no_style),
        ]));

        // Add help message if present
        if let Some(help_text) = help {
            lines.push(Line::from(""));
            lines.push(Line::from(Span::styled(
                help_text.clone(),
                Style::default().fg(Color::Yellow),
            )));
        }

        // Add keybindings hint
        lines.push(Line::from(""));
        lines.push(Line::from(Span::styled(
            "y/n: Select | ←/→: Toggle | Enter: Confirm | Esc: Cancel",
            Style::default().fg(Color::DarkGray),
        )));

        lines
    }
}

impl<'a> Widget for DialogWidget<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        // Generate content based on dialog type
        let lines = match &self.dialog.dialog_type {
            DialogType::Select {
                options,
                selected_index,
            } => Self::render_select(options, *selected_index, &self.dialog.help_message),

            DialogType::MultiSelect {
                options,
                selected_indices,
                cursor_index,
            } => Self::render_multiselect(
                options,
                selected_indices,
                *cursor_index,
                &self.dialog.help_message,
            ),

            DialogType::TextInput {
                prompt,
                input,
                cursor_pos,
                default,
            } => Self::render_text_input(
                prompt,
                input,
                *cursor_pos,
                default,
                &self.dialog.help_message,
            ),

            DialogType::Confirm {
                prompt,
                default,
                selected,
            } => Self::render_confirm(prompt, *default, *selected, &self.dialog.help_message),
        };

        // Create paragraph with border
        let paragraph = Paragraph::new(lines)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_type(BorderType::Rounded)
                    .title(format!(" {} ", self.dialog.title))
                    .title_alignment(Alignment::Center)
                    .style(Style::default().fg(Color::Cyan)),
            )
            .wrap(Wrap { trim: false });

        paragraph.render(area, buf);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cli::tui::dialog::DialogOption;

    #[test]
    fn test_widget_creation() {
        let dialog = Dialog::select(
            "Test",
            vec![
                DialogOption::new("Option 1"),
                DialogOption::new("Option 2"),
            ],
        );

        let widget = DialogWidget::new(&dialog);
        assert_eq!(widget.dialog.title, "Test");
    }

    #[test]
    fn test_select_render() {
        let options = vec![
            DialogOption::new("Option 1"),
            DialogOption::with_description("Option 2", "With description"),
        ];

        let lines = DialogWidget::render_select(&options, 0, &None);

        // Should have: 2 options + empty line + keybindings = 4 lines
        assert!(lines.len() >= 3);
    }

    #[test]
    fn test_multiselect_render() {
        use std::collections::HashSet;

        let options = vec![DialogOption::new("Option 1"), DialogOption::new("Option 2")];

        let mut selected = HashSet::new();
        selected.insert(0);

        let lines = DialogWidget::render_multiselect(&options, &selected, 0, &None);

        // Should have: 2 options + empty line + keybindings = 4 lines
        assert!(lines.len() >= 3);
    }

    #[test]
    fn test_text_input_render() {
        let lines = DialogWidget::render_text_input("Enter text", "hello", 3, &None, &None);

        // Should have: prompt + empty line + input + empty line + keybindings = 5 lines
        assert!(lines.len() >= 4);
    }

    #[test]
    fn test_confirm_render() {
        let lines = DialogWidget::render_confirm("Are you sure?", true, true, &None);

        // Should have: prompt + empty line + options + empty line + keybindings = 5 lines
        assert!(lines.len() >= 4);
    }
}
