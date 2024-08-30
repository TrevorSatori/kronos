use std::error::Error;
use std::io::stdout;

use crossterm::{
    event::{DisableMouseCapture, EnableMouseCapture},
    cursor::Show,
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use log::error;
use ratatui::{backend::CrosstermBackend, Terminal};

pub fn set_terminal() -> Result<Terminal<CrosstermBackend<std::io::Stdout>>, impl Error> {
    enable_raw_mode()?;
    let mut stdout = stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;

    let backend = CrosstermBackend::new(stdout);
    Terminal::new(backend)
}

pub fn reset_terminal(writer: &mut impl std::io::Write) {
    execute!(writer, LeaveAlternateScreen, DisableMouseCapture, Show).unwrap_or_else(|e| {
        error!("tried to execute(...) but couldn't :( {e}");
    });

    disable_raw_mode().unwrap_or_else(|e| {
        error!("tried to disable_raw_mode but couldn't :( {e}");
    });
}
