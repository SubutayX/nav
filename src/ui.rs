use crate::scanner::Entry;
use crate::settings::Settings;
use crossterm::{
    event::{self, Event, KeyCode, KeyEventKind, KeyModifiers},
    execute,
    terminal::*,
};
use fuzzy_matcher::FuzzyMatcher;
use fuzzy_matcher::skim::SkimMatcherV2;
use ratatui::{prelude::*, widgets::*};
use rayon::prelude::*;
use std::{error::Error, io, time::Duration};

/// Aramalar bu süre boyunca yeni tuş gelmezse çalışır (debounce).
const DEBOUNCE: Duration = Duration::from_millis(20);

struct Item {
    entry: Entry,
    key: String, // önceden normalize edilmiş yol, her tuşta yeniden hesaplanmasın
}

struct App {
    input: String,
    items: Vec<Item>,
    results: Vec<(i64, usize)>, // (skor, items indeksi)
    list_state: ListState,
    show_only_dirs: bool,
    needs_update: bool,
}

/// Raw mode ve alternate screen'i her durumda (hata, panic dahil) geri alır.
struct TerminalGuard;

impl Drop for TerminalGuard {
    fn drop(&mut self) {
        let _ = disable_raw_mode();
        let _ = execute!(io::stderr(), LeaveAlternateScreen, crossterm::cursor::Show);
    }
}

pub fn run_tui(
    all_files: Vec<Entry>,
    settings: &Settings,
) -> Result<Option<String>, Box<dyn Error>> {
    let mut app = App {
        input: String::new(),
        items: all_files
            .into_par_iter()
            .map(|entry| Item {
                key: normalize(&entry.path),
                entry,
            })
            .collect(),
        results: vec![],
        list_state: ListState::default(),
        show_only_dirs: true,
        needs_update: false,
    };

    // TUI stderr'e çizilir; stdout sadece seçilen yol için kalır (`cd (nav)` çalışsın diye)
    enable_raw_mode()?;
    let _guard = TerminalGuard;
    execute!(io::stderr(), EnterAlternateScreen)?;
    let mut terminal = Terminal::new(CrosstermBackend::new(io::stderr()))?;
    terminal.hide_cursor()?;

    let matcher = SkimMatcherV2::default();

    loop {
        terminal.draw(|f| draw(f, &mut app, settings))?;

        // Bekleyen arama varsa kısa, yoksa uzun bekle: boşta CPU yakmasın
        let timeout = if app.needs_update {
            DEBOUNCE
        } else {
            Duration::from_secs(1)
        };
        if !event::poll(timeout)? {
            if app.needs_update {
                update_search(&mut app, &matcher, settings);
            }
            continue;
        }

        // Windows'ta tuş bırakma olayları da gelir, sadece basmaları al
        let Event::Key(key) = event::read()? else {
            continue;
        };
        if key.kind != KeyEventKind::Press {
            continue;
        }

        match key.code {
            KeyCode::Esc => return Ok(None),
            KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => return Ok(None),
            KeyCode::Char(c) => {
                app.input.push(c);
                app.needs_update = true;
            }
            KeyCode::Backspace => {
                app.input.pop();
                app.needs_update = true;
            }
            KeyCode::Down if !app.results.is_empty() => {
                let i = app
                    .list_state
                    .selected()
                    .map_or(0, |i| (i + 1) % app.results.len());
                app.list_state.select(Some(i));
            }
            KeyCode::Up if !app.results.is_empty() => {
                let len = app.results.len();
                let i = app.list_state.selected().map_or(0, |i| (i + len - 1) % len);
                app.list_state.select(Some(i));
            }
            KeyCode::Tab => {
                app.show_only_dirs = !app.show_only_dirs;
                update_search(&mut app, &matcher, settings);
            }
            KeyCode::Enter => {
                if let Some(&(_, idx)) = app.list_state.selected().and_then(|i| app.results.get(i))
                {
                    return Ok(Some(app.items[idx].entry.path.clone()));
                }
            }
            _ => {}
        }
    }
}

