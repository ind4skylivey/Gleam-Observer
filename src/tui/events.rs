use crossterm::event::{self, Event, KeyCode, KeyEvent};
use std::time::Duration;

pub enum AppEvent {
    Tick,
    Key(KeyEvent),
    Resize(u16, u16),
    Ignored, // For mouse events - don't update metrics
}

pub struct EventHandler {
    tick_rate: Duration,
}

impl EventHandler {
    pub fn new(tick_rate: Duration) -> Self {
        Self { tick_rate }
    }

    pub fn next(&self) -> crate::error::Result<AppEvent> {
        if event::poll(self.tick_rate)? {
            match event::read()? {
                Event::Key(key) => Ok(AppEvent::Key(key)),
                Event::Resize(w, h) => Ok(AppEvent::Resize(w, h)),
                Event::Mouse(_) => Ok(AppEvent::Ignored), // Ignore mouse - don't update
                _ => Ok(AppEvent::Tick),
            }
        } else {
            Ok(AppEvent::Tick)
        }
    }
}

pub fn should_quit(key: &KeyEvent) -> bool {
    matches!(key.code, KeyCode::Char('q') | KeyCode::Char('Q') | KeyCode::Esc)
}

pub fn should_pause(key: &KeyEvent) -> bool {
    matches!(key.code, KeyCode::Char('p') | KeyCode::Char('P') | KeyCode::Char(' '))
}

pub fn should_toggle_view(key: &KeyEvent) -> bool {
    matches!(key.code, KeyCode::Tab)
}

pub fn should_cycle_sort(key: &KeyEvent) -> bool {
    matches!(key.code, KeyCode::Char('s') | KeyCode::Char('S'))
}

pub fn is_arrow_up(key: &KeyEvent) -> bool {
    matches!(key.code, KeyCode::Up)
}

pub fn is_arrow_down(key: &KeyEvent) -> bool {
    matches!(key.code, KeyCode::Down)
}

pub fn is_enter(key: &KeyEvent) -> bool {
    matches!(key.code, KeyCode::Enter)
}

pub fn should_show_kill_dialog(key: &KeyEvent) -> bool {
    matches!(key.code, KeyCode::Char('k') | KeyCode::Char('K'))
}

pub fn should_show_terminate_dialog(key: &KeyEvent) -> bool {
    // Only 'T' (uppercase) for terminate to avoid conflict with tree toggle
    matches!(key.code, KeyCode::Char('T'))
}

pub fn should_toggle_tree(key: &KeyEvent) -> bool {
    matches!(key.code, KeyCode::Char('t'))
}

pub fn should_enter_filter(key: &KeyEvent) -> bool {
    matches!(key.code, KeyCode::Char('/'))
}

pub fn should_show_info(key: &KeyEvent) -> bool {
    matches!(key.code, KeyCode::Char('i') | KeyCode::Char('I'))
}

pub fn is_escape(key: &KeyEvent) -> bool {
    matches!(key.code, KeyCode::Esc)
}

pub fn is_yes(key: &KeyEvent) -> bool {
    matches!(key.code, KeyCode::Char('y') | KeyCode::Char('Y'))
}

pub fn is_no(key: &KeyEvent) -> bool {
    matches!(key.code, KeyCode::Char('n') | KeyCode::Char('N'))
}

pub fn is_arrow_left(key: &KeyEvent) -> bool {
    matches!(key.code, KeyCode::Left)
}

pub fn is_arrow_right(key: &KeyEvent) -> bool {
    matches!(key.code, KeyCode::Right)
}

pub fn should_enter_history(key: &KeyEvent) -> bool {
    matches!(key.code, KeyCode::Char('h') | KeyCode::Char('H'))
}
