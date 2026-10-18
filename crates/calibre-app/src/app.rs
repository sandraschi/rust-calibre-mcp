//! Pure-Rust `egui` desktop UI for `rust-calibre-mcp`.
//!
//! Enforces the 10-Point Uncrashable Binary Contract:
//! - Zero synchronous SQLite / network / keyring I/O on the UI thread (`self.spawn(...)`).
//! - Zero `ctx.set_visuals()` / `ctx.set_zoom_factor()` inside `ctx.input(|i| ...)`.
//! - Strict glyph allowlist (`°`, `×`, `·`, `…`) verified at compile/test time by `glyph_source_scan`.

use calibre_core::calibre_db::BookQueryParams;
use calibre_core::{dispatch, Command};
use eframe::egui::{
    self, Color32, CornerRadius, FontId, Margin, Pos2, Rect, RichText, Stroke, Vec2,
};
use serde_json::{json, Value};
use std::future::Future;
use std::sync::mpsc::{self, Receiver, Sender};
use std::sync::Arc;

const BG_DARK: Color32 = Color32::from_rgb(9, 13, 22);
const PANEL_DARK: Color32 = Color32::from_rgb(15, 23, 42);
const CARD_DARK: Color32 = Color32::from_rgb(22, 32, 56);
const BORDER_DARK: Color32 = Color32::from_rgb(45, 62, 96);
const ACCENT: Color32 = Color32::from_rgb(56, 189, 248);
const EMERALD: Color32 = Color32::from_rgb(52, 211, 153);
const AMBER: Color32 = Color32::from_rgb(251, 191, 36);
const ROSE: Color32 = Color32::from_rgb(251, 113, 133);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Theme {
    Dark,
    Light,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Page {
    Libraries,
    Books,
    Fulltext,
    Analytics,
    Crossconnect,
    Chat,
    Settings,
    Help,
    RawJson,
}

enum Job {
    Libraries(Value),
    Books(Value),
    BookDetail {
        detail: Value,
        cover_jpeg: Option<Vec<u8>>,
    },
    BookExcerpt(Value),
    Fulltext(Value),
    Health(Value),
    Series(Value),
    ReadingStats(Value),
    UnreadPriority(Value),
    ChatReply(Value),
    StatusNote(String, Value),
    VaultLoaded {
        calibre_server_url: String,
        calibre_password: String,
        llm_api_key: String,
    },
}

pub struct CalibreApp {
    runtime: Arc<tokio::runtime::Runtime>,
    tx: Sender<Job>,
    rx: Receiver<Job>,
    pending: usize,
    theme: Theme,
    sidebar_open: bool,
    zoom: f32,
    page: Page,
    status_line: String,
    active_library: String,
    libraries_val: Value,
    books_val: Value,
    selected_book: Option<Value>,
    cover_tex: Option<egui::TextureHandle>,
    excerpt_val: Value,
    excerpt_offset: usize,
    fulltext_val: Value,
    health_val: Value,
    series_val: Value,
    reading_stats_val: Value,
    priority_val: Value,
    last_raw: Value,
    // Search & filter inputs
    q_text: String,
    q_author: String,
    q_tag: String,
    q_series: String,
    q_format: String,
    fts_query: String,
    // Chat state (`gemma4:12b` default)
    chat_provider: String,
    chat_model: String,
    chat_prompt: String,
    chat_reply: String,
    // Crossconnect state
    speak_text: String,
    speech_url: String,
    webhook_url: String,
    // Settings / Vault state
    vault_loaded: bool,
    vault_status_note: String,
    calibre_server_url: String,
    calibre_password: String,
    llm_api_key: String,
    llm_base_url: String,
    custom_library_name: String,
    custom_library_path: String,
    pinned_registry: crate::registry::Registry,
}

pub fn harsh_visuals() -> egui::Visuals {
    let mut v = egui::Visuals::dark();
    v.panel_fill = PANEL_DARK;
    v.window_fill = BG_DARK;
    v.extreme_bg_color = BG_DARK;
    v.widgets.noninteractive.bg_fill = CARD_DARK;
    v.widgets.noninteractive.bg_stroke = Stroke::new(1.0, BORDER_DARK);
    v.widgets.inactive.bg_fill = Color32::from_rgb(30, 41, 69);
    v.widgets.hovered.bg_fill = Color32::from_rgb(42, 58, 94);
    v.widgets.active.bg_fill = ACCENT;
    v.selection.bg_fill = Color32::from_rgb(14, 116, 144);
    v
}

pub fn crisp_light_visuals() -> egui::Visuals {
    let mut v = egui::Visuals::light();
    v.panel_fill = Color32::from_rgb(248, 250, 252);
    v.window_fill = Color32::from_rgb(255, 255, 255);
    v.extreme_bg_color = Color32::from_rgb(241, 245, 249);
    v.widgets.noninteractive.bg_stroke = Stroke::new(1.0, Color32::from_rgb(203, 213, 225));
    v
}

pub fn apply_theme(ctx: &egui::Context, theme: Theme) {
    match theme {
        Theme::Dark => ctx.set_visuals(harsh_visuals()),
        Theme::Light => ctx.set_visuals(crisp_light_visuals()),
    }
}

pub fn card_frame(ui: &mut egui::Ui, add_contents: impl FnOnce(&mut egui::Ui)) {
    let is_dark = ui.visuals().dark_mode;
    let fill = if is_dark {
        CARD_DARK
    } else {
        Color32::from_rgb(255, 255, 255)
    };
    let stroke_color = if is_dark {
        BORDER_DARK
    } else {
        Color32::from_rgb(203, 213, 225)
    };
    egui::Frame::NONE
        .fill(fill)
        .stroke(Stroke::new(1.0, stroke_color))
        .corner_radius(CornerRadius::same(8))
        .inner_margin(Margin::same(12))
        .show(ui, add_contents);
}

pub fn hero_kpi(ui: &mut egui::Ui, label: &str, value: &str, accent: Color32) {
    card_frame(ui, |ui| {
        ui.set_min_width(145.0);
        ui.label(RichText::new(label).size(11.5).weak());
        ui.add_space(4.0);
        ui.label(RichText::new(value).size(20.0).strong().color(accent));
    });
}

pub fn status_pill(ui: &mut egui::Ui, active: bool, text: &str) {
    let color = if active { EMERALD } else { AMBER };
    ui.horizontal(|ui| {
        let (rect, _) = ui.allocate_exact_size(Vec2::splat(10.0), egui::Sense::hover());
        ui.painter().circle_filled(rect.center(), 4.5, color);
        ui.label(RichText::new(text).size(12.0).color(color).strong());
    });
}

/// Paint a crisp vector Sun (in dark mode) or crescent Moon (in light mode) icon button.
fn paint_theme_button(ui: &mut egui::Ui, theme: Theme) -> egui::Response {
    let (rect, resp) = ui.allocate_exact_size(Vec2::splat(26.0), egui::Sense::click());
    if ui.is_rect_visible(rect) {
        let p = ui.painter();
        let c = rect.center();
        if resp.hovered() {
            p.circle_filled(c, 12.0, Color32::from_rgba_unmultiplied(120, 140, 180, 45));
        }
        match theme {
            Theme::Dark => {
                p.circle_filled(c, 5.0, AMBER);
                for i in 0..8 {
                    let angle = (i as f32) * std::f32::consts::FRAC_PI_4;
                    let p1 = Pos2::new(c.x + angle.cos() * 7.5, c.y + angle.sin() * 7.5);
                    let p2 = Pos2::new(c.x + angle.cos() * 10.5, c.y + angle.sin() * 10.5);
                    p.line_segment([p1, p2], Stroke::new(1.6, AMBER));
                }
            }
            Theme::Light => {
                p.circle_filled(c, 7.0, Color32::from_rgb(51, 65, 85));
                p.circle_filled(
                    Pos2::new(c.x + 3.0, c.y - 2.5),
                    6.0,
                    ui.visuals().panel_fill,
                );
            }
        }
    }
    resp.on_hover_text("Toggle Dark / Light theme")
}

/// GPU horizontal bar chart for library book counts.
pub fn bar_chart(ui: &mut egui::Ui, title: &str, bars: &[(String, f64)], accent: Color32) {
    card_frame(ui, |ui| {
        ui.label(RichText::new(title).strong());
        ui.add_space(6.0);
        let max_val = bars.iter().map(|(_, v)| *v).fold(1.0_f64, f64::max);
        for (name, val) in bars.iter().take(12) {
            ui.horizontal(|ui| {
                ui.set_min_height(20.0);
                ui.add_sized(
                    [190.0, 18.0],
                    egui::Label::new(RichText::new(name).size(12.0)).truncate(),
                );
                let avail = (ui.available_width() - 80.0).max(60.0);
                let (rect, _) =
                    ui.allocate_exact_size(Vec2::new(avail, 14.0), egui::Sense::hover());
                ui.painter().rect_filled(
                    rect,
                    4.0,
                    Color32::from_rgba_unmultiplied(100, 116, 139, 40),
                );
                let frac = ((*val / max_val) as f32).clamp(0.02, 1.0);
                let fill_rect = Rect::from_min_size(rect.min, Vec2::new(rect.width() * frac, 14.0));
                ui.painter().rect_filled(fill_rect, 4.0, accent);
                ui.label(
                    RichText::new(format!("{}", *val as i64))
                        .size(12.0)
                        .strong(),
                );
            });
        }
    });
}

impl CalibreApp {
    pub fn new(cc: &eframe::CreationContext<'_>, runtime: Arc<tokio::runtime::Runtime>) -> Self {
        apply_theme(&cc.egui_ctx, Theme::Dark);
        let (tx, rx) = mpsc::channel();
        let mut app = Self {
            runtime,
            tx,
            rx,
            pending: 0,
            theme: Theme::Dark,
            sidebar_open: true,
            zoom: 1.0,
            page: Page::Libraries,
            status_line: "Discovering Calibre libraries…".to_string(),
            active_library: String::new(),
            libraries_val: json!({}),
            books_val: json!({}),
            selected_book: None,
            cover_tex: None,
            excerpt_val: json!({}),
            excerpt_offset: 0,
            fulltext_val: json!({}),
            health_val: json!({}),
            series_val: json!({}),
            reading_stats_val: json!({}),
            priority_val: json!({}),
            last_raw: json!({}),
            q_text: String::new(),
            q_author: String::new(),
            q_tag: String::new(),
            q_series: String::new(),
            q_format: String::new(),
            fts_query: String::new(),
            chat_provider: "ollama".to_string(),
            chat_model: "gemma4:12b".to_string(),
            chat_prompt:
                "Recommend 3 standout books from my active Calibre library and explain why."
                    .to_string(),
            chat_reply: String::new(),
            speak_text: "Calibre Rust library browser ready.".to_string(),
            speech_url: "http://127.0.0.1:10909/speak".to_string(),
            webhook_url: "http://127.0.0.1:10946/api/events".to_string(),
            vault_loaded: false,
            vault_status_note: "Keyring: ready".to_string(),
            calibre_server_url: "http://127.0.0.1:8080".to_string(),
            calibre_password: String::new(),
            llm_api_key: String::new(),
            llm_base_url: "http://127.0.0.1:11434".to_string(),
            custom_library_name: String::new(),
            custom_library_path: String::new(),
            pinned_registry: crate::registry::load(),
        };
        app.spawn_libraries();
        app.spawn_query_books();
        app
    }

    fn spawn<F, Fut>(&mut self, f: F)
    where
        F: FnOnce() -> Fut + Send + 'static,
        Fut: Future<Output = Job> + Send + 'static,
    {
        self.pending = self.pending.saturating_add(1);
        let tx = self.tx.clone();
        self.runtime.spawn(async move {
            let job = f().await;
            let _ = tx.send(job);
        });
    }

    fn spawn_libraries(&mut self) {
        self.status_line = "Loading Calibre libraries…".to_string();
        self.spawn(|| async move {
            Job::Libraries(
                dispatch(Command::LibrariesList {
                    include_counts: true,
                })
                .await,
            )
        });
    }

    fn spawn_query_books(&mut self) {
        let params = BookQueryParams {
            library: if self.active_library.is_empty() {
                None
            } else {
                Some(self.active_library.clone())
            },
            query: if self.q_text.trim().is_empty() {
                None
            } else {
                Some(self.q_text.trim().to_string())
            },
            author: if self.q_author.trim().is_empty() {
                None
            } else {
                Some(self.q_author.trim().to_string())
            },
            tag: if self.q_tag.trim().is_empty() {
                None
            } else {
                Some(self.q_tag.trim().to_string())
            },
            series: if self.q_series.trim().is_empty() {
                None
            } else {
                Some(self.q_series.trim().to_string())
            },
            format: if self.q_format.trim().is_empty() {
                None
            } else {
                Some(self.q_format.trim().to_string())
            },
            limit: Some(60),
            offset: Some(0),
        };
        self.status_line = "Querying books…".to_string();
        self.spawn(move || async move { Job::Books(dispatch(Command::QueryBooks(params)).await) });
    }

    fn spawn_search_all_libraries(&mut self) {
        let q = self.q_text.trim().to_string();
        if q.is_empty() {
            self.status_line =
                "Enter a search term to search across all discovered libraries.".to_string();
            return;
        }
        self.status_line = format!("Searching all discovered libraries for '{q}'…");
        self.spawn(move || async move {
            let v = dispatch(Command::SearchAllLibraries {
                query: q,
                limit_per_library: 15,
            })
            .await;
            let mut flattened = Vec::new();
            if let Some(arr) = v.get("results_by_library").and_then(Value::as_array) {
                for entry in arr {
                    if let Some(books) = entry.get("books").and_then(Value::as_array) {
                        for b in books {
                            flattened.push(b.clone());
                        }
                    }
                }
            }
            let total = flattened.len();
            Job::Books(json!({
                "mode": v.get("mode"),
                "library_name": "All Libraries (Federated)",
                "count": total,
                "books": flattened,
                "federated_summary": v,
            }))
        });
    }

    fn spawn_book_detail(&mut self, book_id: i64) {
        let lib = if self.active_library.is_empty() {
            None
        } else {
            Some(self.active_library.clone())
        };
        let lib_cover = lib.clone();
        self.excerpt_val = json!({});
        self.excerpt_offset = 0;
        self.status_line = format!("Loading book #{book_id} + cover.jpg…");
        self.spawn(move || async move {
            let detail = dispatch(Command::BookGet {
                library: lib,
                book_id,
            })
            .await;
            let cover_jpeg = tokio::task::spawn_blocking(move || {
                calibre_core::calibre_db::book_cover_bytes(lib_cover.as_deref(), book_id)
                    .ok()
                    .and_then(|(bytes, _)| bytes)
            })
            .await
            .ok()
            .flatten();
            Job::BookDetail { detail, cover_jpeg }
        });
    }

    fn spawn_book_excerpt(&mut self, book_id: i64, offset_chars: usize) {
        let lib = if self.active_library.is_empty() {
            None
        } else {
            Some(self.active_library.clone())
        };
        self.excerpt_offset = offset_chars;
        self.status_line = format!("Reading excerpt for book #{book_id} (offset {offset_chars})…");
        self.spawn(move || async move {
            Job::BookExcerpt(
                dispatch(Command::BookExcerpt {
                    library: lib,
                    book_id,
                    format: None,
                    offset_chars,
                    max_chars: 1800,
                })
                .await,
            )
        });
    }

    fn spawn_fulltext(&mut self) {
        let q = self.fts_query.trim().to_string();
        if q.is_empty() {
            self.status_line = "Enter a full-text search phrase first.".to_string();
            return;
        }
        let lib = if self.active_library.is_empty() {
            None
        } else {
            Some(self.active_library.clone())
        };
        self.status_line = format!("Searching full text for '{q}'…");
        self.spawn(move || async move {
            Job::Fulltext(
                dispatch(Command::SearchFulltext {
                    library: lib,
                    query: q,
                    limit: 30,
                })
                .await,
            )
        });
    }

    fn spawn_analytics(&mut self) {
        let lib1 = if self.active_library.is_empty() {
            None
        } else {
            Some(self.active_library.clone())
        };
        let lib2 = lib1.clone();
        let lib3 = lib1.clone();
        let lib4 = lib1.clone();
        self.status_line =
            "Running library health, reading stats & canon coverage audit…".to_string();
        self.spawn(move || async move {
            Job::Health(dispatch(Command::LibraryHealth { library: lib1 }).await)
        });
        self.spawn(move || async move {
            Job::Series(
                dispatch(Command::SeriesAnalysis {
                    library: lib2,
                    query: None,
                    limit: 40,
                })
                .await,
            )
        });
        self.spawn(move || async move {
            Job::ReadingStats(dispatch(Command::ReadingStats { library: lib3 }).await)
        });
        self.spawn(move || async move {
            Job::UnreadPriority(
                dispatch(Command::UnreadPriority {
                    library: lib4,
                    limit: 15,
                })
                .await,
            )
        });
    }

    fn drain_jobs(&mut self, ctx: &egui::Context) {
        while let Ok(job) = self.rx.try_recv() {
            self.pending = self.pending.saturating_sub(1);
            match job {
                Job::Libraries(v) => {
                    if self.active_library.is_empty() {
                        if let Some(def) = v.get("default_library").and_then(Value::as_str) {
                            self.active_library = def.to_string();
                        }
                    }
                    let count = v
                        .get("total_libraries")
                        .and_then(Value::as_u64)
                        .unwrap_or(0);
                    self.status_line = format!("Discovered {count} Calibre libraries");
                    self.last_raw = v.clone();
                    self.libraries_val = v;
                }
                Job::Books(v) => {
                    let count = v.get("count").and_then(Value::as_u64).unwrap_or(0);
                    let lib_name = v
                        .get("library_name")
                        .and_then(Value::as_str)
                        .unwrap_or("library");
                    self.status_line = format!("Loaded {count} books from {lib_name}");
                    self.last_raw = v.clone();
                    self.books_val = v;
                }
                Job::BookDetail { detail, cover_jpeg } => {
                    self.status_line = "Book details loaded".to_string();
                    self.cover_tex = cover_jpeg.and_then(|bytes| {
                        let decoded = image::load_from_memory(&bytes).ok()?.to_rgba8();
                        let size = [decoded.width() as usize, decoded.height() as usize];
                        let color_img =
                            egui::ColorImage::from_rgba_unmultiplied(size, decoded.as_raw());
                        Some(ctx.load_texture(
                            "calibre-book-cover",
                            color_img,
                            egui::TextureOptions::LINEAR,
                        ))
                    });
                    self.last_raw = detail.clone();
                    self.selected_book = Some(detail);
                }
                Job::BookExcerpt(v) => {
                    self.status_line = "Book excerpt loaded".to_string();
                    self.last_raw = v.clone();
                    self.excerpt_val = v;
                }
                Job::Fulltext(v) => {
                    let count = v.get("count").and_then(Value::as_u64).unwrap_or(0);
                    self.status_line = format!("Full-text search returned {count} hits");
                    self.last_raw = v.clone();
                    self.fulltext_val = v;
                }
                Job::Health(v) => {
                    self.status_line = "Library health audit complete".to_string();
                    self.last_raw = v.clone();
                    self.health_val = v;
                }
                Job::Series(v) => {
                    self.series_val = v;
                }
                Job::ReadingStats(v) => {
                    self.reading_stats_val = v;
                }
                Job::UnreadPriority(v) => {
                    self.priority_val = v;
                }
                Job::ChatReply(v) => {
                    if let Some(r) = v.get("reply").and_then(Value::as_str) {
                        self.chat_reply = r.to_string();
                        self.status_line = "LLM reply received".to_string();
                    } else if let Some(err) = v.get("error").and_then(Value::as_str) {
                        self.chat_reply = format!("Error: {err}");
                        self.status_line = "LLM chat failed".to_string();
                    }
                    self.last_raw = v;
                }
                Job::StatusNote(note, v) => {
                    self.status_line = note;
                    self.last_raw = v;
                }
                Job::VaultLoaded {
                    calibre_server_url,
                    calibre_password,
                    llm_api_key,
                } => {
                    if !calibre_server_url.is_empty() {
                        self.calibre_server_url = calibre_server_url;
                    }
                    self.calibre_password = calibre_password;
                    self.llm_api_key = llm_api_key;
                    self.vault_status_note =
                        "OS Keyring: connected to Windows Credential Manager ('rust-calibre-mcp')"
                            .to_string();
                }
            }
        }
    }

    fn view_libraries(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.heading("Calibre Libraries");
            if ui.button("Refresh Libraries").clicked() {
                self.spawn_libraries();
            }
            if ui.button("Use Demo Library").clicked() {
                self.active_library = "demo".to_string();
                self.spawn_query_books();
                self.page = Page::Books;
            }
        });
        ui.add_space(8.0);

        let total_libs = self
            .libraries_val
            .get("total_libraries")
            .and_then(Value::as_i64)
            .unwrap_or(0);
        let total_books = self
            .libraries_val
            .get("total_books_across_libraries")
            .and_then(Value::as_i64)
            .unwrap_or(0);
        let active_label = if self.active_library.is_empty() {
            "Default Library"
        } else {
            &self.active_library
        };

        ui.horizontal_wrapped(|ui| {
            hero_kpi(ui, "Discovered Libraries", &total_libs.to_string(), ACCENT);
            hero_kpi(ui, "Total Books Indexed", &total_books.to_string(), EMERALD);
            hero_kpi(ui, "Active Library", active_label, AMBER);
        });
        ui.add_space(10.0);

        let mut bars = Vec::new();
        if let Some(arr) = self
            .libraries_val
            .get("libraries")
            .and_then(Value::as_array)
        {
            for lib in arr {
                let name = lib
                    .get("name")
                    .and_then(Value::as_str)
                    .unwrap_or("Unknown")
                    .to_string();
                let count = lib.get("book_count").and_then(Value::as_f64).unwrap_or(0.0);
                bars.push((name, count));
            }
        }
        if !bars.is_empty() {
            bar_chart(ui, "Books per Calibre Library", &bars, ACCENT);
            ui.add_space(10.0);
        }

        let libs = self
            .libraries_val
            .get("libraries")
            .and_then(Value::as_array)
            .cloned()
            .unwrap_or_default();

        egui::ScrollArea::vertical().show(ui, |ui| {
            for lib in libs {
                let name = lib.get("name").and_then(Value::as_str).unwrap_or("Library");
                let path = lib.get("path").and_then(Value::as_str).unwrap_or("");
                let size_mb = lib
                    .get("metadata_size_mb")
                    .and_then(Value::as_f64)
                    .unwrap_or(0.0);
                let count = lib.get("book_count").and_then(Value::as_i64).unwrap_or(0);
                let has_fts = lib
                    .get("has_fts_db")
                    .and_then(Value::as_bool)
                    .unwrap_or(false);
                let is_active = self.active_library == name;

                card_frame(ui, |ui| {
                    ui.horizontal(|ui| {
                        status_pill(ui, is_active, if is_active { "ACTIVE" } else { "READY" });
                        ui.label(RichText::new(name).size(15.5).strong());
                        ui.label(
                            RichText::new(format!("· {count} books · {size_mb:.2} MB metadata.db"))
                                .weak(),
                        );
                        if has_fts {
                            ui.label(RichText::new("· FTS5 Ready").color(EMERALD).size(12.0));
                        }
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            if ui.button("Browse Books").clicked() {
                                self.active_library = name.to_string();
                                self.spawn_query_books();
                                self.page = Page::Books;
                            }
                            if ui.button("Health Audit").clicked() {
                                self.active_library = name.to_string();
                                self.spawn_analytics();
                                self.page = Page::Analytics;
                            }
                        });
                    });
                    ui.label(RichText::new(path).size(11.5).weak());
                });
                ui.add_space(6.0);
            }
        });
    }

    fn view_books(&mut self, ui: &mut egui::Ui) {
        ui.horizontal_wrapped(|ui| {
            ui.heading(format!(
                "Books · {}",
                if self.active_library.is_empty() {
                    "Default Library"
                } else {
                    &self.active_library
                }
            ));
            ui.separator();
            ui.label("Search:");
            let q_resp = ui.add(
                egui::TextEdit::singleline(&mut self.q_text)
                    .hint_text("title, author, synopsis…")
                    .desired_width(160.0),
            );
            ui.label("Author:");
            ui.add(
                egui::TextEdit::singleline(&mut self.q_author)
                    .hint_text("e.g. Herbert")
                    .desired_width(110.0),
            );
            ui.label("Tag:");
            ui.add(
                egui::TextEdit::singleline(&mut self.q_tag)
                    .hint_text("e.g. Sci-Fi")
                    .desired_width(95.0),
            );
            ui.label("Format:");
            ui.add(
                egui::TextEdit::singleline(&mut self.q_format)
                    .hint_text("EPUB/PDF")
                    .desired_width(70.0),
            );
            if ui.button("Search").clicked()
                || (q_resp.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)))
            {
                self.spawn_query_books();
            }
            if ui.button("Search All Libraries").clicked() {
                self.spawn_search_all_libraries();
            }
            if ui.button("Clear").clicked() {
                self.q_text.clear();
                self.q_author.clear();
                self.q_tag.clear();
                self.q_series.clear();
                self.q_format.clear();
                self.spawn_query_books();
            }
        });
        ui.add_space(8.0);

        let books = self
            .books_val
            .get("books")
            .and_then(Value::as_array)
            .cloned()
            .unwrap_or_default();

        ui.columns(2, |cols| {
            cols[0].label(RichText::new(format!("Showing {} books", books.len())).weak());
            egui::ScrollArea::vertical()
                .id_salt("books_list_scroll")
                .show(&mut cols[0], |ui| {
                    for b in &books {
                        let id = b.get("id").and_then(Value::as_i64).unwrap_or(0);
                        let title = b.get("title").and_then(Value::as_str).unwrap_or("Untitled");
                        let authors = b
                            .get("authors")
                            .and_then(Value::as_array)
                            .map(|arr| {
                                arr.iter()
                                    .filter_map(Value::as_str)
                                    .collect::<Vec<_>>()
                                    .join(", ")
                            })
                            .unwrap_or_default();
                        let formats = b
                            .get("formats")
                            .and_then(Value::as_array)
                            .map(|arr| {
                                arr.iter()
                                    .filter_map(Value::as_str)
                                    .collect::<Vec<_>>()
                                    .join(" · ")
                            })
                            .unwrap_or_default();
                        let snippet = b
                            .get("summary_snippet")
                            .and_then(Value::as_str)
                            .unwrap_or("");

                        card_frame(ui, |ui| {
                            ui.horizontal(|ui| {
                                ui.label(RichText::new(format!("#{id}")).color(ACCENT).strong());
                                ui.label(RichText::new(title).size(14.5).strong());
                                ui.with_layout(
                                    egui::Layout::right_to_left(egui::Align::Center),
                                    |ui| {
                                        if ui.button("Inspect").clicked() {
                                            self.spawn_book_detail(id);
                                        }
                                        if !formats.is_empty() {
                                            ui.label(
                                                RichText::new(&formats).size(11.0).color(EMERALD),
                                            );
                                        }
                                    },
                                );
                            });
                            if !authors.is_empty() {
                                ui.label(RichText::new(authors).size(12.0).weak());
                            }
                            if !snippet.is_empty() {
                                ui.label(RichText::new(snippet).size(11.5));
                            }
                        });
                        ui.add_space(5.0);
                    }
                });

            cols[1].heading("Book Inspector");
            cols[1].add_space(4.0);
            if let Some(detail) = self.selected_book.clone() {
                let book = detail.get("book").cloned().unwrap_or_else(|| json!({}));
                let book_id = book.get("id").and_then(Value::as_i64).unwrap_or(0);
                let title = book
                    .get("title")
                    .and_then(Value::as_str)
                    .unwrap_or("Selected Book");
                let synopsis = detail
                    .get("comments_plain")
                    .and_then(Value::as_str)
                    .unwrap_or("No synopsis recorded in metadata.db.");

                card_frame(&mut cols[1], |ui| {
                    ui.horizontal(|ui| {
                        if let Some(tex) = &self.cover_tex {
                            let sz = tex.size_vec2();
                            let scale = (115.0 / sz.y.max(1.0)).min(1.0);
                            ui.add(
                                egui::Image::new(tex)
                                    .fit_to_exact_size(egui::vec2(sz.x * scale, sz.y * scale))
                                    .corner_radius(egui::CornerRadius::same(6)),
                            );
                        }
                        ui.vertical(|ui| {
                            ui.label(RichText::new(title).size(17.0).strong().color(ACCENT));
                            if let Some(series) = book.get("series").and_then(Value::as_str) {
                                let idx = book
                                    .get("series_index")
                                    .and_then(Value::as_f64)
                                    .unwrap_or(1.0);
                                ui.label(
                                    RichText::new(format!("Series: {series} #{idx}")).color(AMBER),
                                );
                            }
                            ui.add_space(4.0);
                            ui.horizontal_wrapped(|ui| {
                                if ui.button("Read Excerpt").clicked() {
                                    self.spawn_book_excerpt(book_id, 0);
                                }
                                if ui.button("Speak Synopsis (speech-mcp)").clicked() {
                                    let text = format!("{title}. {synopsis}");
                                    self.spawn(move || async move {
                                        let res = dispatch(Command::BridgeSpeak {
                                            text,
                                            voice: None,
                                            url: None,
                                        })
                                        .await;
                                        Job::StatusNote(
                                            "Sent synopsis to speech-mcp".to_string(),
                                            res,
                                        )
                                    });
                                }
                                if ui.button("Ask Gemma 4").clicked() {
                                    self.chat_prompt = format!(
                                        "Tell me about the book '{title}' and its main themes: {synopsis}"
                                    );
                                    self.page = Page::Chat;
                                }
                            });
                        });
                    });
                    ui.separator();
                    ui.label(RichText::new("Synopsis / Comments:").strong());
                    egui::ScrollArea::vertical()
                        .id_salt("book_synopsis_scroll")
                        .max_height(180.0)
                        .show(ui, |ui| {
                            ui.label(synopsis);
                        });
                    if let Some(ex) = self
                        .excerpt_val
                        .get("text")
                        .and_then(Value::as_str)
                        .map(ToString::to_string)
                    {
                        let ex_src = self
                            .excerpt_val
                            .get("source")
                            .and_then(Value::as_str)
                            .unwrap_or("books_text")
                            .to_string();
                        let total_c = self
                            .excerpt_val
                            .get("total_chars")
                            .and_then(Value::as_u64)
                            .unwrap_or(0);
                        ui.separator();
                        ui.horizontal(|ui| {
                            ui.label(
                                RichText::new(format!("Book Excerpt ({ex_src} · {total_c} chars)"))
                                    .strong()
                                    .color(EMERALD),
                            );
                            if self.excerpt_offset >= 2400 && ui.button("<< Prev Page").clicked() {
                                self.spawn_book_excerpt(
                                    book_id,
                                    self.excerpt_offset.saturating_sub(2400),
                                );
                            }
                            if ui.button("Next Page >>").clicked() {
                                self.spawn_book_excerpt(book_id, self.excerpt_offset + 2400);
                            }
                        });
                        egui::ScrollArea::vertical()
                            .id_salt("book_excerpt_scroll")
                            .max_height(190.0)
                            .show(ui, |ui| {
                                ui.label(RichText::new(&ex).font(FontId::monospace(12.0)));
                            });
                    }
                    if let Some(files) = detail.get("files").and_then(Value::as_array) {
                        ui.separator();
                        ui.label(RichText::new("Format Files on Disk:").strong());
                        for f in files {
                            let fmt = f.get("format").and_then(Value::as_str).unwrap_or("FMT");
                            let path = f.get("path").and_then(Value::as_str).unwrap_or("");
                            let sz = f.get("size_bytes").and_then(Value::as_i64).unwrap_or(0);
                            ui.label(
                                RichText::new(format!(
                                    "{fmt} ({:.2} MB) · {path}",
                                    (sz as f64) / 1_048_576.0
                                ))
                                .size(11.5)
                                .weak(),
                            );
                        }
                    }
                });
            } else {
                card_frame(&mut cols[1], |ui| {
                    ui.label("Click 'Inspect' on any book on the left to view cover art, full metadata, identifiers, files on disk, synopsis, and full-text excerpt.");
                });
            }
        });
    }

    fn view_fulltext(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.heading("Full-Text Content Search (full-text-search.db)");
            let resp = ui.add(
                egui::TextEdit::singleline(&mut self.fts_query)
                    .hint_text("Search inside book text…")
                    .desired_width(240.0),
            );
            if ui.button("Search Full Text").clicked()
                || (resp.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)))
            {
                self.spawn_fulltext();
            }
        });
        ui.add_space(8.0);

        let hits = self
            .fulltext_val
            .get("hits")
            .and_then(Value::as_array)
            .cloned()
            .unwrap_or_default();

        egui::ScrollArea::vertical().show(ui, |ui| {
            for h in hits {
                let bid = h.get("book_id").and_then(Value::as_i64).unwrap_or(0);
                let title = h.get("title").and_then(Value::as_str).unwrap_or("Book");
                let fmt = h.get("format").and_then(Value::as_str).unwrap_or("EPUB");
                let snippet = h.get("snippet").and_then(Value::as_str).unwrap_or("");

                card_frame(ui, |ui| {
                    ui.horizontal(|ui| {
                        ui.label(
                            RichText::new(format!("#{bid} · {title}"))
                                .strong()
                                .color(ACCENT),
                        );
                        ui.label(RichText::new(fmt).color(EMERALD).size(11.5));
                        if ui.button("Open in Book Inspector").clicked() {
                            self.spawn_book_detail(bid);
                            self.page = Page::Books;
                        }
                    });
                    ui.label(RichText::new(snippet).size(12.0));
                });
                ui.add_space(6.0);
            }
        });
    }

    fn view_analytics(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.heading("Library Health, Reading Statistics & Unread Priority");
            if ui.button("Run Audit").clicked() {
                self.spawn_analytics();
            }
        });
        ui.add_space(8.0);

        let total = self
            .health_val
            .get("total_books")
            .and_then(Value::as_i64)
            .unwrap_or(0);
        let cover_pct = self
            .health_val
            .get("cover_coverage_pct")
            .and_then(Value::as_f64)
            .unwrap_or(100.0);
        let comment_pct = self
            .health_val
            .get("comment_coverage_pct")
            .and_then(Value::as_f64)
            .unwrap_or(100.0);
        let dups = self
            .health_val
            .get("duplicate_title_groups")
            .and_then(Value::as_u64)
            .unwrap_or(0);

        ui.horizontal_wrapped(|ui| {
            hero_kpi(ui, "Total Books", &total.to_string(), ACCENT);
            hero_kpi(ui, "Cover Coverage", &format!("{cover_pct:.1}%"), EMERALD);
            hero_kpi(
                ui,
                "Synopsis Coverage",
                &format!("{comment_pct:.1}%"),
                AMBER,
            );
            hero_kpi(ui, "Duplicate Title Groups", &dups.to_string(), ROSE);
        });
        ui.add_space(10.0);

        let decades_bars: Vec<(String, f64)> = self
            .reading_stats_val
            .get("decades")
            .and_then(Value::as_array)
            .map(|arr| {
                arr.iter()
                    .take(10)
                    .map(|d| {
                        (
                            d.get("decade")
                                .and_then(Value::as_str)
                                .unwrap_or("Unknown")
                                .to_string(),
                            d.get("book_count").and_then(Value::as_i64).unwrap_or(0) as f64,
                        )
                    })
                    .collect()
            })
            .unwrap_or_default();

        if !decades_bars.is_empty() {
            card_frame(ui, |ui| {
                bar_chart(ui, "Books by Publication Decade", &decades_bars, ACCENT);
            });
            ui.add_space(8.0);
        }

        ui.columns(2, |cols| {
            if let Some(series_arr) = self.series_val.get("series").and_then(Value::as_array) {
                card_frame(&mut cols[0], |ui| {
                    ui.label(RichText::new("Series & Canon Coverage (Gap Detector)").strong());
                    ui.add_space(4.0);
                    egui::ScrollArea::vertical()
                        .id_salt("series_gap_scroll")
                        .max_height(260.0)
                        .show(ui, |ui| {
                            for s in series_arr {
                                let name =
                                    s.get("name").and_then(Value::as_str).unwrap_or("Series");
                                let cnt = s.get("book_count").and_then(Value::as_i64).unwrap_or(0);
                                let missing = s
                                    .get("missing_indices")
                                    .and_then(Value::as_array)
                                    .map(|m| m.len())
                                    .unwrap_or(0);
                                ui.horizontal(|ui| {
                                    ui.label(RichText::new(name).strong());
                                    ui.label(format!("· {cnt} volumes"));
                                    if missing == 0 {
                                        ui.label(RichText::new("· Complete").color(EMERALD));
                                    } else {
                                        ui.label(
                                            RichText::new(format!("· {missing} missing volume(s)"))
                                                .color(AMBER),
                                        );
                                    }
                                });
                            }
                        });
                });
            }

            if let Some(prio_arr) = self
                .priority_val
                .get("priority_queue")
                .and_then(Value::as_array)
                .cloned()
            {
                card_frame(&mut cols[1], |ui| {
                    ui.label(RichText::new("Unread Priority Queue (Top Ranked)").strong());
                    ui.add_space(4.0);
                    egui::ScrollArea::vertical()
                        .id_salt("unread_priority_scroll")
                        .max_height(260.0)
                        .show(ui, |ui| {
                            for p in prio_arr {
                                let bid = p.get("id").and_then(Value::as_i64).unwrap_or(0);
                                let title =
                                    p.get("title").and_then(Value::as_str).unwrap_or("Book");
                                let score =
                                    p.get("priority_score").and_then(Value::as_i64).unwrap_or(0);
                                let reason = p.get("reason").and_then(Value::as_str).unwrap_or("");
                                ui.horizontal(|ui| {
                                    ui.label(
                                        RichText::new(format!("[{score} pts]"))
                                            .color(EMERALD)
                                            .strong(),
                                    );
                                    ui.label(RichText::new(title).strong());
                                    if ui.button("Inspect").clicked() {
                                        self.spawn_book_detail(bid);
                                        self.page = Page::Books;
                                    }
                                });
                                ui.label(RichText::new(reason).size(11.0).weak());
                                ui.add_space(3.0);
                            }
                        });
                });
            }
        });
    }

    fn view_crossconnect(&mut self, ui: &mut egui::Ui) {
        ui.heading("Fleet Crossconnect (speech-mcp & aiwatcher)");
        ui.add_space(8.0);
        card_frame(ui, |ui| {
            ui.label(RichText::new("Voice Synthesis (speech-mcp :10909)").strong());
            ui.text_edit_singleline(&mut self.speak_text);
            if ui.button("Speak Now").clicked() {
                let text = self.speak_text.clone();
                let url = if self.speech_url.trim().is_empty() {
                    None
                } else {
                    Some(self.speech_url.trim().to_string())
                };
                self.spawn(move || async move {
                    let res = dispatch(Command::BridgeSpeak {
                        text,
                        voice: None,
                        url,
                    })
                    .await;
                    Job::StatusNote("bridge.speak dispatched".to_string(), res)
                });
            }
        });
        ui.add_space(8.0);
        card_frame(ui, |ui| {
            ui.label(RichText::new("Webhook / Event Push (aiwatcher :10946)").strong());
            ui.text_edit_singleline(&mut self.webhook_url);
            if ui.button("Send Library Heartbeat").clicked() {
                let url = self.webhook_url.clone();
                let lib = self.active_library.clone();
                self.spawn(move || async move {
                    let res = dispatch(Command::BridgeEvent {
                        kind: "calibre.heartbeat".to_string(),
                        device: "rust-calibre-mcp".to_string(),
                        payload: json!({"active_library": lib}),
                        url: Some(url),
                    })
                    .await;
                    Job::StatusNote("bridge.event dispatched".to_string(), res)
                });
            }
        });
    }

    fn view_chat(&mut self, ui: &mut egui::Ui) {
        ui.heading("Library AI Assistant (Default: gemma4:12b on RTX 4090)");
        ui.add_space(6.0);
        card_frame(ui, |ui| {
            ui.horizontal(|ui| {
                ui.label("Provider:");
                ui.text_edit_singleline(&mut self.chat_provider);
                ui.label("Model:");
                ui.text_edit_singleline(&mut self.chat_model);
            });
            ui.add_space(4.0);
            ui.label("Prompt:");
            ui.text_edit_multiline(&mut self.chat_prompt);
            if ui.button("Send with Active Library Context").clicked() {
                let provider = self.chat_provider.clone();
                let model = Some(self.chat_model.clone());
                let prompt = self.chat_prompt.clone();
                let ctx_val = Some(self.books_val.clone());
                let base_url = if self.llm_base_url.trim().is_empty() {
                    None
                } else {
                    Some(self.llm_base_url.trim().to_string())
                };
                let api_key = if self.llm_api_key.trim().is_empty() {
                    None
                } else {
                    Some(self.llm_api_key.trim().to_string())
                };
                self.status_line = "Querying LLM…".to_string();
                self.spawn(move || async move {
                    Job::ChatReply(
                        dispatch(Command::LlmChat {
                            provider,
                            model,
                            prompt,
                            system: Some(
                                "You are a knowledgeable librarian assistant for a Calibre e-book collection."
                                    .to_string(),
                            ),
                            context: ctx_val,
                            base_url,
                            api_key,
                        })
                        .await,
                    )
                });
            }
            if !self.chat_reply.is_empty() {
                ui.separator();
                ui.label(RichText::new(&self.chat_reply));
            }
        });
    }

    fn view_settings(&mut self, ui: &mut egui::Ui) {
        if !self.vault_loaded {
            self.vault_loaded = true;
            self.spawn(|| async move {
                let url = tokio::task::spawn_blocking(|| {
                    crate::vault::get("calibre_server_url")
                        .unwrap_or_default()
                        .unwrap_or_default()
                })
                .await
                .unwrap_or_default();
                let pw = tokio::task::spawn_blocking(|| {
                    crate::vault::get("calibre_password")
                        .unwrap_or_default()
                        .unwrap_or_default()
                })
                .await
                .unwrap_or_default();
                let key = tokio::task::spawn_blocking(|| {
                    crate::vault::get("llm_api_key")
                        .unwrap_or_default()
                        .unwrap_or_default()
                })
                .await
                .unwrap_or_default();
                Job::VaultLoaded {
                    calibre_server_url: url,
                    calibre_password: pw,
                    llm_api_key: key,
                }
            });
        }

        egui::ScrollArea::vertical().show(ui, |ui| {
            ui.heading("Settings · Calibre & Fleet Hardware Configuration");
            ui.label(RichText::new(&self.vault_status_note).weak());
            ui.add_space(8.0);

            // Card 1: Calibre Library Registry & Paths
            card_frame(ui, |ui| {
                ui.heading("Calibre Library Registry & Active Path");
                ui.label("Manage registered libraries, add custom external directories, and switch active collections.");
                ui.add_space(6.0);

                ui.horizontal(|ui| {
                    ui.label("Active Library:");
                    let active_display = if self.active_library.is_empty() {
                        "(none selected)".to_string()
                    } else {
                        self.active_library.clone()
                    };
                    ui.monospace(&active_display);
                });

                ui.add_space(6.0);
                ui.horizontal(|ui| {
                    if ui.button("Pin Active to Registry").clicked() && !self.active_library.is_empty() {
                        let active = self.active_library.clone();
                        self.pinned_registry = crate::registry::pin_active(&active, &active);
                        self.status_line = format!(
                            "Pinned library to registry ({} total)",
                            self.pinned_registry.libraries.len()
                        );
                    }
                    if ui.button("Rescan Discovered Libraries").clicked() {
                        self.spawn_libraries();
                    }
                });

                ui.separator();
                ui.label(RichText::new("Add Custom / External Library Path").strong());
                ui.horizontal(|ui| {
                    ui.label("Name:");
                    ui.text_edit_singleline(&mut self.custom_library_name);
                    ui.label("Directory Path:");
                    ui.add_sized(
                        [260.0, 20.0],
                        egui::TextEdit::singleline(&mut self.custom_library_path),
                    );
                    if ui.button("Add & Pin").clicked() {
                        let path = self.custom_library_path.trim().to_string();
                        if !path.is_empty() {
                            let name = if self.custom_library_name.trim().is_empty() {
                                path.clone()
                            } else {
                                self.custom_library_name.trim().to_string()
                            };
                            self.pinned_registry = crate::registry::pin_active(&name, &path);
                            self.custom_library_name.clear();
                            self.custom_library_path.clear();
                            self.status_line = format!("Added '{name}' to pinned registry");
                        }
                    }
                });

                ui.separator();
                ui.label(RichText::new("Pinned / Registered Libraries").strong());
                if self.pinned_registry.libraries.is_empty() {
                    ui.label("No pinned libraries yet. Pin your active library or add a custom path above.");
                } else {
                    let mut remove_name = None;
                    let mut switch_item = None;
                    for lib in &self.pinned_registry.libraries {
                        ui.horizontal(|ui| {
                            let is_current = lib.path == self.active_library;
                            if is_current {
                                status_pill(ui, true, "ACTIVE");
                            }
                            ui.label(RichText::new(&lib.name).strong());
                            ui.monospace(&lib.path);
                            if !is_current && ui.button("Switch").clicked() {
                                switch_item = Some((lib.name.clone(), lib.path.clone()));
                            }
                            if ui.button("Unpin").clicked() {
                                remove_name = Some(lib.name.clone());
                            }
                        });
                    }
                    if let Some((name, path)) = switch_item {
                        self.active_library = path;
                        self.spawn_query_books();
                        self.spawn_analytics();
                        self.status_line = format!("Switched active library to '{name}'");
                    }
                    if let Some(name) = remove_name {
                        self.pinned_registry = crate::registry::unpin(&name);
                        self.status_line = format!("Unpinned library '{name}'");
                    }
                }
            });

            ui.add_space(8.0);

            // Card 2: Local AI & Ollama Copilot Configuration
            card_frame(ui, |ui| {
                ui.heading("Local AI Copilot (Ollama / Local LLM)");
                ui.label("Configure your local offline neural librarian. Zero telemetry, 100% private.");
                ui.add_space(6.0);

                ui.horizontal(|ui| {
                    ui.label("Provider:");
                    ui.text_edit_singleline(&mut self.chat_provider);
                    ui.label("Model:");
                    ui.text_edit_singleline(&mut self.chat_model);
                });
                ui.horizontal(|ui| {
                    ui.label("Ollama Endpoint:");
                    ui.text_edit_singleline(&mut self.llm_base_url);
                });

                ui.add_space(6.0);
                ui.horizontal(|ui| {
                    if ui.button("Test Ollama Connection").clicked() {
                        let provider = self.chat_provider.clone();
                        let model = Some(self.chat_model.clone());
                        let base_url = if self.llm_base_url.trim().is_empty() {
                            None
                        } else {
                            Some(self.llm_base_url.trim().to_string())
                        };
                        let api_key = if self.llm_api_key.trim().is_empty() {
                            None
                        } else {
                            Some(self.llm_api_key.trim().to_string())
                        };
                        self.status_line = "Testing LLM connection…".to_string();
                        self.spawn(move || async move {
                            let res = dispatch(Command::LlmChat {
                                provider,
                                model,
                                prompt: "Respond with: ONLINE".to_string(),
                                system: Some("Test responder".to_string()),
                                context: None,
                                base_url,
                                api_key,
                            })
                            .await;
                            Job::StatusNote("Ollama test completed".to_string(), res)
                        });
                    }
                    if ui.button("Query Available Models").clicked() {
                        let provider = self.chat_provider.clone();
                        let base_url = if self.llm_base_url.trim().is_empty() {
                            None
                        } else {
                            Some(self.llm_base_url.trim().to_string())
                        };
                        let api_key = if self.llm_api_key.trim().is_empty() {
                            None
                        } else {
                            Some(self.llm_api_key.trim().to_string())
                        };
                        self.status_line = "Querying available models…".to_string();
                        self.spawn(move || async move {
                            let res = dispatch(Command::LlmModels {
                                provider,
                                base_url,
                                api_key,
                            })
                            .await;
                            Job::StatusNote("Models catalog fetched".to_string(), res)
                        });
                    }
                });
            });

            ui.add_space(8.0);

            // Card 3: Windows Credential Manager Vault
            card_frame(ui, |ui| {
                ui.heading("Windows Credential Manager Vault");
                ui.label("Credentials are stored in OS Credential Manager under 'rust-calibre-mcp'. Secrets never touch disk in plain text.");
                ui.add_space(6.0);

                ui.label("Calibre Content Server URL (Optional):");
                ui.text_edit_singleline(&mut self.calibre_server_url);

                ui.label("Calibre Content Server Password:");
                ui.add(egui::TextEdit::singleline(&mut self.calibre_password).password(true));

                ui.label("Cloud LLM API Key (Optional, for remote providers):");
                ui.add(egui::TextEdit::singleline(&mut self.llm_api_key).password(true));

                ui.add_space(6.0);
                ui.horizontal(|ui| {
                    if ui.button("Save to Windows Credential Manager").clicked() {
                        let url = self.calibre_server_url.clone();
                        let pw = self.calibre_password.clone();
                        let key = self.llm_api_key.clone();
                        self.spawn(move || async move {
                            let _ = tokio::task::spawn_blocking(move || {
                                let _ = crate::vault::set("calibre_server_url", &url);
                                if !pw.is_empty() {
                                    let _ = crate::vault::set("calibre_password", &pw);
                                }
                                if !key.is_empty() {
                                    let _ = crate::vault::set("llm_api_key", &key);
                                }
                            })
                            .await;
                            Job::StatusNote(
                                "Saved credentials to Windows Credential Manager".to_string(),
                                json!({"saved": true}),
                            )
                        });
                    }

                    if ui.button("Clear Vault Credentials").clicked() {
                        self.calibre_password.clear();
                        self.llm_api_key.clear();
                        self.spawn(|| async move {
                            let _ = tokio::task::spawn_blocking(|| {
                                let _ = crate::vault::delete("calibre_server_url");
                                let _ = crate::vault::delete("calibre_password");
                                let _ = crate::vault::delete("llm_api_key");
                            })
                            .await;
                            Job::StatusNote(
                                "Cleared credentials from Windows Credential Manager".to_string(),
                                json!({"cleared": true}),
                            )
                        });
                    }
                });
            });

            ui.add_space(8.0);

            // Card 4: Fleet Crossconnect Endpoints
            card_frame(ui, |ui| {
                ui.heading("Fleet Crossconnect Endpoints");
                ui.label("Direct inter-tool communication with speech-mcp and aiwatcher.");
                ui.add_space(6.0);

                ui.label("speech-mcp TTS Endpoint:");
                ui.text_edit_singleline(&mut self.speech_url);

                ui.label("aiwatcher Webhook Endpoint:");
                ui.text_edit_singleline(&mut self.webhook_url);

                ui.add_space(6.0);
                ui.horizontal(|ui| {
                    if ui.button("Test TTS Voice Output").clicked() {
                        let text = self.speak_text.clone();
                        let url = if self.speech_url.trim().is_empty() {
                            None
                        } else {
                            Some(self.speech_url.trim().to_string())
                        };
                        self.spawn(move || async move {
                            let res = dispatch(Command::BridgeSpeak {
                                text,
                                voice: None,
                                url,
                            })
                            .await;
                            Job::StatusNote("speech-mcp ping dispatched".to_string(), res)
                        });
                    }

                    if ui.button("Send Heartbeat Webhook").clicked() {
                        let url = self.webhook_url.clone();
                        let lib = self.active_library.clone();
                        self.spawn(move || async move {
                            let res = dispatch(Command::BridgeEvent {
                                kind: "calibre.heartbeat".to_string(),
                                device: "rust-calibre-mcp".to_string(),
                                payload: json!({"active_library": lib}),
                                url: Some(url),
                            })
                            .await;
                            Job::StatusNote("aiwatcher heartbeat sent".to_string(), res)
                        });
                    }
                });
            });

            ui.add_space(8.0);

            // Card 5: Engine Diagnostics & Display Preferences
            card_frame(ui, |ui| {
                ui.heading("Engine Diagnostics & Safety Guarantees");
                ui.add_space(6.0);

                ui.horizontal(|ui| {
                    status_pill(ui, true, "SQLITE READ-ONLY SAFE");
                    ui.label("SQLITE_OPEN_READ_ONLY | SQLITE_OPEN_NO_MUTEX + 5000 ms busy_timeout");
                });
                ui.label("Zero write-lock contention. Calibre desktop GUI and background jobs can read/write without crashes or corruptions.");

                ui.separator();
                ui.horizontal(|ui| {
                    let zoom_pct = (self.zoom * 100.0).round() as i64;
                    ui.label(format!("UI Scale: {zoom_pct}% (Ctrl + Mouse Wheel to zoom)"));
                    if ui.button("Reset Zoom to 100%").clicked() {
                        self.zoom = 1.0;
                        ui.ctx().set_zoom_factor(1.0);
                    }
                });

                ui.horizontal(|ui| {
                    let theme_label = match self.theme {
                        Theme::Dark => "Harsh Dark (ArtCraft default)",
                        Theme::Light => "Crisp Light",
                    };
                    ui.label(format!("Active Theme: {theme_label}"));
                    if ui.button("Switch Theme").clicked() {
                        self.theme = match self.theme {
                            Theme::Dark => Theme::Light,
                            Theme::Light => Theme::Dark,
                        };
                        apply_theme(ui.ctx(), self.theme);
                    }
                });

                ui.separator();
                ui.label(RichText::new("Runtime & Architecture").strong());
                ui.label("· rust-calibre-mcp v0.4.0 · 64-bit Windows Native Binary");
                ui.label("· GUI: eframe 0.31 / egui (pure Rust, zero Chromium / Electron)");
                ui.label("· Database Engine: rusqlite with bundled SQLite 3.49 (FTS5 enabled)");
                ui.label("· MCP Transport: FastMCP 3.1+ Stdio JSON-RPC 2.0");
            });
        });
    }

    fn view_help(&mut self, ui: &mut egui::Ui) {
        ui.heading("Help · Pure-Rust Calibre ArtCraft Desktop App");
        ui.add_space(8.0);
        card_frame(ui, |ui| {
            ui.label(RichText::new("Keyboard & Mouse Shortcuts").strong());
            ui.label("· Ctrl + Mouse Wheel: Smooth UI zoom (50% to 200%)");
            ui.label(
                "· Topbar Sun / Moon button: Switch between Harsh Dark and Crisp Light themes",
            );
            ui.label("· SQLite Contention Safety: Opens metadata.db and full-text-search.db strictly with SQLITE_OPEN_READ_ONLY | SQLITE_OPEN_NO_MUTEX + 5000 ms busy_timeout.");
        });
    }

    fn view_raw(&mut self, ui: &mut egui::Ui) {
        ui.heading("Raw JSON Inspector (Last L1 Command Output)");
        let pretty =
            serde_json::to_string_pretty(&self.last_raw).unwrap_or_else(|_| "{}".to_string());
        egui::ScrollArea::vertical().show(ui, |ui| {
            ui.add(
                egui::TextEdit::multiline(&mut pretty.as_str())
                    .font(FontId::monospace(12.0))
                    .desired_width(f32::INFINITY),
            );
        });
    }
}

