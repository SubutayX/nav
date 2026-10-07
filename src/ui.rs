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

// Renk paleti (256 renk: çoğu terminalde aynı görünür)
const ACCENT: Color = Color::Cyan;
const MATCH: Color = Color::LightYellow;
const DIM: Color = Color::Indexed(245);
const SELECTED_BG: Color = Color::Indexed(237);

struct Item {
    entry: Entry,
    display: String, // ev dizini "~" ile kısaltılmış yol
    key: String,     // display'in normalize hali, her tuşta yeniden hesaplanmasın
}

struct App {
    input: String,
    items: Vec<Item>,
    results: Vec<(usize, Vec<usize>)>, // (items indeksi, eşleşen karakter indeksleri)
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
    let home = dirs::home_dir().map(|h| h.to_string_lossy().into_owned());
    let mut app = App {
        input: String::new(),
        items: all_files
            .into_par_iter()
            .map(|entry| {
                // Ev dizini aramaya dahil edilmez: her yolda olduğu için "help" gibi
                // sorgular "/home/..." içindeki harflerle sahte eşleşme üretiyordu
                let display = match &home {
                    Some(h) if entry.path.starts_with(h.as_str()) => {
                        format!("~{}", &entry.path[h.len()..])
                    }
                    _ => entry.path.clone(),
                };
                Item {
                    key: normalize(&display),
                    display,
                    entry,
                }
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
        terminal.draw(|f| draw(f, &mut app))?;

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
                if let Some((idx, _)) = app.list_state.selected().and_then(|i| app.results.get(i)) {
                    return Ok(Some(app.items[*idx].entry.path.clone()));
                }
            }
            _ => {}
        }
    }
}

fn draw(f: &mut Frame, app: &mut App) {
    let [input_area, list_area, help_area] = Layout::vertical([
        Constraint::Length(3),
        Constraint::Min(0),
        Constraint::Length(1),
    ])
    .areas(f.area());

    // --- Arama kutusu ---
    let mode = if app.show_only_dirs {
        "📁 Klasörler"
    } else {
        "📄 Hepsi"
    };
    let prompt = if app.input.is_empty() {
        Line::from(vec![
            Span::styled("❯ ", Style::new().fg(ACCENT).bold()),
            Span::styled(
                "Aramak için yazmaya başlayın…",
                Style::new().fg(DIM).italic(),
            ),
        ])
    } else {
        Line::from(vec![
            Span::styled("❯ ", Style::new().fg(ACCENT).bold()),
            Span::styled(app.input.as_str(), Style::new().bold()),
            Span::styled("▏", Style::new().fg(ACCENT)), // imleç
        ])
    };
    let input_box = Paragraph::new(prompt).block(
        Block::bordered()
            .border_type(BorderType::Rounded)
            .border_style(Style::new().fg(ACCENT))
            .title(Line::from(" nav ").bold())
            .title(Line::from(format!(" {mode} ")).right_aligned()),
    );
    f.render_widget(input_box, input_area);

    // --- Sonuç listesi ---
    let rows: Vec<ListItem> = app
        .results
        .iter()
        .map(|(idx, matched)| result_row(&app.items[*idx], matched))
        .collect();

    let title = if app.input.is_empty() {
        format!(" {} girdi indekslendi ", app.items.len())
    } else {
        format!(" {} sonuç ", app.results.len())
    };
    let block = Block::bordered()
        .border_type(BorderType::Rounded)
        .border_style(Style::new().fg(DIM))
        .title(title);

    if rows.is_empty() && !app.input.is_empty() {
        let empty = Paragraph::new(Line::from("Sonuç bulunamadı").fg(DIM).italic())
            .alignment(Alignment::Center)
            .block(block);
        f.render_widget(empty, list_area);
    } else {
        let list = List::new(rows)
            .block(block)
            // fg verilmiyor: seçili satırda da renkler korunsun
            .highlight_style(Style::new().bg(SELECTED_BG))
            .highlight_symbol(Line::from("▌ ").fg(ACCENT))
            .highlight_spacing(HighlightSpacing::Always);
        f.render_stateful_widget(list, list_area, &mut app.list_state);
    }

    // --- Kısayol çubuğu ---
    let key = |k: &'static str| Span::styled(k, Style::new().fg(ACCENT).bold());
    let label = |l: &'static str| Span::styled(l, Style::new().fg(DIM));
    let help = Line::from(vec![
        key(" ↑↓"),
        label(" gez   "),
        key("⏎"),
        label(" git   "),
        key("TAB"),
        label(" klasör/hepsi   "),
        key("ESC"),
        label(" çık"),
    ]);
    f.render_widget(help, help_area);
}