fn draw(f: &mut Frame, app: &mut App, settings: &Settings) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(3), Constraint::Min(0)])
        .split(f.area());

    let mode_info = if app.show_only_dirs {
        Span::styled(
            " [MOD: Klasör] ",
            Style::default()
                .fg(Color::Magenta)
                .add_modifier(Modifier::BOLD),
        )
    } else {
        Span::styled(
            " [MOD: Hepsi] ",
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        )
    };

    let title = Line::from(vec![
        Span::styled(
            " Ara ",
            Style::default()
                .fg(Color::Green)
                .add_modifier(Modifier::BOLD),
        ),
        mode_info,
        Span::raw(" (Toplam "),
        Span::styled(
            app.items.len().to_string(),
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw(" Girdi) - "),
        Span::styled("[TAB: Mod Değiştir] ", Style::default().fg(Color::DarkGray)),
        Span::styled("[ESC: Çık]", Style::default().fg(Color::DarkGray)),
    ]);

    let input_box = Paragraph::new(app.input.as_str())
        .style(
            Style::default()
                .fg(Color::White)
                .add_modifier(Modifier::BOLD),
        )
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Blue))
                .title(title),
        );
    f.render_widget(input_box, chunks[0]);

    let items: Vec<ListItem> = app
        .results
        .iter()
        .map(|&(score, idx)| {
            let entry = &app.items[idx].entry;
            let ext = std::path::Path::new(&entry.path).extension();
            let icon = if entry.is_dir {
                "📁 "
            } else if ext.is_some_and(|e| e == "rs") {
                "🦀 "
            } else if ext.is_some_and(|e| e == "toml") {
                "⚙️  "
            } else {
                "📄 "
            };

            ListItem::new(Line::from(vec![
                Span::styled(
                    format!("[{:>3}] ", score),
                    Style::default().fg(Color::Yellow),
                ),
                Span::raw(icon),
                Span::raw(entry.path.as_str()),
            ]))
        })
        .collect();

    let list = List::new(items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(format!(" En Yakın {} Sonuç ", settings.limit)),
        )
        .highlight_style(
            Style::default()
                .bg(Color::Blue)
                .fg(Color::White)
                .add_modifier(Modifier::BOLD),
        );

    f.render_stateful_widget(list, chunks[1], &mut app.list_state);
}

/// Türkçe karakterleri ASCII karşılığına çevirip küçük harfe indirir.
/// Önce replace, sonra lowercase: aksi halde 'İ' -> "i̇" (birleşik nokta) olur.
fn normalize(text: &str) -> String {
    text.replace(['ğ', 'Ğ'], "g")
        .replace(['ü', 'Ü'], "u")
        .replace(['ş', 'Ş'], "s")
        .replace(['ı', 'İ'], "i")
        .replace(['ö', 'Ö'], "o")
        .replace(['ç', 'Ç'], "c")
        .to_lowercase()
}

fn update_search(app: &mut App, matcher: &SkimMatcherV2, settings: &Settings) {
    app.needs_update = false;

    if app.input.is_empty() {
        app.results.clear();
        app.list_state.select(None);
        return;
    }

    let query = normalize(&app.input);
    let mut scored: Vec<(i64, usize)> = app
        .items
        .par_iter()
        .enumerate()
        .filter(|(_, item)| !app.show_only_dirs || item.entry.is_dir)
        .filter_map(|(i, item)| {
            matcher
                .fuzzy_match(&item.key, &query)
                .filter(|&s| s >= settings.fuzzy_threshold)
                .map(|s| (s, i))
        })
        .collect();

    scored.par_sort_unstable_by_key(|&(s, _)| std::cmp::Reverse(s));
    scored.truncate(settings.limit);
    app.results = scored;

    app.list_state.select(if app.results.is_empty() {
        None
    } else {
        Some(0)
    });
}

#[cfg(test)]
mod tests {
    use super::normalize;

    #[test]
    fn normalize_turkish() {
        assert_eq!(normalize("İstanbul/Çalışma/ĞÜŞÖ"), "istanbul/calisma/guso");
    }
}