impl eframe::App for CalibreApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        let ctx = ui.ctx().clone();
        self.drain_jobs(&ctx);
        if self.pending > 0 {
            ctx.request_repaint();
        }

        // Safe Ctrl+Wheel zoom (read delta inside ctx.input, apply outside!)
        let zoom_delta = ctx.input(|i| i.zoom_delta());
        if (zoom_delta - 1.0).abs() > 0.001 {
            self.zoom = (self.zoom * zoom_delta).clamp(0.5, 2.0);
            ctx.set_zoom_factor(self.zoom);
        }

        let mut toggle_theme = false;
        let mut reset_zoom = false;

        egui::Panel::top("calibre_topbar").show(ui, |ui| {
            ui.horizontal(|ui| {
                if ui
                    .button(if self.sidebar_open { "<<" } else { ">>" })
                    .clicked()
                {
                    self.sidebar_open = !self.sidebar_open;
                }
                ui.label(RichText::new("CALIBRE · ARTCRAFT").strong().color(ACCENT));
                ui.separator();
                status_pill(
                    ui,
                    self.pending == 0,
                    if self.pending == 0 { "IDLE" } else { "BUSY" },
                );
                ui.label(RichText::new(&self.status_line).size(12.0).weak());

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if paint_theme_button(ui, self.theme).clicked() {
                        toggle_theme = true;
                    }
                    if (self.zoom - 1.0).abs() > 0.01
                        && ui
                            .button(format!("{}%", (self.zoom * 100.0).round() as i32))
                            .on_hover_text("Click to reset zoom to 100%")
                            .clicked()
                    {
                        reset_zoom = true;
                    }
                });
            });
        });

        if toggle_theme {
            self.theme = match self.theme {
                Theme::Dark => Theme::Light,
                Theme::Light => Theme::Dark,
            };
            apply_theme(&ctx, self.theme);
        }
        if reset_zoom {
            self.zoom = 1.0;
            ctx.set_zoom_factor(1.0);
        }

        if self.sidebar_open {
            egui::Panel::left("calibre_sidebar")
                .default_size(185.0)
                .show(ui, |ui| {
                    ui.add_space(6.0);
                    ui.label(RichText::new("LIBRARY").size(11.0).weak());
                    for (p, label) in [
                        (Page::Libraries, "Libraries"),
                        (Page::Books, "Books & Inspector"),
                        (Page::Fulltext, "Full-Text Search"),
                        (Page::Analytics, "Health & Series"),
                    ] {
                        if ui.selectable_label(self.page == p, label).clicked() {
                            self.page = p;
                        }
                    }
                    ui.separator();
                    ui.label(RichText::new("FLEET & AI").size(11.0).weak());
                    for (p, label) in [
                        (Page::Chat, "AI Librarian (Gemma 4)"),
                        (Page::Crossconnect, "Crossconnect"),
                        (Page::Settings, "Settings & Vault"),
                        (Page::Help, "Help"),
                        (Page::RawJson, "Raw JSON"),
                    ] {
                        if ui.selectable_label(self.page == p, label).clicked() {
                            self.page = p;
                        }
                    }
                });
        }

        egui::CentralPanel::default().show(ui, |ui| match self.page {
            Page::Libraries => self.view_libraries(ui),
            Page::Books => self.view_books(ui),
            Page::Fulltext => self.view_fulltext(ui),
            Page::Analytics => self.view_analytics(ui),
            Page::Crossconnect => self.view_crossconnect(ui),
            Page::Chat => self.view_chat(ui),
            Page::Settings => self.view_settings(ui),
            Page::Help => self.view_help(ui),
            Page::RawJson => self.view_raw(ui),
        });
    }
}

