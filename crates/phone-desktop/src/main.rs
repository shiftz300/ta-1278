mod audio;
mod candidate_view;
mod canvas;
mod config;
mod effects;
mod gpu;
mod host_input;
mod local_files;
#[cfg(target_os = "macos")]
mod macos_lifecycle;
mod media;
mod menu;
mod midi_synth;
mod native_audio;
mod packed;
mod phone;
mod services;
mod smoke;
mod snake_art;
mod storage;
mod storage_setup;
mod theme;
mod ui;
mod views;
mod visual_media;
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
    event::{ElementState, WindowEvent},
    event_loop::{ActiveEventLoop, ControlFlow, EventLoop},
    keyboard::{KeyCode, ModifiersState, PhysicalKey},
    window::{Window, WindowId},
};
struct App {
    window: Option<Arc<Window>>,
    gpu: Option<gpu::Gpu>,
    canvas: canvas::Canvas,
    phone: phone::Phone,
    services: services::Services,
    predictor: Arc<phone_core::predict::firmware::Firmware>,
    input: Input,
    web: web::Web,
    media: Option<media::Media>,
    modifiers: ModifiersState,
    save_retry_at: Instant,
    start: Instant,
    last: Instant,
    next_redraw: Instant,
    occluded: bool,
    data_dir: PathBuf,
    utilities: storage::utilities::Session,
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
        if let Some(error) = self.predictor.take_error() {
            self.phone.status = format!("输入法错误：{error}");
        }
        if Instant::now() >= self.save_retry_at
            && let Some((data, revision)) = self.predictor.pending_save()
        {
            match storage::save_json(&self.data_dir, "input-native.json", &data, 2 * 1024 * 1024) {
                Ok(()) => self.predictor.saved(revision),
                Err(e) => {
                    self.save_retry_at = Instant::now() + Duration::from_secs(5);
                    self.phone.status = format!("输入记录保存失败：{e}");
                }
            }
        }
        if Instant::now() >= self.save_retry_at
            && let Err(e) = self
                .utilities
                .save(&self.data_dir, &mut self.phone.tools, false)
        {
            self.save_retry_at = Instant::now() + Duration::from_secs(5);
            self.phone.status = format!("计时数据保存失败：{e}");
        }
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
        if self.phone.files.busy
            || self
                .phone
                .files
                .pending
                .as_ref()
                .is_some_and(phone::files::FileRequest::mutating)
        {
            self.phone.notice("文件操作尚未结束，请稍候。");
            return;
        }
        if self.phone.mail_ui.pending.is_some() {
            self.save_retry_at = Instant::now();
            self.flush();
            // Failed queue writes roll back and leave the original editor open.
            if matches!(
                self.phone.screen,
                phone::Screen::Compose | phone::Screen::Conversation
            ) {
                return;
            }
        }
        if matches!(
            self.phone.screen,
            phone::Screen::Compose | phone::Screen::Conversation
        ) {
            if !self.phone.compose.body.accept_prediction()
                || !self.phone.compose.to.accept_prediction()
            {
                self.phone
                    .notice("Text limit reached. Confirm or cancel the candidate before closing.");
                return;
            }
            self.phone.save_draft();
        }
        self.save_retry_at = Instant::now();
        self.flush();
        if self.phone.dirty
            || self.predictor.pending_save().is_some()
            || self.phone.preferences_dirty
            || self.phone.browser_dirty
            || self.phone.local_dirty
            || self.phone.snake_dirty
        {
            self.phone
                .notice("Save failed. Window stays open so you can keep your text.");
            return;
        }
        if let Err(e) = self
            .utilities
            .save(&self.data_dir, &mut self.phone.tools, true)
        {
            self.phone
                .notice(format!("计时数据保存失败，窗口保持打开：{e}"));
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
            }
            WindowEvent::ModifiersChanged(m) => self.modifiers = m.state(),
            // Ignore even late IME events queued before the window disabled composition.
            WindowEvent::Ime(_) => {}
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
                    // Option/Alt must not open a second, host-controlled text path.
                    if self.modifiers.alt_key() {
                        return;
                    }
                    match host_input::key(code, self.modifiers.shift_key()) {
                        Some(host_input::Keyboard::Phone(key)) if !event.repeat => {
                            self.press(source, key);
                        }
                        Some(host_input::Keyboard::Text(ch)) => self.phone.text(&ch.to_string()),
                        _ => {}
                    }
                }
            }
            WindowEvent::RedrawRequested => {
                if self.occluded {
                    return;
                }
                let before = self.gpu.as_ref().map_or(0, |g| g.presented);
                views::render(&mut self.canvas, &mut self.phone);
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
        #[cfg(target_os = "macos")]
        if macos_lifecycle::take_quit_request() {
            self.quit(event_loop);
        }
        if event_loop.exiting() {
            return;
        }
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
            if !self.phone.preferences.mobile_data {
                self.phone
                    .browser
                    .complete(id, Err("移动数据已关闭。".into()));
            } else if let Err(e) = self.web.load(id, url) {
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
            if let Err(e) = self.services.tick(&mut self.phone) {
                self.phone.status = e;
            }
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
    let mut storage_dir = None;
    let mut read_only = false;
    let mut args = std::env::args().skip(1);
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--version" | "-V" => {
                println!("TA1278 {}", env!("CARGO_PKG_VERSION"));
                return Ok(());
            }
            "--render-tone" => {
                let tone = args.next().ok_or("--render-tone needs NAME OUTPUT.wav")?;
                let output =
                    PathBuf::from(args.next().ok_or("--render-tone needs NAME OUTPUT.wav")?);
                if args.next().is_some() {
                    return Err("--render-tone does not accept other options".into());
                }
                native_audio::export(&tone, &output)?;
                return Ok(());
            }
            "--print-media-helper" => {
                println!("{}", option_env!("TA1278_MEDIA_HELPER").unwrap_or(""));
                return Ok(());
            }
            "--data-dir" => data_dir = PathBuf::from(args.next().ok_or("--data-dir needs a path")?),
            "--storage-dir" => {
                storage_dir = Some(PathBuf::from(
                    args.next().ok_or("--storage-dir needs a directory")?,
                ));
            }
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
                    "cargo run -p phone-desktop -- [--data-dir PATH] [--storage-dir DIRECTORY] [--read-only-storage] [--disable music,files,network,pictures,video,sounds] [--smoke-test OUTPUT]\nOffline tone preview: --render-tone NAME OUTPUT.wav (creates a new file)\nArrows: navigate; Enter: center; F1/F2: softkeys; F3: disabled; Home: end/home; Tab: recipient/body; # (Shift+3): input mode. Mouse left/right/middle = left/right/center key."
                );
                return Ok(());
            }
            _ => return Err(format!("Unknown option: {arg}").into()),
        }
    }
    let _data_lock = storage::DataLock::acquire(&data_dir)?;
    let folder = if smoke.is_none() {
        match storage_setup::select(&data_dir, storage_dir, read_only)? {
            Some(folder) => Some(folder),
            None => return Ok(()),
        }
    } else {
        None
    };
    let mailbox = storage::load(&data_dir)?;
    let mut phone = phone::Phone::new(mailbox);
    let predictor = Arc::new(phone_core::predict::firmware::Firmware::from_data(
        storage::load_json(&data_dir, "input-native.json", 2 * 1024 * 1024)?,
    )?);
    phone.prediction_engine = predictor.clone();
    phone.features = features;
    phone.apply_preferences(storage::load_preferences(&data_dir)?);
    phone.browser_data = storage::load_browser(&data_dir)?;
    phone.local_data = storage::load_organizer(&data_dir)?;
    phone.apply_snake_data(storage::load_snake(&data_dir)?);
    if smoke.is_some() {
        phone.accent = 4;
    }
    let media = folder
        .map(|folder| media::Media::new(folder, &data_dir, &mut phone))
        .transpose()?;
    let utilities = storage::utilities::Session::load(&data_dir, &mut phone.tools)?;
    let now = Instant::now();
    let mut app = App {
        window: None,
        gpu: None,
        canvas: canvas::Canvas::new()?,
        phone,
        services: services::Services::default(),
        predictor,
        input: Input::default(),
        web: web::Web::new(features.network),
        media,
        modifiers: ModifiersState::empty(),
        save_retry_at: now,
        start: now,
        last: now,
        next_redraw: now,
        occluded: false,
        data_dir,
        utilities,
        smoke,
        smoke_stage: 0,
        failure: None,
    };
    #[cfg(target_os = "macos")]
    macos_lifecycle::initialize()?;
    EventLoop::new()?.run_app(&mut app)?;
    if let Some(e) = app.failure {
        return Err(e.into());
    }
    Ok(())
}
