use ratatui::{
    layout::{Constraint, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Cell, Gauge, Paragraph, Row, Table, TableState},
    Frame,
};

use crate::{
    hardware::{self, HardwareInfo},
    network::{self, NetworkInfo},
    system::{self, SystemInfo},
};

pub struct AppData {
    pub system: SystemInfo,
    pub hardware: HardwareInfo,
    pub network: NetworkInfo,
    pub table_state: TableState,
}

impl AppData {
    pub fn new() -> Self {
        let system = system::collect_info();
        let hardware = hardware::collect_info();
        let network = network::get_network_info();
        let mut table_state = TableState::default();
        if !network.interfaces.is_empty() {
            table_state.select(Some(0));
        }

        Self {
            system,
            hardware,
            network,
            table_state,
        }
    }

    pub fn refresh(&mut self) {
        self.system = system::collect_info();
        self.hardware = hardware::collect_info();
        self.network = network::get_network_info();

        let len = self.network.interfaces.len();
        if len == 0 {
            self.table_state.select(None);
        } else if let Some(selected) = self.table_state.selected() {
            if selected >= len {
                self.table_state.select(Some(len - 1));
            }
        } else {
            self.table_state.select(Some(0));
        }
    }

    pub fn select_next(&mut self) {
        let len = self.network.interfaces.len();
        if len == 0 {
            return;
        }
        let next = match self.table_state.selected() {
            Some(i) if i + 1 < len => i + 1,
            _ => 0,
        };
        self.table_state.select(Some(next));
    }

    pub fn select_prev(&mut self) {
        let len = self.network.interfaces.len();
        if len == 0 {
            return;
        }
        let prev = match self.table_state.selected() {
            Some(0) | None => len - 1,
            Some(i) => i - 1,
        };
        self.table_state.select(Some(prev));
    }
}

