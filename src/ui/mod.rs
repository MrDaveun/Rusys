use ratatui::{
    layout::{Constraint, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Cell, Gauge, Paragraph, Row, Table, TableState},
    Frame,
};

use crate::{
    hardware::{HardwareCollector, HardwareInfo},
    network::{NetworkCollector, NetworkInfo},
    process::{self, ProcessCollector, ProcessListInfo, ProcessSort},
    system::{self, SystemInfo},
};

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum ActivePanel {
    Processes,
    CpuCores,
    Disks,
    Network,
}

pub struct AppData {
    hw_collector: HardwareCollector,
    net_collector: NetworkCollector,
    proc_collector: ProcessCollector,
    pub system: SystemInfo,
    pub hardware: HardwareInfo,
    pub network: NetworkInfo,
    pub processes: ProcessListInfo,
    pub process_sort: ProcessSort,
    pub cpu_table_state: TableState,
    pub process_table_state: TableState,
    pub disk_table_state: TableState,
    pub network_table_state: TableState,
    pub active_panel: ActivePanel,
}

impl AppData {
    pub fn new() -> Self {
        let mut proc_collector = ProcessCollector::new();
        let mut hw_collector = HardwareCollector::new();
        let mut net_collector = NetworkCollector::new();
        let process_sort = ProcessSort::Cpu;

        let system = system::collect_info();
        let hardware = hw_collector.collect();
        let network = net_collector.collect();
        let processes = proc_collector.collect(process_sort);

        let mut cpu_table_state = TableState::default();
        if !hardware.cpu.cores_usage.is_empty() {
            cpu_table_state.select(Some(0));
        }

        let mut process_table_state = TableState::default();
        if !processes.processes.is_empty() {
            process_table_state.select(Some(0));
        }

        let mut disk_table_state = TableState::default();
        if !hardware.disks.is_empty() {
            disk_table_state.select(Some(0));
        }

        let mut network_table_state = TableState::default();
        if !network.interfaces.is_empty() {
            network_table_state.select(Some(0));
        }

        Self {
            hw_collector,
            net_collector,
            proc_collector,
            system,
            hardware,
            network,
            processes,
            process_sort,
            cpu_table_state,
            process_table_state,
            disk_table_state,
            network_table_state,
            active_panel: ActivePanel::Processes,
        }
    }

    pub fn refresh(&mut self) {
        self.system = system::collect_info();
        self.hardware = self.hw_collector.collect();
        self.network = self.net_collector.collect();
        self.processes = self.proc_collector.collect(self.process_sort);

        clamp_table_state(&mut self.cpu_table_state, self.hardware.cpu.cores_usage.len());
        clamp_table_state(&mut self.process_table_state, self.processes.processes.len());
        clamp_table_state(&mut self.disk_table_state, self.hardware.disks.len());
        clamp_table_state(&mut self.network_table_state, self.network.interfaces.len());
    }

    pub fn cycle_process_sort(&mut self) {
        self.process_sort = self.process_sort.next();
        process::sort_processes(&mut self.processes.processes, self.process_sort);
        clamp_table_state(&mut self.process_table_state, self.processes.processes.len());
    }

    pub fn toggle_panel(&mut self) {
        self.active_panel = match self.active_panel {
            ActivePanel::Processes => ActivePanel::CpuCores,
            ActivePanel::CpuCores => ActivePanel::Disks,
            ActivePanel::Disks => ActivePanel::Network,
            ActivePanel::Network => ActivePanel::Processes,
        };
    }

    pub fn select_next(&mut self) {
        match self.active_panel {
            ActivePanel::Processes => {
                step_next(&mut self.process_table_state, self.processes.processes.len())
            }
            ActivePanel::CpuCores => {
                step_next(&mut self.cpu_table_state, self.hardware.cpu.cores_usage.len())
            }
            ActivePanel::Disks => step_next(&mut self.disk_table_state, self.hardware.disks.len()),
            ActivePanel::Network => {
                step_next(&mut self.network_table_state, self.network.interfaces.len())
            }
        }
    }

    pub fn select_prev(&mut self) {
        match self.active_panel {
            ActivePanel::Processes => {
                step_prev(&mut self.process_table_state, self.processes.processes.len())
            }
            ActivePanel::CpuCores => {
                step_prev(&mut self.cpu_table_state, self.hardware.cpu.cores_usage.len())
            }
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
        Constraint::Length(13),
        Constraint::Min(10),
        Constraint::Length(1),
    ])
    .split(area);

    render_header(frame, root[0], app);

