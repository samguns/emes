use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Gauge, List, ListItem, Paragraph},
    Frame,
};
use std::time::Duration;

use crate::playlist::Track;

pub struct UI {
    current_track: Option<Track>,
    is_playing: bool,
    position: Duration,
    duration: Option<Duration>,
    volume: f32,
    playlist: Vec<Track>,
    current_index: Option<usize>,
    selected_index: usize,
    show_help: bool,
}

impl UI {
    pub fn new() -> Self {
        Self {
            current_track: None,
            is_playing: false,
            position: Duration::from_secs(0),
            duration: None,
            volume: 0.5,
            playlist: Vec::new(),
            current_index: None,
            selected_index: 0,
            show_help: false,
        }
    }

    pub fn update_player_state(
        &mut self,
        is_playing: bool,
        position: Duration,
        duration: Option<Duration>,
        volume: f32,
    ) {
        self.is_playing = is_playing;
        self.position = position;
        self.duration = duration;
        self.volume = volume;
    }

    pub fn update_playlist(&mut self, playlist: &[Track], current_index: Option<usize>) {
        self.playlist = playlist.to_vec();
        self.current_index = current_index;
    }

    pub fn update_current_track(&mut self, track: Option<Track>) {
        self.current_track = track;
    }

    pub fn toggle_help(&mut self) {
        self.show_help = !self.show_help;
    }

    pub fn draw(&self, frame: &mut Frame) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .margin(1)
            .constraints([
                Constraint::Length(3), // Title
                Constraint::Length(5), // Now Playing
                Constraint::Length(3), // Progress
                Constraint::Min(5),    // Playlist
                Constraint::Length(3), // Controls
            ])
            .split(frame.area());

        self.draw_title(frame, chunks[0]);
        self.draw_now_playing(frame, chunks[1]);
        self.draw_progress(frame, chunks[2]);
        self.draw_playlist(frame, chunks[3]);
        self.draw_controls(frame, chunks[4]);

