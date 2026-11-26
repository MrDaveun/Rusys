mod hardware;
mod network;
mod process;
mod system;
mod ui;

use std::time::{Duration, Instant};

use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use ui::AppData;

fn main() -> std::io::Result<()> {
    let mut app = AppData::new();
    let refresh_interval = Duration::from_secs(1);
    let mut last_tick = Instant::now();

    ratatui::run(|terminal| {
        loop {
            terminal.draw(|frame| ui::draw(frame, &mut app))?;

            let timeout = refresh_interval.saturating_sub(last_tick.elapsed());
            if event::poll(timeout)? {
                if let Event::Key(key) = event::read()? {
                    if key.kind == KeyEventKind::Press {
                        match key.code {
                            KeyCode::Char('q') | KeyCode::Esc => break,
                            KeyCode::Char('r') => {
                                app.refresh();
                                last_tick = Instant::now();
                            }
                            KeyCode::Char('s') => app.cycle_process_sort(),
                            KeyCode::Tab => app.toggle_panel(),
                            KeyCode::Down | KeyCode::Char('j') => app.select_next(),
                            KeyCode::Up | KeyCode::Char('k') => app.select_prev(),
                            _ => {}
                        }
                    }
                }
            }

            if last_tick.elapsed() >= refresh_interval {
                app.refresh();
                last_tick = Instant::now();
            }
        }
        Ok::<(), std::io::Error>(())
    })
}