    if area.width >= 100 {
        let top_cols = Layout::horizontal([
            Constraint::Percentage(28),
            Constraint::Percentage(42),
            Constraint::Percentage(30),
        ])
        .split(root[1]);

        render_system(frame, top_cols[0], &app.system, app.processes.total_count);
        render_hardware(frame, top_cols[1], &app.hardware);
        render_cpu_cores(
            frame,
            top_cols[2],
            &app.hardware,
            &mut app.cpu_table_state,
            app.active_panel == ActivePanel::CpuCores,
        );

        let bottom_cols =
            Layout::horizontal([Constraint::Percentage(50), Constraint::Percentage(50)])
                .split(root[2]);
        let right_stack =
            Layout::vertical([Constraint::Percentage(46), Constraint::Percentage(54)])
                .split(bottom_cols[1]);

        render_processes(
            frame,
            bottom_cols[0],
            &app.processes,
            app.process_sort,
            &mut app.process_table_state,
            app.active_panel == ActivePanel::Processes,
        );
        render_disks(
            frame,
            right_stack[0],
            &app.hardware,
            &mut app.disk_table_state,
            app.active_panel == ActivePanel::Disks,
        );
        render_network(
            frame,
            right_stack[1],
            &app.network,
            &mut app.network_table_state,
            app.active_panel == ActivePanel::Network,
        );
    } else {
        let top_cols = Layout::horizontal([Constraint::Percentage(45), Constraint::Percentage(55)])
            .split(root[1]);
        render_system(frame, top_cols[0], &app.system, app.processes.total_count);
        render_hardware(frame, top_cols[1], &app.hardware);

        let bottom_stack = Layout::vertical([
            Constraint::Percentage(24),
            Constraint::Percentage(34),
            Constraint::Percentage(21),
            Constraint::Percentage(21),
        ])
        .split(root[2]);

        render_cpu_cores(
            frame,
            bottom_stack[0],
            &app.hardware,
            &mut app.cpu_table_state,
            app.active_panel == ActivePanel::CpuCores,
        );
        render_processes(
            frame,
            bottom_stack[1],
            &app.processes,
            app.process_sort,
            &mut app.process_table_state,
            app.active_panel == ActivePanel::Processes,
        );
        render_disks(
            frame,
            bottom_stack[2],
            &app.hardware,
            &mut app.disk_table_state,
            app.active_panel == ActivePanel::Disks,
        );
        render_network(
            frame,
            bottom_stack[3],
            &app.network,
            &mut app.network_table_state,
            app.active_panel == ActivePanel::Network,
        );
    }

    render_footer(frame, root[3]);
}

fn render_header(frame: &mut Frame, area: Rect, app: &AppData) {
    let hostname = app.system.hostname.as_deref().unwrap_or("unknown");
    let cpu_text = format!("{:.1}%", app.hardware.cpu.usage_percent);
    let swap_text = if app.hardware.swap.total > 0 {
        format!(
            "{}/{}",
            format_bytes(app.hardware.swap.used),
            format_bytes(app.hardware.swap.total)
        )
    } else {
        "off".to_string()
    };

    let title = Line::from(vec![
        Span::styled(
            " RUSYS ",
            Style::default()
                .fg(Color::Black)
                .bg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw("  Host: "),
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
        Span::raw("  |  Swap: "),
        Span::styled(swap_text, Style::default().fg(Color::Magenta)),
        Span::raw("  |  Procs: "),
        Span::styled(
            app.processes.total_count.to_string(),
            Style::default().fg(Color::White),
        ),
    ]);

    let header = Paragraph::new(title).block(Block::bordered());
    frame.render_widget(header, area);
}

fn render_system(frame: &mut Frame, area: Rect, sys: &SystemInfo, process_count: usize) {
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
        kv_line("Processes", process_count.to_string()),
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
        Constraint::Length(2),
        Constraint::Length(3),
        Constraint::Length(3),
        Constraint::Length(3),
    ])
    .split(inner);

    let cpu_lines = vec![
        kv_line("CPU", hw.cpu.name.as_str()),
        kv_line(
            "Details",
            format!(
                "{} | {} MHz | {} cores",
                hw.cpu.manufacturer, hw.cpu.frequency, hw.cpu.cores
            ),
        ),
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

    let swap_ratio = usage_ratio(hw.swap.used, hw.swap.total);
    let swap_label = if hw.swap.total > 0 {
        format!(
            "{} / {} (Free: {})",
            format_bytes(hw.swap.used),
            format_bytes(hw.swap.total),
            format_bytes(hw.swap.free)
        )
    } else {
        "Not configured (0 B)".to_string()
    };
    let swap_gauge = Gauge::default()
        .block(Block::bordered().title(" Swap Usage "))
        .gauge_style(
            Style::default()
                .fg(usage_color(swap_ratio))
                .bg(Color::DarkGray),
        )
        .ratio(swap_ratio)
        .label(swap_label);
    frame.render_widget(swap_gauge, chunks[3]);
}