pub fn draw(frame: &mut Frame, app: &mut AppData) {
    let root = Layout::vertical([
        Constraint::Length(3),
        Constraint::Length(11),
        Constraint::Min(6),
        Constraint::Length(1),
    ])
    .split(frame.area());

    render_header(frame, root[0], app);

    let top_cols = Layout::horizontal([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(root[1]);
    render_system(frame, top_cols[0], &app.system);
    render_hardware(frame, top_cols[1], &app.hardware);

    render_network(frame, root[2], &app.network, &mut app.table_state);
    render_footer(frame, root[3]);
}

fn render_header(frame: &mut Frame, area: ratatui::layout::Rect, app: &AppData) {
    let hostname = app.system.hostname.as_deref().unwrap_or("unknown");
    let title = Line::from(vec![
        Span::styled(
            " RUSYS ",
            Style::default()
                .fg(Color::Black)
                .bg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw("  System Monitor  |  Host: "),
        Span::styled(
            hostname,
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw("  |  OS: "),
        Span::styled(app.system.os, Style::default().fg(Color::Green)),
    ]);

    let header = Paragraph::new(title).block(Block::bordered());
    frame.render_widget(header, area);
}

fn render_system(frame: &mut Frame, area: ratatui::layout::Rect, sys: &SystemInfo) {
    let lines = vec![
        kv_line("OS", sys.os),
        kv_line("Arch", sys.arch),
        kv_line("Family", sys.family),
        kv_line("Hostname", sys.hostname.as_deref().unwrap_or("unknown")),
        kv_line("OS Version", sys.os_version.as_deref().unwrap_or("unknown")),
        kv_line(
            "Kernel",
            sys.kernel_version.as_deref().unwrap_or("unknown"),
        ),
        kv_line("Distro", sys.distribution.as_str()),
        kv_line("Uptime", format_uptime(sys.uptime)),
    ];

    let widget = Paragraph::new(lines).block(
        Block::bordered()
            .title(" System ")
            .border_style(Style::default().fg(Color::Cyan)),
    );
    frame.render_widget(widget, area);
}

fn render_hardware(frame: &mut Frame, area: ratatui::layout::Rect, hw: &HardwareInfo) {
    let outer = Block::bordered()
        .title(" Hardware ")
        .border_style(Style::default().fg(Color::Magenta));
    let inner = outer.inner(area);
    frame.render_widget(outer, area);

    let chunks = Layout::vertical([Constraint::Length(5), Constraint::Length(4)]).split(inner);

    let cpu_lines = vec![
        kv_line("CPU", hw.cpu.name.as_str()),
        kv_line("Vendor", hw.cpu.manufacturer.as_str()),
        kv_line("Frequency", format!("{} MHz", hw.cpu.frequency)),
        kv_line("Cores", hw.cpu.cores.to_string()),
    ];
    frame.render_widget(Paragraph::new(cpu_lines), chunks[0]);

    let ratio = if hw.ram.total > 0 {
        (hw.ram.used as f64 / hw.ram.total as f64).clamp(0.0, 1.0)
    } else {
        0.0
    };
    let ram_label = format!(
        "{} / {} (Avail: {})",
        format_bytes(hw.ram.used),
        format_bytes(hw.ram.total),
        format_bytes(hw.ram.available)
    );
    let ram_gauge = Gauge::default()
        .block(Block::bordered().title(" RAM Usage "))
        .gauge_style(Style::default().fg(Color::Green).bg(Color::DarkGray))
        .ratio(ratio)
        .label(ram_label);
    frame.render_widget(ram_gauge, chunks[1]);
}

fn render_network(
    frame: &mut Frame,
    area: ratatui::layout::Rect,
    net: &NetworkInfo,
    table_state: &mut TableState,
) {
    let header = Row::new(["Interface", "MAC Address", "RX Total", "TX Total"])
        .style(
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        )
        .bottom_margin(1);

    let rows: Vec<Row> = net
        .interfaces
        .iter()
        .map(|iface| {
            Row::new(vec![
                Cell::from(iface.name.as_str()),
                Cell::from(iface.mac_address.as_str()),
                Cell::from(format_bytes(iface.received)),
                Cell::from(format_bytes(iface.transmitted)),
            ])
        })
        .collect();

    let widths = [
        Constraint::Percentage(25),
        Constraint::Percentage(35),
        Constraint::Percentage(20),
        Constraint::Percentage(20),
    ];

    let table = Table::new(rows, widths)
        .header(header)
        .block(
            Block::bordered()
                .title(" Network ")
                .border_style(Style::default().fg(Color::Blue)),
        )
        .row_highlight_style(
            Style::default()
                .bg(Color::DarkGray)
                .fg(Color::White)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol("▶ ");

    frame.render_stateful_widget(table, area, table_state);
}

fn render_footer(frame: &mut Frame, area: ratatui::layout::Rect) {
    let footer = Paragraph::new(Line::from(vec![
        Span::styled(
            " [q / Esc] ",
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw("Quit   "),
        Span::styled(
            "[r] ",
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw("Refresh   "),
        Span::styled(
            "[↑/↓ or j/k] ",
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw("Select Interface"),
    ]));
    frame.render_widget(footer, area);
}

fn kv_line(key: &str, value: impl Into<String>) -> Line<'static> {
    Line::from(vec![
        Span::styled(
            format!("{:<11}: ", key),
            Style::default()
                .fg(Color::Gray)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(value.into(), Style::default().fg(Color::White)),
    ])
}

fn format_bytes(bytes: u64) -> String {
    const KB: f64 = 1024.0;
    const MB: f64 = KB * 1024.0;
    const GB: f64 = MB * 1024.0;
    let b = bytes as f64;

    if b >= GB {
        format!("{:.2} GB", b / GB)
    } else if b >= MB {
        format!("{:.2} MB", b / MB)
    } else if b >= KB {
        format!("{:.2} KB", b / KB)
    } else {
        format!("{} B", bytes)
    }
}

fn format_uptime(seconds: u64) -> String {
    let days = seconds / 86_400;
    let hours = (seconds % 86_400) / 3_600;
    let minutes = (seconds % 3_600) / 60;
    let secs = seconds % 60;

    if days > 0 {
        format!("{}d {}h {}m {}s", days, hours, minutes, secs)
    } else if hours > 0 {
        format!("{}h {}m {}s", hours, minutes, secs)
    } else if minutes > 0 {
        format!("{}m {}s", minutes, secs)
    } else {
        format!("{}s", secs)
    }
}

#[cfg(test)]
mod tests {
    use super::{format_bytes, format_uptime};

    #[test]
    fn formats_bytes_units() {
        assert_eq!(format_bytes(512), "512 B");
        assert_eq!(format_bytes(1024), "1.00 KB");
        assert_eq!(format_bytes(1024 * 1024), "1.00 MB");
        assert_eq!(format_bytes(1024 * 1024 * 1024), "1.00 GB");
    }

    #[test]
    fn formats_uptime_durations() {
        assert_eq!(format_uptime(45), "45s");
        assert_eq!(format_uptime(125), "2m 5s");
        assert_eq!(format_uptime(3_661), "1h 1m 1s");
        assert_eq!(format_uptime(90_061), "1d 1h 1m 1s");
    }
}