/// Headless CI smoke check (`cargo run -p calibre-app -- --smoke`).
pub fn smoke() -> eframe::Result<()> {
    let rt = match tokio::runtime::Runtime::new() {
        Ok(r) => r,
        Err(e) => {
            eprintln!("smoke: runtime init failed: {e}");
            std::process::exit(1);
        }
    };
    let (status, demo_books, demo_health) = rt.block_on(async {
        let s = dispatch(Command::Status).await;
        let b = dispatch(Command::QueryBooks(BookQueryParams {
            library: Some("demo".to_string()),
            ..Default::default()
        }))
        .await;
        let h = dispatch(Command::LibraryHealth {
            library: Some("demo".to_string()),
        })
        .await;
        (s, b, h)
    });
    let ok = status.get("status").and_then(Value::as_str) == Some("ok")
        && demo_books.get("count").and_then(Value::as_u64) == Some(6)
        && demo_health.get("total_books").and_then(Value::as_i64) == Some(6);
    if ok {
        println!("smoke: status + demo query_books (6) + demo library_health OK");
        Ok(())
    } else {
        eprintln!("smoke: parity check failed");
        std::process::exit(1);
    }
}

#[cfg(test)]
mod parity_tests {
    use super::*;
    use egui_kittest::Harness;

    #[test]
    fn glyph_allowlist() {
        let mut harness = Harness::new_ui(|ui| {
            ui.label("4.20 MB · 3 × 2 · querying…");
        });
        harness.run();
    }

    #[test]
    fn glyph_source_scan() {
        const ALLOWED: &[char] = &['°', '×', '·', '…'];
        let source = include_str!("app.rs");
        let mut offenders = Vec::new();
        for (idx, ch) in source.char_indices() {
            if !ch.is_ascii() && !ALLOWED.contains(&ch) {
                offenders.push((idx, ch));
                if offenders.len() >= 8 {
                    break;
                }
            }
        }
        assert!(
            offenders.is_empty(),
            "non-allowlisted glyphs in app.rs: {offenders:?}"
        );
    }

    #[test]
    fn kpi_and_bar_chart_render_cleanly() {
        let mut harness = Harness::new_ui(|ui| {
            hero_kpi(ui, "Discovered Libraries", "2", ACCENT);
            bar_chart(
                ui,
                "Books per Library",
                &[
                    ("Primary-Library".to_string(), 500.0),
                    ("Tech-Library".to_string(), 250.0),
                ],
                EMERALD,
            );
        });
        harness.run();
    }
}
