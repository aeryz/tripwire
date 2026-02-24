use ratatui::{
    buffer::Buffer,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style, Stylize},
    text::Text,
    widgets::{Block, BorderType, Clear, List, ListItem, Paragraph, StatefulWidget, Widget},
};

use crate::app::{App, Mode};

impl Widget for &App {
    /// Renders the user interface widgets.
    ///
    // This is where you add new widgets.
    // See the following resources:
    // - https://docs.rs/ratatui/latest/ratatui/widgets/index.html
    // - https://github.com/ratatui/ratatui/tree/master/examples
    fn render(self, area: Rect, buf: &mut Buffer) {
        // 1) Split the screen into Left + Right
        let cols = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(30), // left column
                Constraint::Percentage(70), // right big pane
            ])
            .split(area);

        let left = cols[0];
        let right = cols[1];

        // 2) Split the left column into Top + Bottom
        let left_rows = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Percentage(50), // left-top
                Constraint::Percentage(50), // left-bottom
            ])
            .split(left);

        // --- Left / Top pane ---
        let left_top_block = Block::bordered()
            .title("Left / Top")
            .title_alignment(Alignment::Center)
            .border_type(BorderType::Rounded);

        // --- Left / Bottom pane ---
        let left_bottom_block = Block::bordered()
            .title("Left / Bottom")
            .title_alignment(Alignment::Center)
            .border_type(BorderType::Rounded);

        let items: Vec<ListItem> = self
            .commands
            .iter()
            .map(|c| ListItem::new(c.title()))
            .collect();

        let list = List::new(items)
            .block(left_bottom_block)
            // style for unselected items
            .style(Style::default().fg(Color::White).bg(Color::Black))
            // style for the selected row
            .highlight_style(
                Style::default()
                    .fg(Color::Black)
                    .bg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            )
            // optional marker shown beside selected item
            .highlight_symbol("➤ ");

        if let Some(mapping) = &self.debugger_ctx.function_mapping {
            let functions: Vec<ListItem> = mapping
                .into_iter()
                .map(|(k, v)| ListItem::new(format!("{k}: {}", v.symbol)))
                .collect();
            let function_list = List::new(functions)
                .block(left_top_block)
                // style for unselected items
                .style(Style::default().fg(Color::White).bg(Color::Black))
                // style for the selected row
                .highlight_style(
                    Style::default()
                        .fg(Color::Black)
                        .bg(Color::Cyan)
                        .add_modifier(Modifier::BOLD),
                )
                // optional marker shown beside selected item
                .highlight_symbol("➤ ");
            StatefulWidget::render(
                function_list,
                left_rows[0],
                buf,
                &mut self.mapping_list_state.clone(),
            );
        } else {
            let left_top = Paragraph::new(format!("Counter",))
                .block(left_top_block)
                .fg(Color::Yellow)
                .bg(Color::Black)
                .centered();
            left_top.render(left_rows[0], buf);
        }

        // --- Right / Big pane ---
        let right_block = Block::bordered()
            .title("Right / Big")
            .title_alignment(Alignment::Center)
            .border_type(BorderType::Rounded);

        let right_pane = Paragraph::new(Text::styled(&self.disas_str, Style::default()))
            .block(right_block)
            .fg(Color::Cyan)
            .bg(Color::Black);

        StatefulWidget::render(list, left_rows[1], buf, &mut self.list_state.clone());

        // list.render(left_rows[1], buf);
        right_pane.render(right, buf);

        // Popup overlay
        if self.mode == Mode::StartProcessPopup {
            let popup_area = centered_rect(60, 25, area);

            // Clears underneath so the popup doesn't blend with background
            Clear.render(popup_area, buf);

            let popup_block = Block::bordered()
                .title("Attach")
                .title_alignment(Alignment::Center)
                .border_type(BorderType::Rounded);

            let text = format!(
                "Enter PID (or target):\n\n{}\n\n[Enter]=confirm  [Esc]=cancel",
                self.attach_input
            );

            Paragraph::new(text)
                .block(popup_block)
                .fg(Color::White)
                .bg(Color::Black)
                .render(popup_area, buf);

            // Optional: crude cursor hint by drawing a block at end of input line
            // (Real cursor placement is typically done via Terminal::set_cursor_position in draw loop.)
        }
    }
}

// Helper: centered rectangle by percentage
fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}