fn render_cpu_cores(
    frame: &mut Frame,
    area: Rect,
    hw: &HardwareInfo,
    table_state: &mut TableState,
    is_active: bool,
) {
    let header = Row::new(["Core", "Freq", "Use", "Load"])
        .style(
            Style::default()
                .fg(Color::Green)
                .add_modifier(Modifier::BOLD),
        )
        .bottom_margin(1);

    let rows: Vec<Row> = hw
        .cpu
        .cores_usage
        .iter()
        .map(|core| {
            let ratio = (core.usage_percent as f64 / 100.0).clamp(0.0, 1.0);
            let bar = usage_bar(ratio, 8);

            Row::new(vec![
                Cell::from(core.name.as_str()),
                Cell::from(format!("{}MHz", core.frequency)),
                Cell::from(format!("{:.1}%", core.usage_percent))
                    .style(Style::default().fg(usage_color(ratio))),
                Cell::from(bar).style(Style::default().fg(usage_color(ratio))),
            ])
        })
        .collect();

    let widths = [
        Constraint::Percentage(24),
        Constraint::Percentage(26),
        Constraint::Percentage(22),
        Constraint::Percentage(28),
    ];

    let title = if is_active {
        " CPU Cores [Active] "
    } else {
        " CPU Cores "
    };
    let border_color = if is_active {
        Color::Green
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

fn render_processes(
    frame: &mut Frame,
    area: Rect,
    procs: &ProcessListInfo,
    sort: ProcessSort,
    table_state: &mut TableState,
    is_active: bool,
) {
    let title = if is_active {
        format!(" Processes ({}) [Sort: {}] [Active] ", procs.total_count, sort.label())
    } else {
        format!(" Processes ({}) [Sort: {}] ", procs.total_count, sort.label())
    };
    let border_color = if is_active {
        Color::Cyan
    } else {
        Color::DarkGray
    };

    let outer = Block::bordered()
        .title(title)
        .border_style(Style::default().fg(border_color));
    let inner = outer.inner(area);
    frame.render_widget(outer, area);

    let chunks = Layout::vertical([Constraint::Min(3), Constraint::Length(1)]).split(inner);

    let header = Row::new(["PID", "Name", "CPU%", "Memory", "State"])
        .style(
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )
        .bottom_margin(1);

    let rows: Vec<Row> = procs
        .processes
        .iter()
        .map(|proc_item| {
            let cpu_ratio = (proc_item.cpu_usage as f64 / 100.0).clamp(0.0, 1.0);
            Row::new(vec![
                Cell::from(proc_item.pid.to_string()),
                Cell::from(proc_item.name.as_str()),
                Cell::from(format!("{:.1}%", proc_item.cpu_usage))
                    .style(Style::default().fg(usage_color(cpu_ratio))),
                Cell::from(format_bytes(proc_item.memory)),
                Cell::from(proc_item.status.as_str()),
            ])
        })
        .collect();

    let widths = [
        Constraint::Length(8),
        Constraint::Percentage(36),
        Constraint::Length(8),
        Constraint::Length(11),
        Constraint::Min(8),
    ];

    let table = Table::new(rows, widths)
        .header(header)
        .row_highlight_style(
            Style::default()
                .bg(Color::DarkGray)
                .fg(Color::White)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol("▶ ");

    frame.render_stateful_widget(table, chunks[0], table_state);

    let detail = if let Some(selected) = table_state
        .selected()
        .and_then(|idx| procs.processes.get(idx))
    {
        format!(
            "Selected: {} (PID {}) | Virt: {} | State: {}",
            selected.name,
            selected.pid,
            format_bytes(selected.virtual_memory),
            selected.status
        )
    } else {
        "No process selected".to_string()
    };

    frame.render_widget(
        Paragraph::new(Line::from(detail)).style(Style::default().fg(Color::Gray)),
        chunks[1],
    );
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

    let outer = Block::bordered()
        .title(title)
        .border_style(Style::default().fg(border_color));
    let inner = outer.inner(area);
    frame.render_widget(outer, area);

    let chunks = Layout::vertical([Constraint::Min(3), Constraint::Length(2)]).split(inner);

    let header = Row::new(["Interface", "Down/s", "Up/s", "RX Total", "TX Total"])
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
                Cell::from(format_speed(iface.download_speed)),
                Cell::from(format_speed(iface.upload_speed)),
                Cell::from(format_bytes(iface.total_received)),
                Cell::from(format_bytes(iface.total_transmitted)),
            ])
        })
        .collect();

    let widths = [
        Constraint::Percentage(22),
        Constraint::Percentage(19),
        Constraint::Percentage(19),
        Constraint::Percentage(20),
        Constraint::Percentage(20),
    ];

    let table = Table::new(rows, widths)
        .header(header)
        .row_highlight_style(
            Style::default()
                .bg(Color::DarkGray)
                .fg(Color::White)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol("▶ ");

    frame.render_stateful_widget(table, chunks[0], table_state);

    let detail_lines = if let Some(iface) = table_state
        .selected()
        .and_then(|idx| net.interfaces.get(idx))
    {
        let ips = if iface.ip_addresses.is_empty() {
            "none".to_string()
        } else {
            iface.ip_addresses.join(", ")
        };

        vec![
            Line::from(format!(
                "MAC: {} | MTU: {} | IPs: {}",
                iface.mac_address, iface.mtu, ips
            )),
            Line::from(format!(
                "Pkts: ↓{} ↑{} | Err: ↓{} ↑{} | Delta: ↓{} ↑{}",
                iface.packets_received,
                iface.packets_transmitted,
                iface.errors_received,
                iface.errors_transmitted,
                format_bytes(iface.received),
                format_bytes(iface.transmitted),
            )),
        ]
    } else {
        vec![Line::from("No network interface selected")]
    };

    frame.render_widget(
        Paragraph::new(detail_lines).style(Style::default().fg(Color::Gray)),
        chunks[1],
    );
}

