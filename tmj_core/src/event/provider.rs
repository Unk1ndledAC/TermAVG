use std::time::Duration;

use ratatui::crossterm::event::{self, Event, KeyCode, KeyEventKind, KeyModifiers};

use super::GameEvent;

pub trait EventProvider: Send {
    fn poll_event(&mut self) -> Option<GameEvent>;
}

pub struct CrosstermProvider {
    poll_timeout: Duration,
}

impl CrosstermProvider {
    pub fn new(poll_timeout: Duration) -> Self {
        Self { poll_timeout }
    }
}

impl EventProvider for CrosstermProvider {
    fn poll_event(&mut self) -> Option<GameEvent> {
        if event::poll(self.poll_timeout).ok()? {
            let ct = event::read().ok()?;
            Some(convert_crossterm_event(ct))
        } else {
            None
        }
    }
}

pub struct NoopProvider;

impl EventProvider for NoopProvider {
    fn poll_event(&mut self) -> Option<GameEvent> {
        None
    }
}

pub fn convert_crossterm_event(ct_event: Event) -> GameEvent {
    match &ct_event {
        Event::Key(key) => {
            if key.kind == KeyEventKind::Press {
                match key.code {
                    KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                        return GameEvent::QuitGame;
                    }
                    _ => return GameEvent::CtKeyEvent(*key),
                }
            } else {
                return GameEvent::CtKeyEvent(*key);
            }
        }
        Event::Resize(w, h) => GameEvent::ResizeTerm(*w, *h),
        Event::Mouse(mouse) => GameEvent::CtMouseEvent(*mouse),
        _ => GameEvent::CtUnDefined,
    }
}
