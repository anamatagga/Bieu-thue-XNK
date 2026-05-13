use egui::{
    Align, Frame, Key, Layout, Margin, Rounding, ScrollArea, Stroke, Vec2,
};
use egui_extras::{Column, TableBuilder};

use crate::data::HsEntry;
use crate::theme::{rate_color, C};

// ── View mode ────────────────────────────────────────────────────────────────
#[derive(PartialEq, Clone, Copy)]
pub enum View { Basic, Fta, Export, All }

impl View {
    fn tabs() -> &'static [(View, &'static str)] {
        &[
            (View::Basic,  "⚡  Cơ bản"),
            (View::Fta,    "🌏  FTA đầy đủ"),
            (View::Export, "📤  Xuất khẩu"),
            (View::All,    "📋  Tất cả"),
        ]
    }
}

// ── Grouped search result ────────────────────────────────────────────────────
struct Group<'a> {
    code4:   String,
    chapter: String,
    header:  Option<&'a HsEntry>,
    rows:    Vec<&'a HsEntry>,
}

// ── App state ────────────────────────────────────────────────────────────────
pub struct BtApp {
    conn:      Option<rusqlite::Connection>,
    query:     String,
    committed: String,
    results:   Vec<HsEntry>,
    view:      View,
    status:    String,
    loaded:    bool,
    total_db:  usize,
}

impl BtApp {
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        Self {
            conn:      None,
            query:     String::new(),
            committed: String::new(),
            results:   Vec::new(),
            view:      View::Basic,
            status:    "Đang khởi tạo cơ sở dữ liệu (SQLite)...".into(),
            loaded:    false,
            total_db:  0,
        }
    }

    fn ensure_loaded(&mut self) {
        if self.loaded { return; }
        if let Ok(c) = crate::data::init_db() {
            self.total_db = crate::data::get_total_count(&c);
            self.conn = Some(c);
            self.status = format!("✅  {} mã HS  │  Biểu thuế XNK 2026 — SQLite", self.total_db);
        } else {
            self.status = "❌  Lỗi khởi tạo CSDL SQLite!".into();
        }
        self.loaded = true;
    }

    // ── Search ───────────────────────────────────────────────────────────────
    fn search(&mut self) {
        let q = self.query.trim().to_string();
        if q == self.committed { return; }
        self.committed = q.clone();

        if q.is_empty() {
            self.results.clear();
            self.status = format!("📦  {} mã HS  │  Nhập mã HS hoặc tên hàng để tra cứu", self.total_db);
            return;
        }

        self.results.clear();
        if let Some(conn) = &self.conn {
            self.results = crate::data::search_db(conn, &q);
        }

        self.status = if self.results.is_empty() {
            format!("🔎  Không tìm thấy kết quả cho  \"{}\"", q)
        } else {
            format!("🔍  {}  kết quả cho  \"{}\"", self.results.len(), q)
        };
    }

    fn clear(&mut self) {
        self.query.clear();
        self.committed.clear();
        self.results.clear();
        self.status = format!("📦  {} mã HS  │  Nhập mã HS hoặc tên hàng để tra cứu", self.total_db);
    }

    // ── Groups ───────────────────────────────────────────────────────────────
    fn build_groups(&self) -> Vec<Group<'_>> {
        let mut groups: Vec<Group<'_>> = Vec::new();
        for e in &self.results {
            let code4 = e.digits().chars().take(4).collect::<String>();
            if let Some(g) = groups.iter_mut().find(|g| g.code4 == code4) {
                g.rows.push(e);
            } else {
                groups.push(Group {
                    code4:   code4,
                    chapter: e.chapter.clone(),
                    header:  None,
                    rows:    vec![e],
                });
            }
        }
        for g in &mut groups {
            g.header = g.rows.iter().copied()
                .find(|e| e.is_group_header());
        }
        groups.sort_by(|a, b| a.code4.cmp(&b.code4));
        groups
    }

    // ── Column definitions ───────────────────────────────────────────────────
    fn columns(&self) -> Vec<(&'static str, f32)> {
        let mut c = vec![
            ("Mã HS",             90.0_f32),
            ("Mô tả hàng hoá",   220.0),
        ];
        if self.view != View::Basic {
            c.push(("Description (EN)", 180.0));
        }
        c.extend([("Đơn vị", 52.0), ("NK TT", 52.0), ("NK ưu đãi", 60.0), ("VAT", 68.0)]);

        if matches!(self.view, View::Fta | View::All) {
            for n in ["ACFTA","ATIGA","AJCEP","VJEPA","AKFTA","AANZFTA",
                      "AIFTA","VKFTA","VCFTA","VN-EAEU","CPTPP","AHKFTA",
                      "VNCU","EVFTA","UKVFTA","VN-LAO","VN-CAM","VIFTA","RCEPT"] {
                c.push((n, 52.0));
            }
        }
        if matches!(self.view, View::Export | View::All) {
            c.extend([("TT ĐB", 50.0), ("Thuế XK", 58.0), ("BVMT", 50.0)]);
        }
        if self.view == View::All {
            c.extend([("Chính sách mặt hàng", 200.0), ("Giảm VAT", 58.0)]);
        }
        c
    }
}

