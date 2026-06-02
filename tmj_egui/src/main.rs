use std::time::{self, Duration};

use anyhow::Context;
use chrono::{FixedOffset, Utc};
use eframe::egui::{self, ColorImage, TextureOptions, ViewportBuilder, ViewportCommand};
use ratatui::Terminal;
use soft_ratatui::{CosmicText, SoftBackend};
use tmj_app::app::App;
use tmj_app::setting::SETTING;
use tmj_core::command::CmdBuffer;
use tmj_core::event::handler::EventDispatcher;
use tmj_core::event::looper::EventLooper;
use tmj_core::event::provider::{NoopProvider, convert_crossterm_event};
use tmj_core::event::sender::EventSender;
use tmj_core::event::EventManager;
use tmj_core::pathes;
use tmj_core::pathes::PathResolver;
use tracing_subscriber::fmt::format::Writer;
use tracing_subscriber::fmt::time::FormatTime;

struct ChinaLocalTime;
impl FormatTime for ChinaLocalTime {
    fn format_time(&self, w: &mut Writer<'_>) -> std::fmt::Result {
        let tz = FixedOffset::east_opt(8 * 3600).unwrap();
        let now = Utc::now().with_timezone(&tz);
        write!(w, "{}", now.format("%m-%d %H:%M:%S%.3f"))
    }
}

fn screen_size() -> (f32, f32) {
    if let Ok(displays) = display_info::DisplayInfo::all() {
        if let Some(d) = displays.iter().find(|d| d.is_primary).or(displays.first()) {
            return (d.width as f32, d.height as f32);
        }
    }
    (1920.0, 1080.0)
}

fn main() -> eframe::Result {
    PathResolver::global_init();
    let writer_path = pathes::path("log.txt");
    let _ = std::fs::OpenOptions::new().create(true).write(true).truncate(true).open(&writer_path);
    tracing_subscriber::fmt()
        .with_timer(ChinaLocalTime)
        .with_writer(move || {
            std::fs::OpenOptions::new().create(true).append(true).open(&writer_path).expect("open log")
        })
        .init();

    let (looper, receiver) = EventLooper::new_with_provider(256, Box::new(NoopProvider));
    EventSender::init(looper.sender.clone());
    EventManager::init(looper);
    EventManager::with_looper(|l| l.cool_down(Duration::from_millis(100)));

    let font_path = pathes::path(&SETTING.font);
    let font_data = std::fs::read(&font_path)
        .unwrap_or_else(|_| panic!("font not found: {}", font_path.display()));

    let (scr_w, scr_h) = screen_size();
    let raw = scr_h / 67.0;
    let cell_h = (((raw / 2.0).floor() * 2.0) as u32).max(16).min(18);
    let cell_w = cell_h / 2;
    let font_size = cell_h;

    let mut backend = SoftBackend::<CosmicText>::new(240, 67, font_size as i32, &font_data);
    backend.char_width = cell_w as usize;
    backend.char_height = cell_h as usize;
    backend.resize(240, 67);

    let area = ratatui::layout::Rect::new(0, 0, 240, 67);
    backend.buffer.set_style(area, ratatui::style::Style::new().bg(ratatui::style::Color::Black));
    backend.redraw();

    let pix_w = backend.get_pixmap_width() as f32;
    let pix_h = backend.get_pixmap_height() as f32;

    tracing::info!("screen {scr_w}x{scr_h} cell {cell_w}x{cell_h} font_size {font_size} pixmap {pix_w}x{pix_h}");

    let terminal = Terminal::new(backend).unwrap();
    let mut app = App::new(terminal);
    let mut last_tick = time::Instant::now();

    let options = eframe::NativeOptions {
        viewport: ViewportBuilder::default().with_fullscreen(true).with_decorations(false),
        ..Default::default()
    };

    eframe::run_ui_native("TUI", options, move |ctx, _frame| {
        let tick = last_tick.elapsed();
        last_tick = time::Instant::now();

        let events = ctx.input(|i| i.events.clone());
        for ev in &events {
            if let Ok(t) = terminput_egui::to_terminput(ev.clone()) {
                if let Ok(c) = terminput_crossterm::to_crossterm(t) {
                    let _ = EventSender::sender_event(convert_crossterm_event(c));
                }
            }
        }

        EventManager::with_looper(|l| {
            if !l.check_is_warmup() { l.drain_buffer(&receiver); }
        });

        {
            let mut game = app.game.borrow_mut();
            while let Ok(event) = receiver.try_recv() {
                if !game.handle_event(&event).context("event").is_ok_and(|v| v) { return; }
            }
            game.handle_tick(tick);
            for cmd in CmdBuffer::take_commands() {
                let _ = game.handle_cmd(&cmd);
            }
            app.terminal.draw(|f| game.draw(f)).ok();
        }

        ctx.set_visuals(egui::Visuals::dark());
        egui::CentralPanel::default().frame(egui::Frame {
            fill: egui::Color32::TRANSPARENT,
            inner_margin: egui::Margin::default(),
            outer_margin: egui::Margin::default(),
            shadow: egui::Shadow::NONE,
            stroke: egui::Stroke::NONE,
            ..Default::default()
        }).show(ctx, |ui| {
            let backend = app.terminal.backend();
            let img = ColorImage::from_rgb(
                [backend.get_pixmap_width(), backend.get_pixmap_height()],
                backend.get_pixmap_data(),
            );
            let tex = ui.ctx().load_texture("term", img, TextureOptions::NEAREST);
            let avail = ui.available_size();
            let pix_w = backend.get_pixmap_width() as f32;
            let pix_h = backend.get_pixmap_height() as f32;
            let off_x = ((avail.x - pix_w) / 2.0).max(0.0);
            let off_y = ((avail.y - pix_h) / 2.0).max(0.0);
            ui.painter().rect_filled(ui.max_rect(), 0.0, egui::Color32::from_gray(0x1E));
            let mut mesh = egui::Mesh::with_texture(tex.id());
            mesh.add_rect_with_uv(
                egui::Rect::from_min_size(egui::pos2(off_x, off_y), egui::vec2(pix_w, pix_h)),
                egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0)),
                egui::Color32::WHITE,
            );
            ui.painter().add(mesh);
        });

        let frame_budget = Duration::from_millis(16);
        let remaining = frame_budget.saturating_sub(last_tick.elapsed());
        ctx.request_repaint_after(remaining);

        if app.game.borrow().game_flow.borrow().is_ready_quit() {
            ctx.send_viewport_cmd(ViewportCommand::Close);
        }
    })
}