/// Bir sonuç satırı: ikon, kalın isim, ardından soluk ana dizin.
/// Örn: `📁 help   ~/Downloads/opt/OpenVSP`
fn result_row<'a>(item: &'a Item, matched: &[usize]) -> ListItem<'a> {
    let display = item.display.as_str();
    // Karakter sayısı tutmazsa (nadir Unicode durumları) indeksler güvenilmez, vurgulama yapma
    let matched = if display.chars().count() == item.key.chars().count() {
        matched
    } else {
        &[]
    };

    let name_start = display.rfind(['/', '\\']).map_or(0, |i| i + 1);
    let (parent, name) = display.split_at(name_start);
    let name = if name.is_empty() { parent } else { name }; // "/" veya "~" gibi kökler
    let name_offset = display[..display.len() - name.len()].chars().count();
    let parent = parent.trim_end_matches(['/', '\\']);

    let ext = std::path::Path::new(name).extension();
    let (icon, name_color) = if item.entry.is_dir {
        ("📁 ", Color::LightBlue)
    } else if ext.is_some_and(|e| e == "rs") {
        ("🦀 ", Color::White)
    } else if ext.is_some_and(|e| e == "toml") {
        ("⚙️  ", Color::White)
    } else {
        ("📄 ", Color::White)
    };

    let mut spans = vec![Span::raw(icon)];
    spans.extend(highlight(
        name,
        name_offset,
        matched,
        Style::new().fg(name_color).bold(),
    ));
    if !parent.is_empty() && name.len() != display.len() {
        spans.push(Span::raw("   "));
        spans.extend(highlight(parent, 0, matched, Style::new().fg(DIM)));
    }
    ListItem::new(Line::from(spans))
}

/// `text`i eşleşen ve eşleşmeyen parçalara böler. `offset`, `text`in tam yoldaki
/// karakter başlangıcıdır (`matched` indeksleri tam yola göredir).
fn highlight<'a>(text: &'a str, offset: usize, matched: &[usize], base: Style) -> Vec<Span<'a>> {
    let hit = base.fg(MATCH).bold().underlined();
    let mut spans = Vec::new();
    let mut start = 0; // mevcut parçanın byte başlangıcı
    let mut in_hit = false;
    for (ci, (bi, _)) in text.char_indices().enumerate() {
        let is_hit = matched.binary_search(&(offset + ci)).is_ok();
        if is_hit != in_hit && bi > start {
            spans.push(Span::styled(
                &text[start..bi],
                if in_hit { hit } else { base },
            ));
            start = bi;
        }
        in_hit = is_hit;
    }
    spans.push(Span::styled(
        &text[start..],
        if in_hit { hit } else { base },
    ));
    spans
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

    // Skor eşitse kısa yol önce: "help" klasörü "help/images"tan önce gelsin
    scored.par_sort_unstable_by_key(|&(s, i)| (std::cmp::Reverse(s), app.items[i].key.len()));
    scored.truncate(settings.limit);
    // İndeksler sadece gösterilen birkaç sonuç için hesaplanır (pahalı, arama döngüsüne girmez)
    app.results = scored
        .into_iter()
        .map(|(_, i)| {
            let matched = matcher
                .fuzzy_indices(&app.items[i].key, &query)
                .map(|(_, m)| m)
                .unwrap_or_default();
            (i, matched)
        })
        .collect();

    app.list_state.select(if app.results.is_empty() {
        None
    } else {
        Some(0)
    });
}

#[cfg(test)]
mod tests {
    use super::{highlight, normalize};
    use ratatui::style::Style;

    #[test]
    fn normalize_turkish() {
        assert_eq!(normalize("İstanbul/Çalışma/ĞÜŞÖ"), "istanbul/calisma/guso");
    }

    #[test]
    fn highlight_splits_matched_runs() {
        // "src", tam yolda 9. karakterden başlıyor; 9 ve 10 eşleşmiş
        let parts: Vec<String> = highlight("src", 9, &[1, 2, 9, 10], Style::new())
            .iter()
            .map(|s| s.content.to_string())
            .collect();
        assert_eq!(parts, ["sr", "c"]);
    }
}