// ── eframe::App ──────────────────────────────────────────────────────────────
impl eframe::App for BtApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.ensure_loaded();

        // Global keyboard
        if ctx.input(|i| i.key_pressed(Key::Escape)) { self.clear(); }
        if ctx.input(|i| i.modifiers.ctrl && i.key_pressed(Key::F)) {
            ctx.memory_mut(|m| m.request_focus(egui::Id::new("search_input")));
        }

        // ── Header ────────────────────────────────────────────────────────
        egui::TopBottomPanel::top("hdr")
            .frame(Frame::none()
                .fill(C::CRUST)
                .inner_margin(Margin::symmetric(16.0, 11.0)))
            .show(ctx, |ui| { self.ui_header(ui); });

        // ── Search bar ────────────────────────────────────────────────────
        egui::TopBottomPanel::top("search_bar")
            .frame(Frame::none()
                .fill(C::MANTLE)
                .inner_margin(Margin::symmetric(14.0, 10.0)))
            .show(ctx, |ui| {
                if self.ui_search(ui) { self.search(); }
            });

        // ── View tabs ─────────────────────────────────────────────────────
        if !self.results.is_empty() || !self.committed.is_empty() {
            egui::TopBottomPanel::top("tabs")
                .frame(Frame::none()
                    .fill(C::BASE)
                    .stroke(Stroke::new(1.0, C::SURFACE0))
                    .inner_margin(Margin::symmetric(14.0, 7.0)))
                .show(ctx, |ui| { self.ui_tabs(ui); });
        }

        // ── Status bar ────────────────────────────────────────────────────
        egui::TopBottomPanel::bottom("status")
            .frame(Frame::none()
                .fill(C::CRUST)
                .stroke(Stroke::new(1.0, C::SURFACE0))
                .inner_margin(Margin::symmetric(16.0, 8.0)))
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.label(egui::RichText::new("Biểu thuế XNK 2026 v1.0")
                        .size(11.5).color(C::OVERLAY0));
                        
                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                        ui.label(egui::RichText::new("Dữ liệu: Bộ Tài chính Việt Nam")
                            .size(11.5).color(C::OVERLAY0));
                            
                        ui.add_space(20.0);
                        ui.label(egui::RichText::new("Enter = tìm   ·   Esc = xoá   ·   Ctrl+F = focus")
                            .size(11.5).color(C::SUBTEXT0));
                    });
                });
            });

        // ── Main content ──────────────────────────────────────────────────
        egui::CentralPanel::default()
            .frame(Frame::none()
                .fill(C::MANTLE)
                .inner_margin(Margin::symmetric(10.0, 8.0)))
            .show(ctx, |ui| { self.ui_results(ui); });
    }
}

