use ratatui::{
    layout::{Constraint, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Cell, Gauge, Paragraph, Row, Table, TableState},
    Frame,
};

use crate::{
    hardware::{HardwareCollector, HardwareInfo},
    network::{self, NetworkInfo},
    system::{self, SystemInfo},
};

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum ActivePanel {
    Disks,
    Network,
}

pub struct AppData {
    collector: HardwareCollector,
    pub system: SystemInfo,
    pub hardware: HardwareInfo,
    pub network: NetworkInfo,
    pub disk_table_state: TableState,
    pub network_table_state: TableState,
    pub active_panel: ActivePanel,
}

impl AppData {
    pub fn new() -> Self {
        let mut collector = HardwareCollector::new();
        let system = system::collect_info();
        let hardware = collector.collect();
        let network = network::get_network_info();

        let mut disk_table_state = TableState::default();
        if !hardware.disks.is_empty() {
            disk_table_state.select(Some(0));
        }

        let mut network_table_state = TableState::default();
        if !network.interfaces.is_empty() {
            network_table_state.select(Some(0));
        }

        Self {
            collector,
            system,
            hardware,
            network,
            disk_table_state,
            network_table_state,
            active_panel: ActivePanel::Disks,
        }
    }

    pub fn refresh(&mut self) {
        self.system = system::collect_info();
        self.hardware = self.collector.collect();
        self.network = network::get_network_info();

        clamp_table_state(&mut self.disk_table_state, self.hardware.disks.len());
        clamp_table_state(&mut self.network_table_state, self.network.interfaces.len());
    }

    pub fn toggle_panel(&mut self) {
        self.active_panel = match self.active_panel {
            ActivePanel::Disks => ActivePanel::Network,
            ActivePanel::Network => ActivePanel::Disks,
        };
    }

    pub fn select_next(&mut self) {
        match self.active_panel {
            ActivePanel::Disks => step_next(&mut self.disk_table_state, self.hardware.disks.len()),
            ActivePanel::Network => {
                step_next(&mut self.network_table_state, self.network.interfaces.len())
            }
        }
    }

    pub fn select_prev(&mut self) {
        match self.active_panel {
            ActivePanel::Disks => step_prev(&mut self.disk_table_state, self.hardware.disks.len()),
            ActivePanel::Network => {
                step_prev(&mut self.network_table_state, self.network.interfaces.len())
            }
        }
    }
}

fn clamp_table_state(state: &mut TableState, len: usize) {
    if len == 0 {
        state.select(None);
    } else if let Some(selected) = state.selected() {
        if selected >= len {
            state.select(Some(len - 1));
        }
    } else {
        state.select(Some(0));
    }
}

fn step_next(state: &mut TableState, len: usize) {
    if len == 0 {
        return;
    }
    let next = match state.selected() {
        Some(i) if i + 1 < len => i + 1,
        _ => 0,
    };
    state.select(Some(next));
}

fn step_prev(state: &mut TableState, len: usize) {
    if len == 0 {
        return;
    }
    let prev = match state.selected() {
        Some(0) | None => len - 1,
        Some(i) => i - 1,
    };
    state.select(Some(prev));
}

pub fn draw(frame: &mut Frame, app: &mut AppData) {
    let area = frame.area();
    let root = Layout::vertical([
        Constraint::Length(3),
        Constraint::Length(12),
        Constraint::Min(8),
        Constraint::Length(1),
    ])
    .split(area);

    render_header(frame, root[0], app);

    let top_cols = Layout::horizontal([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(root[1]);
    render_system(frame, top_cols[0], &app.system);
    render_hardware(frame, top_cols[1], &app.hardware);

    let bottom_areas = if area.width >= 100 {
        Layout::horizontal([Constraint::Percentage(50), Constraint::Percentage(50)]).split(root[2])
    } else {
        Layout::vertical([Constraint::Percentage(50), Constraint::Percentage(50)]).split(root[2])
    };

    render_disks(
        frame,
        bottom_areas[0],
        &app.hardware,
        &mut app.disk_table_state,
        app.active_panel == ActivePanel::Disks,
    );
    render_network(
        frame,
        bottom_areas[1],
        &app.network,
        &mut app.network_table_state,
        app.active_panel == ActivePanel::Network,
    );

    render_footer(frame, root[3]);
}

fn render_header(frame: &mut Frame, area: Rect, app: &AppData) {
    let hostname = app.system.hostname.as_deref().unwrap_or("unknown");
    let cpu_text = format!("{:.1}%", app.hardware.cpu.usage_percent);
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
        Span::raw("  |  CPU: "),
        Span::styled(
            cpu_text,
            Style::default()
                .fg(usage_color(app.hardware.cpu.usage_percent as f64 / 100.0))
                .add_modifier(Modifier::BOLD),
        ),
    ]);

    let header = Paragraph::new(title).block(Block::bordered());
    frame.render_widget(header, area);
}