fn render_footer(frame: &mut Frame, area: Rect) {
    let footer = Paragraph::new(Line::from(vec![
        Span::styled(
            " [q / Esc] ",
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw("Quit  "),
        Span::styled(
            "[r] ",
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw("Refresh  "),
        Span::styled(
            "[s] ",
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw("Sort Procs  "),
        Span::styled(
            "[Tab] ",
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw("Switch Panel  "),
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

fn usage_bar(ratio: f64, width: usize) -> String {
    let clamped = ratio.clamp(0.0, 1.0);
    let filled = (clamped * width as f64).round() as usize;
    let empty = width.saturating_sub(filled);
    format!("{}{}", "█".repeat(filled), "░".repeat(empty))
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

fn format_speed(bytes_per_sec: f64) -> String {
    const KB: f64 = 1024.0;
    const MB: f64 = KB * 1024.0;
    const GB: f64 = MB * 1024.0;

    if bytes_per_sec >= GB {
        format!("{:.2} GB/s", bytes_per_sec / GB)
    } else if bytes_per_sec >= MB {
        format!("{:.2} MB/s", bytes_per_sec / MB)
    } else if bytes_per_sec >= KB {
        format!("{:.1} KB/s", bytes_per_sec / KB)
    } else {
        format!("{:.0} B/s", bytes_per_sec)
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
    use super::{format_bytes, format_speed, format_uptime, usage_bar, usage_ratio};

    #[test]
    fn formats_bytes_units() {
        assert_eq!(format_bytes(512), "512 B");
        assert_eq!(format_bytes(1024), "1.00 KB");
        assert_eq!(format_bytes(1024 * 1024), "1.00 MB");
        assert_eq!(format_bytes(1024 * 1024 * 1024), "1.00 GB");
    }

    #[test]
    fn formats_speed_units() {
        assert_eq!(format_speed(500.0), "500 B/s");
        assert_eq!(format_speed(2048.0), "2.0 KB/s");
        assert_eq!(format_speed(1024.0 * 1024.0), "1.00 MB/s");
        assert_eq!(format_speed(1024.0 * 1024.0 * 1024.0), "1.00 GB/s");
    }

    #[test]
    fn formats_uptime_durations() {
        assert_eq!(format_uptime(45), "45s");
        assert_eq!(format_uptime(125), "2m 5s");
        assert_eq!(format_uptime(3_661), "1h 1m 1s");
        assert_eq!(format_uptime(90_061), "1d 1h 1m 1s");
    }

    #[test]
    fn computes_usage_ratio_and_bar_safely() {
        assert_eq!(usage_ratio(0, 0), 0.0);
        assert_eq!(usage_ratio(50, 100), 0.5);
        assert_eq!(usage_ratio(150, 100), 1.0);
        assert_eq!(usage_bar(0.0, 4), "░░░░");
        assert_eq!(usage_bar(0.5, 4), "██░░");
        assert_eq!(usage_bar(1.0, 4), "████");
    }
}