// ── Sub-UI helpers ───────────────────────────────────────────────────────────
impl BtApp {
    fn ui_header(&self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            // Logo pill
            let (rect, _) = ui.allocate_exact_size(Vec2::splat(36.0), egui::Sense::hover());
            ui.painter().circle_filled(rect.center(), 18.0, C::MAUVE);
            ui.painter().text(rect.center(), egui::Align2::CENTER_CENTER, "税",
                egui::FontId::proportional(19.0), C::CRUST);

            ui.add_space(10.0);
            ui.vertical(|ui| {
                ui.label(egui::RichText::new("BIỂU THUẾ XUẤT NHẬP KHẨU 2026")
                    .size(15.5).color(C::TEXT).strong());
                ui.label(egui::RichText::new(&self.status)
                    .size(11.5).color(C::SUBTEXT0));
            });
        });
    }

    fn ui_search(&mut self, ui: &mut egui::Ui) -> bool {
        let mut fire = false;
        ui.horizontal(|ui| {
            ui.label(egui::RichText::new("🔍").size(17.0).color(C::OVERLAY1));

            let te = egui::TextEdit::singleline(&mut self.query)
                .id(egui::Id::new("search_input"))
                .hint_text(" Nhập mã HS (vd: 0101, 01012100) hoặc tên hàng (vd: ngựa, gạo)...")
                .font(egui::FontId::proportional(15.0))
                .min_size(Vec2::new(0.0, 32.0))
                .desired_width(ui.available_width() - 200.0)
                .text_color(C::TEXT);

            let r = ui.add(te);

            // Auto-focus on first frame
            if !self.loaded { r.request_focus(); }

            if r.lost_focus() && ui.input(|i| i.key_pressed(Key::Enter)) {
                fire = true;
            }

            if ui.add(
                egui::Button::new(egui::RichText::new("  Tìm kiếm  ").size(13.0).color(C::CRUST))
                    .fill(C::MAUVE)
                    .rounding(Rounding::same(8.0))
                    .min_size(Vec2::new(100.0, 32.0)),
            ).clicked() { fire = true; }

            if !self.query.is_empty() {
                if ui.add(
                    egui::Button::new(egui::RichText::new("✕").size(13.0).color(C::SUBTEXT0))
                        .fill(C::SURFACE0)
                        .rounding(Rounding::same(8.0))
                        .min_size(Vec2::splat(32.0)),
                ).clicked() { self.clear(); }
            }
        });
        fire
    }

    fn ui_tabs(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.label(egui::RichText::new("Chế độ xem:")
                .size(12.0).color(C::SUBTEXT0));
            ui.add_space(6.0);
            for &(mode, label) in View::tabs() {
                let active = self.view == mode;
                let btn = egui::SelectableLabel::new(active, label);
                if ui.add(btn).clicked() { self.view = mode; }
            }
        });
    }

    fn ui_results(&self, ui: &mut egui::Ui) {
        if self.results.is_empty() {
            self.ui_empty(ui);
            return;
        }
        let groups = self.build_groups();
        ScrollArea::both()
            .auto_shrink([false, false])
            .show(ui, |ui| {
                for g in &groups {
                    self.ui_group(ui, g);
                    ui.add_space(10.0);
                }
                ui.add_space(6.0);
            });
    }

    fn ui_empty(&self, ui: &mut egui::Ui) {
        ui.add_space(50.0);
        ui.vertical_centered(|ui| {
            if self.committed.is_empty() {
                ui.label(egui::RichText::new("📋").size(52.0));
                ui.add_space(10.0);
                ui.label(egui::RichText::new("Tra cứu Biểu thuế Xuất Nhập khẩu 2026")
                    .size(17.0).color(C::SUBTEXT1).strong());
                ui.add_space(6.0);
                ui.label(egui::RichText::new("Nhập mã HS hoặc tên hàng vào thanh tìm kiếm")
                    .size(13.0).color(C::OVERLAY1));
                ui.add_space(20.0);

                Frame::none().fill(C::SURFACE0).rounding(Rounding::same(12.0))
                    .inner_margin(Margin::same(18.0)).show(ui, |ui| {
                    ui.set_max_width(440.0);
                    ui.label(egui::RichText::new("💡  Gợi ý tìm kiếm")
                        .size(13.0).color(C::MAUVE).strong());
                    ui.add_space(8.0);
                    for (k, v) in [
                        ("Mã 4 số :", "0101  →  Toàn bộ nhóm ngựa, lừa, la"),
                        ("Mã 8 số :", "01012100  →  Mã HS đầy đủ"),
                        ("Tên VN  :", "ngựa · gạo · thép · nhựa · cà phê"),
                        ("Tên EN  :", "horse · rice · steel · coffee"),
                    ] {
                        ui.horizontal(|ui| {
                            ui.label(egui::RichText::new(k).size(12.0).color(C::LAVENDER).strong());
                            ui.label(egui::RichText::new(v).size(12.0).color(C::SUBTEXT0));
                        });
                    }
                });
            } else {
                ui.label(egui::RichText::new("🔎").size(44.0));
                ui.add_space(10.0);
                ui.label(egui::RichText::new(
                    format!("Không tìm thấy kết quả cho  \"{}\"", self.committed))
                    .size(14.0).color(C::SUBTEXT0));
                ui.add_space(4.0);
                ui.label(egui::RichText::new("Thử tìm với mã HS ngắn hơn hoặc từ khoá khác")
                    .size(12.0).color(C::OVERLAY1));
            }
        });
    }

    // ── Group card ────────────────────────────────────────────────────────────
    fn ui_group(&self, ui: &mut egui::Ui, g: &Group<'_>) {
        Frame::none()
            .fill(C::BASE)
            .rounding(Rounding::same(12.0))
            .stroke(Stroke::new(1.0, C::SURFACE1))
            .outer_margin(Margin::symmetric(2.0, 0.0))
            .show(ui, |ui| {
                // ── Group header ──────────────────────────────────────────
                Frame::none()
                    .fill(C::SURFACE0)
                    .rounding(Rounding { nw: 12.0, ne: 12.0, sw: 0.0, se: 0.0 })
                    .inner_margin(Margin::symmetric(14.0, 10.0))
                    .show(ui, |ui| {
                        ui.horizontal(|ui| {
                            // Chapter pill
                            Frame::none()
                                .fill(C::MAUVE)
                                .rounding(Rounding::same(6.0))
                                .inner_margin(Margin::symmetric(9.0, 3.0))
                                .show(ui, |ui| {
                                    ui.label(egui::RichText::new(g.chapter_num())
                                        .size(11.0).color(C::CRUST).strong());
                                });

                            ui.add_space(8.0);

                            // 4-digit code
                            ui.label(egui::RichText::new(&g.code4)
                                .size(18.0).color(C::LAVENDER).strong().monospace());

                            ui.add_space(10.0);

                            if let Some(h) = g.header {
                                ui.label(egui::RichText::new(&h.vn)
                                    .size(13.5).color(C::TEXT).strong());
                                if !h.en.is_empty() {
                                    ui.add_space(4.0);
                                    ui.label(egui::RichText::new(format!("/ {}", h.en))
                                        .size(12.0).color(C::SUBTEXT0).italics());
                                }
                            } else {
                                ui.label(egui::RichText::new(g.chapter_name())
                                    .size(12.0).color(C::SUBTEXT0));
                            }

                            // Row count badge (right-aligned)
                            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                                Frame::none()
                                    .fill(C::SURFACE1)
                                    .rounding(Rounding::same(20.0))
                                    .inner_margin(Margin::symmetric(9.0, 3.0))
                                    .show(ui, |ui| {
                                        ui.label(egui::RichText::new(
                                            format!("{} dòng", g.rows.len()))
                                            .size(11.0).color(C::SUBTEXT0));
                                    });
                            });
                        });
                    });

                // ── Table ─────────────────────────────────────────────────
                let cols = self.columns();

                let mut tb = TableBuilder::new(ui)
                    .resizable(true)
                    .striped(false)
                    .cell_layout(Layout::left_to_right(Align::Center));
                    
                for (_, w) in &cols {
                    tb = tb.column(Column::initial(*w).at_least(40.0));
                }

                tb.min_scrolled_height(0.0)
                    .max_scroll_height(f32::INFINITY)
                    .header(26.0, |mut hdr| {
                        for (name, _) in &cols {
                            hdr.col(|ui| {
                                ui.centered_and_justified(|ui| {
                                    ui.label(egui::RichText::new(*name)
                                        .size(11.0).color(C::SUBTEXT0).strong());
                                });
                            });
                        }
                    })
                    .body(|body| {
                        body.rows(36.0, g.rows.len(), |mut row| {
                            let e = g.rows[row.index()];
                            let is_hdr = e.is_group_header();
                            row.set_selected(is_hdr);
                            self.fill_row(&mut row, e, is_hdr);
                        });
                    });
            });
    }

    // ── Fill one table row ────────────────────────────────────────────────────
    fn fill_row(
        &self,
        row: &mut egui_extras::TableRow<'_, '_>,
        e: &HsEntry,
        is_hdr: bool,
    ) {
        let code_col = if is_hdr { C::MAUVE } else { C::LAVENDER };
        let text_col = if is_hdr { C::TEXT  } else { C::SUBTEXT1 };

        // Mã HS
        row.col(|ui| {
            ui.label(egui::RichText::new(e.formatted_code())
                .size(12.5).color(code_col).strong().monospace());
        });
        // Mô tả VN
        row.col(|ui| {
            ui.add(egui::Label::new(
                egui::RichText::new(&e.vn).size(12.5).color(text_col)
            ).wrap());
        });
        // Mô tả EN (optional)
        if self.view != View::Basic {
            row.col(|ui| {
                ui.add(egui::Label::new(
                    egui::RichText::new(&e.en).size(12.0).color(C::SUBTEXT0).italics()
                ).wrap());
            });
        }
        // DVT
        row.col(|ui| {
            ui.centered_and_justified(|ui| {
                ui.label(egui::RichText::new(&e.dvt).size(11.5).color(C::OVERLAY1));
            });
        });
        // NK TT / NK ưu đãi / VAT
        for rate in [&e.nk_tt, &e.nk_ud] {
            row.col(|ui| {
                ui.centered_and_justified(|ui| {
                    ui.label(egui::RichText::new(rate.as_str())
                        .size(12.5).color(rate_color(rate)).strong());
                });
            });
        }
        row.col(|ui| {
            ui.centered_and_justified(|ui| {
                ui.label(egui::RichText::new(&e.vat).size(12.0).color(C::SAPPHIRE));
            });
        });
        // FTA rates
        if matches!(self.view, View::Fta | View::All) {
            for rate in [&e.acfta, &e.atiga, &e.ajcep, &e.vjepa,
                         &e.akfta, &e.aanzfta, &e.aifta, &e.vkfta,
                         &e.vcfta, &e.vneaeu, &e.cptpp, &e.ahkfta,
                         &e.vncu,  &e.evfta,  &e.ukvfta,&e.vnlao,
                         &e.vncam, &e.vifta,  &e.rcept] {
                row.col(|ui| {
                    ui.centered_and_justified(|ui| {
                        ui.label(egui::RichText::new(rate.as_str())
                            .size(12.0).color(rate_color(rate)));
                    });
                });
            }
        }
        // Export rates
        if matches!(self.view, View::Export | View::All) {
            for rate in [&e.ttdb, &e.xk, &e.bvmt] {
                row.col(|ui| {
                    ui.centered_and_justified(|ui| {
                        ui.label(egui::RichText::new(rate.as_str())
                            .size(12.0).color(rate_color(rate)));
                    });
                });
            }
        }
        // Policy + VAT reduction
        if self.view == View::All {
            row.col(|ui| {
                ui.add(egui::Label::new(
                    egui::RichText::new(&e.cshs).size(11.0).color(C::OVERLAY1)
                ).wrap());
            });
            row.col(|ui| {
                ui.centered_and_justified(|ui| {
                    ui.label(egui::RichText::new(&e.giam_vat)
                        .size(12.0).color(C::TEAL));
                });
            });
        }
    }
}

// ── Group helpers ─────────────────────────────────────────────────────────────
impl Group<'_> {
    fn chapter_num(&self) -> &str {
        self.chapter.splitn(2, " - ").next().unwrap_or("").trim()
    }
    fn chapter_name(&self) -> &str {
        self.chapter.splitn(2, " - ").nth(1).unwrap_or("").trim()
    }
}
