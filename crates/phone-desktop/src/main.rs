mod audio;
mod candidate_view;
mod canvas;
mod config;
mod gpu;
mod host_input;
mod media;
mod menu;
mod phone;
mod smoke;
mod snake_art;
mod storage;
mod theme;
mod views;
mod web;
use phone_core::input::{Event, Input};
use std::{
    path::PathBuf,
    sync::Arc,
    time::{Duration, Instant},
};
use winit::{
    application::ApplicationHandler,
    dpi::LogicalSize,
    event::{ElementState, Ime, WindowEvent},
    event_loop::{ActiveEventLoop, ControlFlow, EventLoop},
    keyboard::{KeyCode, ModifiersState, PhysicalKey},
    window::{Window, WindowId},
};
struct App {
    window: Option<Arc<Window>>,
    gpu: Option<gpu::Gpu>,
    canvas: canvas::Canvas,
    phone: phone::Phone,
    input: Input,
    web: web::Web,
    media: Option<media::Media>,
    modifiers: ModifiersState,
    ime: bool,
    ime_allowed: bool,
    preedit: String,
    save_retry_at: Instant,
    start: Instant,
    last: Instant,
    next_redraw: Instant,
    occluded: bool,
    data_dir: PathBuf,
    smoke: Option<PathBuf>,
    smoke_stage: usize,
    failure: Option<String>,
}
impl App {
    fn now(&self) -> u64 {
        self.start.elapsed().as_millis() as u64
    }
    fn dispatch(&mut self, e: Event) {
        self.phone.status.clear();
        // Multiple key events can arrive before a redraw. Refresh visual row
        // metrics before vertical motion instead of using stale wrap positions.
        if matches!(
            e.key,
            phone_core::input::Key::Up | phone_core::input::Key::Down
        ) && self.phone.editor().is_some()
        {
            views::render(&mut self.canvas, &mut self.phone);
        }
        self.phone.handle(e, self.now());
    }
    fn press(&mut self, source: u32, key: phone_core::input::Key) {
        if let Some(e) = self.input.press(source, key, self.now()) {
            self.dispatch(e);
        }
    }
    fn release(&mut self, source: u32) {
        for e in self.input.release(source) {
            self.dispatch(e);
        }
    }
    fn flush(&mut self) {
        if self.phone.snake_dirty && Instant::now() >= self.save_retry_at {
            match storage::save_json(&self.data_dir, "snake.json", &self.phone.snake_data, 4096) {
                Ok(()) => self.phone.snake_dirty = false,
                Err(e) => {
                    self.save_retry_at = Instant::now() + Duration::from_secs(5);
                    self.phone.status = format!("游戏保存失败：{e}");
                }
            }
        }
        if self.phone.local_dirty && Instant::now() >= self.save_retry_at {
            match storage::save_json(
                &self.data_dir,
                "organizer.json",
                &self.phone.local_data,
                8 * 1024 * 1024,
            ) {
                Ok(()) => self.phone.local_dirty = false,
                Err(e) => {
                    self.save_retry_at = Instant::now() + Duration::from_secs(5);
                    self.phone.status = format!("保存失败：{e}");
                }
            }
        }
        if self.phone.browser_dirty && Instant::now() >= self.save_retry_at {
            match storage::save_browser(&self.data_dir, &self.phone.browser_data) {
                Ok(()) => self.phone.browser_dirty = false,
                Err(e) => {
                    self.save_retry_at = Instant::now() + Duration::from_secs(5);
                    self.phone.status = format!("浏览记录保存失败：{e}");
                }
            }
        }
        if self.phone.preferences_dirty && Instant::now() >= self.save_retry_at {
            match storage::save_preferences(&self.data_dir, &self.phone.preferences) {
                Ok(()) => self.phone.preferences_dirty = false,
                Err(e) => {
                    self.save_retry_at = Instant::now() + Duration::from_secs(5);
                    self.phone.status = format!("设置保存失败：{e}");
                }
            }
        }
        if self.phone.dirty && Instant::now() >= self.save_retry_at {
            self.save_retry_at = Instant::now() + Duration::from_secs(5);
            match storage::save(&self.data_dir, &self.phone.mailbox) {
                Ok(()) => {
                    self.phone.storage_saved();
                    self.save_retry_at = Instant::now();
                }
                Err(e) => {
                    self.phone.storage_failed();
                    self.phone.status = format!("Save failed: {e}");
                    eprintln!("{}", self.phone.status);
                    self.phone
                        .notice(format!("Save failed; data remains in memory. {e}"));
                }
            }
        }
    }
    fn quit(&mut self, event_loop: &ActiveEventLoop) {
        if self.phone.mail_ui.pending.is_some() {
            self.save_retry_at = Instant::now();
            self.flush();
            // Failed queue writes roll back and leave the original editor open.
            if self.phone.screen == phone::Screen::Compose {
                return;
            }
        }
        if self.phone.screen == phone::Screen::Compose {
            if !self.phone.compose.body.accept_prediction()
                || !self.phone.compose.to.accept_prediction()
            {
                self.phone
                    .notice("Text limit reached. Confirm or cancel the candidate before closing.");
                return;
            }
            let m = self.phone.message();
            if !m.to.is_empty() || !m.body.is_empty() {
                let index = self.phone.mailbox.save(self.phone.compose.draft, m);
                self.phone.compose.draft = Some(index);
                self.phone.dirty = true;
            }
        }
        self.save_retry_at = Instant::now();
        self.flush();
        if self.phone.dirty
            || self.phone.preferences_dirty
            || self.phone.browser_dirty
            || self.phone.local_dirty
            || self.phone.snake_dirty
        {
            self.phone
                .notice("Save failed. Window stays open so you can keep your text.");
            return;
        }
        if let Some(media) = &mut self.media
            && let Err(e) = media.save(&self.data_dir, &self.phone, true)
        {
            self.phone.notice(format!("音乐设置保存失败：{e}"));
            return;
        }
        event_loop.exit();
    }
    fn fail(&mut self, event_loop: &ActiveEventLoop, e: String) {
        eprintln!("{e}");
        self.failure = Some(e);
        event_loop.exit();
    }
}
impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.window.is_some() {
            return;
        }
        let attributes = Window::default_attributes()
            .with_title("TA-1278 · Phone interaction")
            .with_inner_size(LogicalSize::new(480., 640.))
            .with_min_inner_size(LogicalSize::new(240., 320.));
        #[cfg(target_os = "macos")]
        let attributes = {
            use winit::platform::macos::WindowAttributesExtMacOS;
            attributes.with_accepts_first_mouse(true)
        };
        match event_loop.create_window(attributes) {
            Ok(window) => {
                let window = Arc::new(window);
                window.set_ime_allowed(false);
                match pollster::block_on(gpu::Gpu::new(window.clone())) {
                    Ok(gpu) => {
                        if self.smoke.is_some() {
                            window.focus_window();
                        }
                        self.gpu = Some(gpu);
                        self.window = Some(window);
                    }
                    Err(e) => self.fail(event_loop, e),
                }
            }
            Err(e) => self.fail(event_loop, e.to_string()),
        }
    }
    fn window_event(&mut self, event_loop: &ActiveEventLoop, id: WindowId, event: WindowEvent) {
        if self.window.as_ref().is_none_or(|w| w.id() != id) {
            return;
        }
        match event {
            WindowEvent::CloseRequested => self.quit(event_loop),
            WindowEvent::Occluded(value) => self.occluded = value,
            WindowEvent::Resized(size) => {
                if let Some(gpu) = &mut self.gpu {
                    gpu.resize(size.width, size.height);
                }
            }
            WindowEvent::Focused(false) => {
                for e in self.input.cancel() {
                    self.dispatch(e);
                }
                self.modifiers = ModifiersState::empty();
                self.ime = false;
                self.preedit.clear();
            }
            WindowEvent::ModifiersChanged(m) => self.modifiers = m.state(),
            WindowEvent::Ime(Ime::Preedit(s, _)) => {
                self.ime = !s.is_empty();
                self.preedit = s;
            }
            WindowEvent::Ime(Ime::Commit(s)) => {
                self.ime = false;
                self.preedit.clear();
                self.phone.text(&s);
            }
            WindowEvent::MouseInput { state, button, .. } => {
                if let Some((source, key)) = host_input::mouse(button) {
                    if state == ElementState::Pressed {
                        self.press(source, key)
                    } else {
                        self.release(source)
                    }
                }
            }
            WindowEvent::KeyboardInput {
                event,
                is_synthetic: false,
                ..
            } => {
                if let PhysicalKey::Code(code) = event.physical_key {
                    let source = code as u32;
                    if event.state == ElementState::Released {
                        self.release(source);
                        return;
                    }
                    if self.ime {
                        return;
                    }
                    if self.modifiers.super_key() || self.modifiers.control_key() {
                        if code == KeyCode::KeyA {
                            if let Some(e) = self.phone.editor_mut() {
                                e.select_all();
                            }
                        } else if code == KeyCode::KeyQ {
                            self.quit(event_loop);
                        }
                        return;
                    }
                    if code == KeyCode::Tab {
                        if !event.repeat {
                            self.phone.tab();
                        }
                        return;
                    }
                    if code == KeyCode::F4 {
                        if !event.repeat {
                            self.phone.input_mode();
                        }
                        return;
                    }
                    if self.modifiers.shift_key() {
                        if code == KeyCode::Enter {
                            self.phone.text("\n");
                            return;
                        }
                        if let Some(text) = event
                            .text
                            .as_deref()
                            .filter(|s| s.chars().all(|c| !c.is_control()))
                        {
                            self.phone.text(text);
                            return;
                        }
                    }
                    if let Some(key) = host_input::keyboard(code) {
                        if !event.repeat {
                            self.press(source, key);
                        }
                    } else if let Some(text) = event
                        .text
                        .as_deref()
                        .filter(|s| s.chars().all(|c| !c.is_control()))
                    {
                        self.phone.text(text);
                    }
                }
            }
            WindowEvent::RedrawRequested => {
                if self.occluded {
                    return;
                }
                let before = self.gpu.as_ref().map_or(0, |g| g.presented);
                views::render(&mut self.canvas, &mut self.phone);
                if !self.preedit.is_empty() {
                    self.canvas.rect(4, 250, 232, 44, self.canvas.accent(3));
                    self.canvas
                        .fit(10, 254, &self.preedit, 16, canvas::WHITE, 220);
                    self.canvas
                        .text(10, 275, "IME · confirm candidate", 12, canvas::WHITE);
                }
                if let Some(gpu) = &mut self.gpu
                    && let Err(e) = gpu.render(&self.canvas.pixels)
                {
                    self.fail(event_loop, e);
                    return;
                }
                if self.smoke.is_some() && self.gpu.as_ref().is_some_and(|g| g.presented > before) {
                    self.smoke_frame(event_loop);
                }
            }
            _ => {}
        }
    }
    fn about_to_wait(&mut self, event_loop: &ActiveEventLoop) {
        let now = Instant::now();
        if self.smoke.is_some() && self.start.elapsed() > Duration::from_secs(30) {
            self.fail(event_loop, "Window smoke test timed out before all frames were presented; ensure the window is visible.".into());
            return;
        }
        let dt = now.duration_since(self.last).as_secs_f32().min(0.25);
        self.last = now;
        for e in self.input.tick(self.now()) {
            self.dispatch(e);
        }
        self.phone.tick(self.now(), dt);
        if let Some(media) = &mut self.media {
            media.tick(&mut self.phone);
            if Instant::now() >= self.save_retry_at
                && let Err(e) = media.save(&self.data_dir, &self.phone, false)
            {
                self.save_retry_at = Instant::now() + Duration::from_secs(5);
                self.phone.notice(format!("音乐设置保存失败：{e}"));
            }
        }
        if let Some(url) = self.phone.pending_url.take() {
            self.phone.requested_url = url.clone();
            let id = self.phone.browser.start();
            if let Err(e) = self.web.load(id, url) {
                self.phone.browser.complete(id, Err(e));
            }
        }
        while let Ok((id, result)) = self.web.results.try_recv() {
            let updated = self.phone.browser.complete(id, result);
            if updated {
                self.phone.browser_data.visit(&self.phone.browser.page);
                self.phone.browser_dirty = true;
            }
            if updated
                && matches!(
                    self.phone.overlay,
                    Some(
                        phone::Overlay::Text {
                            target: phone::TextTarget::Field(_),
                            ..
                        } | phone::Overlay::Select { .. }
                    )
                )
            {
                self.phone.overlay = None;
            }
        }
        self.flush();
        if let Some(w) = &self.window {
            let enabled = self.phone.editor().is_some();
            if enabled != self.ime_allowed {
                w.set_ime_allowed(enabled);
                self.ime_allowed = enabled;
                self.ime = false;
                self.preedit.clear();
            }
            let size = w.inner_size().to_logical::<f64>(w.scale_factor());
            w.set_ime_cursor_area(
                winit::dpi::LogicalPosition::new(20., size.height * 0.8),
                LogicalSize::new(200., 24.),
            );
            if now >= self.next_redraw {
                self.next_redraw = now + Duration::from_millis(33);
                if !self.occluded {
                    w.request_redraw();
                }
            }
        }
        event_loop.set_control_flow(ControlFlow::WaitUntil(self.next_redraw));
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut data_dir = storage::default_dir();
    let mut smoke = None;
    let mut features = config::Features::default();
    let mut card = None;
    let mut no_card = false;
    let mut read_only = false;
    let mut args = std::env::args().skip(1);
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--data-dir" => data_dir = PathBuf::from(args.next().ok_or("--data-dir needs a path")?),
            "--mount-card" => {
                card = Some(PathBuf::from(
                    args.next().ok_or("--mount-card needs a directory")?,
                ))
            }
            "--no-card" => no_card = true,
            "--read-only-storage" => read_only = true,
            "--disable" => features.disable(
                &args
                    .next()
                    .ok_or("--disable needs a comma-separated list")?,
            )?,
            "--smoke-test" => {
                let dir = PathBuf::from(
                    args.next()
                        .ok_or("--smoke-test needs an output directory")?,
                );
                std::fs::create_dir_all(&dir)?;
                data_dir = dir.join("data");
                smoke = Some(dir);
            }
            "--help" => {
                println!(
                    "cargo run -p phone-desktop -- [--data-dir PATH] [--mount-card DIRECTORY | --no-card] [--read-only-storage] [--disable music,files,network] [--smoke-test OUTPUT]\nArrows: navigate; Enter: center; F1/F2: softkeys; F3: call; Home: end/home; Tab: recipient/body; F4: input mode. Mouse left/right/middle = left/right/center key."
                );
                return Ok(());
            }
            _ => return Err(format!("Unknown option: {arg}").into()),
        }
    }
    if no_card && card.is_some() {
        return Err("--no-card conflicts with --mount-card".into());
    }
    let mailbox = storage::load(&data_dir)?;
    let mut phone = phone::Phone::new(mailbox);
    phone.features = features;
    phone.apply_preferences(storage::load_preferences(&data_dir)?);
    phone.browser_data = storage::load_browser(&data_dir)?;
    phone.local_data = storage::load_organizer(&data_dir)?;
    phone.snake_data = storage::load_snake(&data_dir)?;
    phone.snake.level = phone.snake_data.level;
    phone.snake.kind = phone.snake_data.kind;
    if smoke.is_some() {
        phone.accent = 4;
    }
    let media = if smoke.is_none() {
        let mut vfs = phone_vfs::Vfs::default();
        let phone_root = data_dir.join("files");
        std::fs::create_dir_all(&phone_root)?;
        vfs.mount(
            "phone",
            "手机存储",
            Box::new(phone_vfs::Directory::new(phone_root)?),
            read_only,
        )?;
        if !no_card {
            let card_root = if let Some(path) = card {
                path
            } else {
                let path = data_dir.join("card");
                std::fs::create_dir_all(&path)?;
                path
            };
            vfs.mount(
                "card",
                "存储卡",
                Box::new(phone_vfs::Directory::new(card_root)?),
                read_only,
            )?;
        }
        Some(media::Media::new(vfs, &data_dir, &mut phone)?)
    } else {
        None
    };
    let now = Instant::now();
    let mut app = App {
        window: None,
        gpu: None,
        canvas: canvas::Canvas::new()?,
        phone,
        input: Input::default(),
        web: web::Web::new(features.network),
        media,
        modifiers: ModifiersState::empty(),
        ime: false,
        ime_allowed: false,
        preedit: String::new(),
        save_retry_at: now,
        start: now,
        last: now,
        next_redraw: now,
        occluded: false,
        data_dir,
        smoke,
        smoke_stage: 0,
        failure: None,
    };
    EventLoop::new()?.run_app(&mut app)?;
    if let Some(e) = app.failure {
        return Err(e.into());
    }
    Ok(())
}