        if self.show_help {
            self.draw_help(frame);
        }
    }

    fn draw_title(&self, frame: &mut Frame, area: Rect) {
        let title = Paragraph::new("🎵 MP3 Player")
            .style(
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            )
            .alignment(Alignment::Center)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_type(BorderType::Rounded)
                    .border_style(Style::default().fg(Color::Cyan)),
            );
        frame.render_widget(title, area);
    }

    fn draw_now_playing(&self, frame: &mut Frame, area: Rect) {
        let mut lines = vec![];

        if let Some(ref track) = self.current_track {
            lines.push(Line::from(vec![
                Span::styled("Now Playing: ", Style::default().fg(Color::Yellow)),
                Span::styled(
                    &track.name,
                    Style::default()
                        .fg(Color::White)
                        .add_modifier(Modifier::BOLD),
                ),
            ]));

            if let Some(ref artist) = track.artist {
                lines.push(Line::from(vec![
                    Span::styled("Artist: ", Style::default().fg(Color::Yellow)),
                    Span::raw(artist),
                ]));
            }

            if let Some(ref album) = track.album {
                lines.push(Line::from(vec![
                    Span::styled("Album: ", Style::default().fg(Color::Yellow)),
                    Span::raw(album),
                ]));
            }

            let status = if self.is_playing {
                "▶ Playing"
            } else {
                "⏸ Paused"
            };
            let status_color = if self.is_playing {
                Color::Green
            } else {
                Color::Yellow
            };
            lines.push(Line::from(Span::styled(
                status,
                Style::default().fg(status_color),
            )));
        } else {
            lines.push(Line::from(Span::styled(
                "No track loaded",
                Style::default().fg(Color::DarkGray),
            )));
        }

        let now_playing = Paragraph::new(lines).block(
            Block::default()
                .title("Now Playing")
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded),
        );
        frame.render_widget(now_playing, area);
    }

    fn draw_progress(&self, frame: &mut Frame, area: Rect) {
        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Length(8),  // Time
                Constraint::Min(10),    // Progress bar
                Constraint::Length(8),  // Duration
                Constraint::Length(15), // Volume
            ])
            .split(area);

        // Time display
        let time = format!(
            "{:02}:{:02}",
            self.position.as_secs() / 60,
            self.position.as_secs() % 60
        );
        let time_widget = Paragraph::new(time)
            .alignment(Alignment::Center)
            .block(Block::default().borders(Borders::LEFT | Borders::TOP | Borders::BOTTOM));
        frame.render_widget(time_widget, chunks[0]);

        // Progress bar
        let progress = if let Some(duration) = self.duration {
            if duration.as_secs() > 0 {
                (self.position.as_secs() as f64 / duration.as_secs() as f64).min(1.0)
            } else {
                0.0
            }
        } else {
            0.0
        };

        let progress_bar = Gauge::default()
            .block(Block::default().borders(Borders::TOP | Borders::BOTTOM))
            .gauge_style(Style::default().fg(Color::Cyan))
            .ratio(progress);
        frame.render_widget(progress_bar, chunks[1]);

        // Duration display
        let duration = if let Some(duration) = self.duration {
            format!(
                "{:02}:{:02}",
                duration.as_secs() / 60,
                duration.as_secs() % 60
            )
        } else {
            "--:--".to_string()
        };
        let duration_widget = Paragraph::new(duration)
            .alignment(Alignment::Center)
            .block(Block::default().borders(Borders::RIGHT | Borders::TOP | Borders::BOTTOM));
        frame.render_widget(duration_widget, chunks[2]);

        // Volume
        let volume_text = format!("Vol: {}%", (self.volume * 100.0) as u32);
        let volume_widget = Paragraph::new(volume_text)
            .alignment(Alignment::Center)
            .block(Block::default().borders(Borders::ALL));
        frame.render_widget(volume_widget, chunks[3]);
    }

    fn draw_playlist(&self, frame: &mut Frame, area: Rect) {
        let items: Vec<ListItem> = self
            .playlist
            .iter()
            .enumerate()
            .map(|(i, track)| {
                let prefix = if Some(i) == self.current_index {
                    "▶ "
                } else {
                    "  "
                };

                let style = if Some(i) == self.current_index {
                    Style::default()
                        .fg(Color::Green)
                        .add_modifier(Modifier::BOLD)
                } else {
                    Style::default()
                };

                ListItem::new(format!("{}{}", prefix, track.name)).style(style)
            })
            .collect();

        let playlist = List::new(items)
            .block(
                Block::default()
                    .title(format!("Playlist ({} tracks)", self.playlist.len()))
                    .borders(Borders::ALL)
                    .border_type(BorderType::Rounded),
            )
            .highlight_style(
                Style::default()
                    .bg(Color::DarkGray)
                    .add_modifier(Modifier::BOLD),
            )
            .highlight_symbol("> ");

        frame.render_widget(playlist, area);
    }

    fn draw_controls(&self, frame: &mut Frame, area: Rect) {
        let controls = vec![
            ("Space", "Play/Pause"),
            ("←/→", "Prev/Next"),
            ("↑/↓", "Select"),
            ("Enter", "Play Selected"),
            ("+/-", "Volume"),
            ("s", "Stop"),
            ("r", "Repeat"),
            ("h", "Help"),
            ("q", "Quit"),
        ];

        let control_text: Vec<Span> = controls
            .iter()
            .flat_map(|(key, action)| {
                vec![
                    Span::styled(format!("{}: ", key), Style::default().fg(Color::Yellow)),
                    Span::raw(format!("{}  ", action)),
                ]
            })
            .collect();

        let controls_widget = Paragraph::new(Line::from(control_text))
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_type(BorderType::Rounded),
            )
            .alignment(Alignment::Center);

        frame.render_widget(controls_widget, area);
    }

    fn draw_help(&self, frame: &mut Frame) {
        let help_text = vec![
            Line::from(""),
            Line::from(vec![Span::styled(
                "Keyboard Shortcuts",
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            )]),
            Line::from(""),
            Line::from(vec![
                Span::styled("Space      ", Style::default().fg(Color::Cyan)),
                Span::raw("Toggle play/pause"),
            ]),
            Line::from(vec![
                Span::styled("Enter      ", Style::default().fg(Color::Cyan)),
                Span::raw("Play selected track"),
            ]),
            Line::from(vec![
                Span::styled("← / →      ", Style::default().fg(Color::Cyan)),
                Span::raw("Previous/Next track"),
            ]),
            Line::from(vec![
                Span::styled("↑ / ↓      ", Style::default().fg(Color::Cyan)),
                Span::raw("Move selection up/down"),
            ]),
            Line::from(vec![
                Span::styled("+ / -      ", Style::default().fg(Color::Cyan)),
                Span::raw("Increase/Decrease volume"),
            ]),
            Line::from(vec![
                Span::styled("s          ", Style::default().fg(Color::Cyan)),
                Span::raw("Stop playback"),
            ]),
            Line::from(vec![
                Span::styled("r          ", Style::default().fg(Color::Cyan)),
                Span::raw("Toggle repeat mode"),
            ]),
            Line::from(vec![
                Span::styled("h / ?      ", Style::default().fg(Color::Cyan)),
                Span::raw("Show/Hide this help"),
            ]),
            Line::from(vec![
                Span::styled("q / Ctrl+C ", Style::default().fg(Color::Cyan)),
                Span::raw("Quit application"),
            ]),
            Line::from(""),
            Line::from(vec![Span::styled(
                "Press any key to close this help",
                Style::default().fg(Color::DarkGray),
            )]),
        ];

        let help = Paragraph::new(help_text)
            .block(
                Block::default()
                    .title("Help")
                    .borders(Borders::ALL)
                    .border_type(BorderType::Double)
                    .border_style(Style::default().fg(Color::Yellow)),
            )
            .alignment(Alignment::Left);

        let area = centered_rect(60, 60, frame.area());
        frame.render_widget(help, area);
    }
}

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