fn render_system(frame: &mut Frame, area: Rect, sys: &SystemInfo) {
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

fn render_hardware(frame: &mut Frame, area: Rect, hw: &HardwareInfo) {
    let outer = Block::bordered()
        .title(" Hardware ")
        .border_style(Style::default().fg(Color::Magenta));
    let inner = outer.inner(area);
    frame.render_widget(outer, area);

    let chunks = Layout::vertical([
        Constraint::Length(4),
        Constraint::Length(3),
        Constraint::Length(3),
    ])
    .split(inner);

    let cpu_lines = vec![
        kv_line("CPU", hw.cpu.name.as_str()),
        kv_line("Vendor", hw.cpu.manufacturer.as_str()),
        kv_line("Frequency", format!("{} MHz", hw.cpu.frequency)),
        kv_line("Cores", hw.cpu.cores.to_string()),
    ];
    frame.render_widget(Paragraph::new(cpu_lines), chunks[0]);

    let cpu_ratio = (hw.cpu.usage_percent as f64 / 100.0).clamp(0.0, 1.0);
    let cpu_gauge = Gauge::default()
        .block(Block::bordered().title(" CPU Usage "))
        .gauge_style(
            Style::default()
                .fg(usage_color(cpu_ratio))
                .bg(Color::DarkGray),
        )
        .ratio(cpu_ratio)
        .label(format!("{:.1}%", hw.cpu.usage_percent));
    frame.render_widget(cpu_gauge, chunks[1]);

    let ram_ratio = usage_ratio(hw.ram.used, hw.ram.total);
    let ram_label = format!(
        "{} / {} (Avail: {})",
        format_bytes(hw.ram.used),
        format_bytes(hw.ram.total),
        format_bytes(hw.ram.available)
    );
    let ram_gauge = Gauge::default()
        .block(Block::bordered().title(" RAM Usage "))
        .gauge_style(
            Style::default()
                .fg(usage_color(ram_ratio))
                .bg(Color::DarkGray),
        )
        .ratio(ram_ratio)
        .label(ram_label);
    frame.render_widget(ram_gauge, chunks[2]);
}

fn render_disks(
    frame: &mut Frame,
    area: Rect,
    hw: &HardwareInfo,
    table_state: &mut TableState,
    is_active: bool,
) {
    let header = Row::new(["Mount", "FS", "Used / Total", "Avail", "Use"])
        .style(
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        )
        .bottom_margin(1);

    let rows: Vec<Row> = hw
        .disks
        .iter()
        .map(|disk| {
            let mount_label = if disk.mount_point.is_empty() {
                disk.name.as_str()
            } else {
                disk.mount_point.as_str()
            };
            let fs_label = if disk.file_system.is_empty() {
                "-"
            } else {
                disk.file_system.as_str()
            };
            let ratio = usage_ratio(disk.used, disk.total);

            Row::new(vec![
                Cell::from(mount_label.to_string()),
                Cell::from(fs_label.to_string()),
                Cell::from(format!(
                    "{} / {}",
                    format_bytes(disk.used),
                    format_bytes(disk.total)
                )),
                Cell::from(format_bytes(disk.available)),
                Cell::from(format!("{:.1}%", ratio * 100.0)),
            ])
        })
        .collect();

    let widths = [
        Constraint::Percentage(24),
        Constraint::Percentage(14),
        Constraint::Percentage(30),
        Constraint::Percentage(18),
        Constraint::Percentage(14),
    ];

    let title = if is_active {
        " Storage [Active] "
    } else {
        " Storage "
    };
    let border_color = if is_active {
        Color::Yellow
    } else {
        Color::DarkGray
    };

    let table = Table::new(rows, widths)
        .header(header)
        .block(
            Block::bordered()
                .title(title)
                .border_style(Style::default().fg(border_color)),
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

fn render_network(
    frame: &mut Frame,
    area: Rect,
    net: &NetworkInfo,
    table_state: &mut TableState,
    is_active: bool,
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

    let title = if is_active {
        " Network [Active] "
    } else {
        " Network "
    };
    let border_color = if is_active {
        Color::Blue
    } else {
        Color::DarkGray
    };

    let table = Table::new(rows, widths)
        .header(header)
        .block(
            Block::bordered()
                .title(title)
                .border_style(Style::default().fg(border_color)),
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

fn render_footer(frame: &mut Frame, area: Rect) {
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
            "[Tab] ",
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw("Switch Panel   "),
        Span::styled(
            "[↑/↓ or j/k] ",
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw("Select Row"),
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

fn usage_ratio(used: u64, total: u64) -> f64 {
    if total > 0 {
        (used as f64 / total as f64).clamp(0.0, 1.0)
    } else {
        0.0
    }
}

fn usage_color(ratio: f64) -> Color {
    if ratio >= 0.85 {
        Color::Red
    } else if ratio >= 0.65 {
        Color::Yellow
    } else {
        Color::Green
    }
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
    use super::{format_bytes, format_uptime, usage_ratio};

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

    #[test]
    fn computes_usage_ratio_safely() {
        assert_eq!(usage_ratio(0, 0), 0.0);
        assert_eq!(usage_ratio(50, 100), 0.5);
        assert_eq!(usage_ratio(150, 100), 1.0);
    }
}
