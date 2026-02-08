// Output Widget - Renders the scrollable output area
//
// Displays messages from OutputManager in the top section of the TUI
//
// NOTE: This widget is currently UNUSED in the TUI implementation.
// The current design uses Viewport::Inline(6) which renders only the bottom 6 lines
// (3 for input + 3 for status). All conversation output (user queries, Claude responses,
// tool output) is written directly to stdout above the TUI viewport and scrolls naturally
// in the terminal's scrollback buffer.
//
// This widget is fully implemented and ready for future use if we expand the viewport
// to include a TUI-native output area (e.g., Viewport::Inline(20) for split-screen mode).
// For now, the flush-through pattern (OutputManager buffering + periodic flush) provides
// better UX with full terminal scrollback and history persistence.

use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Widget, Wrap},
};

use crate::cli::{OutputManager, OutputMessage};

/// Widget for rendering the output area
pub struct OutputWidget<'a> {
    output_manager: &'a OutputManager,
    scroll_offset: usize,
}

impl<'a> OutputWidget<'a> {
    /// Create a new output widget
    pub fn new(output_manager: &'a OutputManager) -> Self {
        Self {
            output_manager,
            scroll_offset: 0,
        }
    }

    /// Create with a specific scroll offset (for Phase 4)
    #[allow(dead_code)]
    pub fn with_scroll_offset(output_manager: &'a OutputManager, scroll_offset: usize) -> Self {
        Self {
            output_manager,
            scroll_offset,
        }
    }

    /// Convert OutputMessage to styled Line
    fn message_to_line(message: &OutputMessage) -> Line<'static> {
        match message {
            OutputMessage::UserMessage { content } => {
                // User messages: bright white with "> " prefix
                Line::from(vec![
                    Span::styled(
                        "> ",
                        Style::default()
                            .fg(Color::Cyan)
                            .add_modifier(Modifier::BOLD),
                    ),
                    Span::styled(content.clone(), Style::default().fg(Color::White)),
                ])
            }
            OutputMessage::ClaudeResponse { content } => {
                // Claude responses: default color
                Line::from(Span::styled(
                    content.clone(),
                    Style::default().fg(Color::Gray),
                ))
            }
            OutputMessage::ToolOutput { tool_name, content } => {
                // Tool output: dark gray with tool name prefix
                Line::from(vec![
                    Span::styled(
                        format!("[{}] ", tool_name),
                        Style::default()
                            .fg(Color::Yellow)
                            .add_modifier(Modifier::BOLD),
                    ),
                    Span::styled(content.clone(), Style::default().fg(Color::DarkGray)),
                ])
            }
            OutputMessage::StatusInfo { content } => {
                // Status info: cyan
                Line::from(Span::styled(
                    content.clone(),
                    Style::default().fg(Color::Cyan),
                ))
            }
            OutputMessage::Error { content } => {
                // Errors: red
                Line::from(Span::styled(
                    content.clone(),
                    Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
                ))
            }
            OutputMessage::Progress { content } => {
                // Progress: yellow
                Line::from(Span::styled(
                    content.clone(),
                    Style::default().fg(Color::Yellow),
                ))
            }
            OutputMessage::SystemInfo { content } => {
                // System info: green with ℹ️ prefix
                Line::from(vec![
                    Span::styled(
                        "ℹ️  ",
                        Style::default()
                            .fg(Color::Green)
                            .add_modifier(Modifier::BOLD),
                    ),
                    Span::styled(content.clone(), Style::default().fg(Color::Green)),
                ])
            }
        }
    }
}

impl<'a> Widget for OutputWidget<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        // Get all messages from the buffer
        let messages = self.output_manager.get_messages();

        // Convert messages to styled lines
        let lines: Vec<Line> = messages.iter().map(Self::message_to_line).collect();

        // Create paragraph with wrapping
        let paragraph = Paragraph::new(lines)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(" Output ")
                    .style(Style::default().fg(Color::Gray)),
            )
            .wrap(Wrap { trim: false })
            .scroll((self.scroll_offset as u16, 0));

        paragraph.render(area, buf);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_message_to_line_user() {
        let msg = OutputMessage::UserMessage {
            content: "Hello".to_string(),
        };
        let line = OutputWidget::message_to_line(&msg);
        assert_eq!(line.spans.len(), 2); // "> " + content
    }

    #[test]
    fn test_message_to_line_claude() {
        let msg = OutputMessage::ClaudeResponse {
            content: "Response".to_string(),
        };
        let line = OutputWidget::message_to_line(&msg);
        assert_eq!(line.spans.len(), 1);
    }

    #[test]
    fn test_message_to_line_tool() {
        let msg = OutputMessage::ToolOutput {
            tool_name: "read".to_string(),
            content: "File contents".to_string(),
        };
        let line = OutputWidget::message_to_line(&msg);
        assert_eq!(line.spans.len(), 2); // "[tool] " + content
    }

    #[test]
    fn test_widget_creation() {
        let output_mgr = OutputManager::new();
        output_mgr.write_user("Test");

        let widget = OutputWidget::new(&output_mgr);
        assert_eq!(widget.scroll_offset, 0);
    }

    #[test]
    fn test_widget_with_scroll() {
        let output_mgr = OutputManager::new();
        let widget = OutputWidget::with_scroll_offset(&output_mgr, 10);
        assert_eq!(widget.scroll_offset, 10);
    }
}
