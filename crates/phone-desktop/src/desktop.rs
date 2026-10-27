use crate::runtime::Time;
use crate::{audio, config, gpu, host, phone, platform, runtime::Runtime, storage};
mod smoke;
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
    runtime: Runtime,
    platform: platform::Native,
    modifiers: ModifiersState,
    save_retry_at: Instant,
    start: Instant,
    next_redraw: Instant,
    occluded: bool,
    data_dir: PathBuf,
    utilities: storage::session::Session,
    smoke: Option<PathBuf>,
    smoke_stage: usize,
    failure: Option<String>,
}
impl App {
    fn now(&self) -> u64 {
        self.start.elapsed().as_millis() as u64
    }
    fn press(&mut self, source: u32, key: phone_core::input::Key) {
        self.runtime.press(source, key);
    }
    fn release(&mut self, source: u32) {
        self.runtime.release(source);
    }
    fn flush(&mut self) {
        if let Some(error) = self.runtime.predictor.take_error() {
            self.runtime.phone.status = format!("输入法错误：{error}");
        }
        if Instant::now() >= self.save_retry_at
            && let Some((data, revision)) = self.runtime.predictor.pending_save()
        {
            match storage::save_json(&self.data_dir, "input-native.json", &data, 2 * 1024 * 1024) {
                Ok(()) => self.runtime.predictor.saved(revision),
                Err(e) => {
                    self.save_retry_at = Instant::now() + Duration::from_secs(5);
                    self.runtime.phone.status = format!("输入记录保存失败：{e}");
                }
            }
        }
        if Instant::now() >= self.save_retry_at
            && let Err(e) =
                self.utilities
                    .save(&self.data_dir, &mut self.runtime.phone.tools, false)
        {
            self.save_retry_at = Instant::now() + Duration::from_secs(5);
            self.runtime.phone.status = format!("计时数据保存失败：{e}");
        }
        if self.runtime.phone.snake_dirty && Instant::now() >= self.save_retry_at {
            match storage::save_json(
                &self.data_dir,
                "snake.json",
                &self.runtime.phone.snake_data,
                4096,
            ) {
                Ok(()) => self.runtime.phone.snake_dirty = false,
                Err(e) => {
                    self.save_retry_at = Instant::now() + Duration::from_secs(5);
                    self.runtime.phone.status = format!("游戏保存失败：{e}");
                }
            }
        }
        if self.runtime.phone.local_dirty && Instant::now() >= self.save_retry_at {
            match storage::save_json(
                &self.data_dir,
                "organizer.json",
                &self.runtime.phone.local_data,
                8 * 1024 * 1024,
            ) {
                Ok(()) => self.runtime.phone.local_dirty = false,
                Err(e) => {
                    self.save_retry_at = Instant::now() + Duration::from_secs(5);
                    self.runtime.phone.status = format!("保存失败：{e}");
                }
            }
        }
        if self.runtime.phone.browser_dirty && Instant::now() >= self.save_retry_at {
            match storage::save_browser(&self.data_dir, &self.runtime.phone.browser_data) {
                Ok(()) => self.runtime.phone.browser_dirty = false,
                Err(e) => {
                    self.save_retry_at = Instant::now() + Duration::from_secs(5);
                    self.runtime.phone.status = format!("浏览记录保存失败：{e}");
                }
            }
        }
        if self.runtime.phone.preferences_dirty && Instant::now() >= self.save_retry_at {
            match storage::save_preferences(&self.data_dir, &self.runtime.phone.preferences) {
                Ok(()) => self.runtime.phone.preferences_dirty = false,
                Err(e) => {
                    self.save_retry_at = Instant::now() + Duration::from_secs(5);
                    self.runtime.phone.status = format!("设置保存失败：{e}");
                }
            }
        }
        if self.runtime.phone.dirty && Instant::now() >= self.save_retry_at {
            self.save_retry_at = Instant::now() + Duration::from_secs(5);
            match storage::save(&self.data_dir, &self.runtime.phone.mailbox) {
                Ok(()) => {
                    self.runtime.phone.storage_saved();
                    self.save_retry_at = Instant::now();
                }
                Err(e) => {
                    self.runtime.phone.storage_failed();
                    self.runtime.phone.status = format!("Save failed: {e}");
                    eprintln!("{}", self.runtime.phone.status);
                    self.runtime
                        .phone
                        .notice(format!("Save failed; data remains in memory. {e}"));
                }
            }
        }
    }
    fn quit(&mut self, event_loop: &ActiveEventLoop) {
        if self.runtime.phone.files.busy
            || self
                .runtime
                .phone
                .files
                .pending
                .as_ref()
                .is_some_and(phone::files::FileRequest::mutating)
        {
            self.runtime.phone.notice("文件操作尚未结束，请稍候。");
            return;
        }
        if self.runtime.phone.mail_ui.pending.is_some() {
            self.save_retry_at = Instant::now();
            self.flush();
            // Failed queue writes roll back and leave the original editor open.
            if matches!(
                self.runtime.phone.screen,
                phone::Screen::Compose | phone::Screen::Conversation
            ) {
                return;
            }
        }
        if matches!(
            self.runtime.phone.screen,
            phone::Screen::Compose | phone::Screen::Conversation
        ) {
            if !self.runtime.phone.compose.body.accept_prediction()
                || !self.runtime.phone.compose.to.accept_prediction()
            {
                self.runtime
                    .phone
                    .notice("Text limit reached. Confirm or cancel the candidate before closing.");
                return;
            }
            self.runtime.phone.save_draft();
        }
        self.save_retry_at = Instant::now();
        self.flush();
        if self.runtime.phone.dirty
            || self.runtime.predictor.pending_save().is_some()
            || self.runtime.phone.preferences_dirty
            || self.runtime.phone.browser_dirty
            || self.runtime.phone.local_dirty
            || self.runtime.phone.snake_dirty
        {
            self.runtime
                .phone
                .notice("Save failed. Window stays open so you can keep your text.");
            return;
        }
        if let Err(e) = self
            .utilities
            .save(&self.data_dir, &mut self.runtime.phone.tools, true)
        {
            self.runtime
                .phone
                .notice(format!("计时数据保存失败，窗口保持打开：{e}"));
            return;
        }
        if let Err(e) = self.platform.save(&self.runtime, &self.data_dir, true) {
            self.runtime.phone.notice(format!("音乐设置保存失败：{e}"));
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
                self.runtime.cancel_input();
                self.modifiers = ModifiersState::empty();
            }
            WindowEvent::ModifiersChanged(m) => self.modifiers = m.state(),
            // Ignore even late IME events queued before the window disabled composition.
            WindowEvent::Ime(_) => {}
            WindowEvent::MouseInput { state, button, .. } => {
                if let Some((source, key)) = host::input::mouse(button) {
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
                            self.runtime.select_all();
                        } else if code == KeyCode::KeyQ {
                            self.quit(event_loop);
                        }
                        return;
                    }
                    if code == KeyCode::Tab {
                        if !event.repeat {
                            self.runtime.tab();
                        }
                        return;
                    }
                    // Option/Alt must not open a second, host-controlled text path.
                    if self.modifiers.alt_key() {
                        return;
                    }
                    match host::input::key(code, self.modifiers.shift_key()) {
                        Some(host::input::Keyboard::Phone(key)) if !event.repeat => {
                            self.press(source, key);
                        }
                        Some(host::input::Keyboard::Text(ch)) => self.runtime.text(&ch.to_string()),
                        _ => {}
                    }
                }
            }
            WindowEvent::RedrawRequested => {
                if self.occluded {
                    return;
                }
                let before = self.gpu.as_ref().map_or(0, |g| g.presented);
                self.runtime.render();
                if let Some(gpu) = &mut self.gpu
                    && let Err(e) = gpu.render(&self.runtime.canvas.pixels)
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
        if host::lifecycle::take_quit_request() {
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
        if let Err(error) = self.runtime.advance(Time {
            elapsed_ms: self.now(),
            unix_ms: storage::session::wall_ms(),
        }) {
            self.fail(event_loop, error);
            return;
        }
        if let Err(error) = self.platform.poll(&mut self.runtime) {
            self.runtime.phone.status = error;
        }
        if Instant::now() >= self.save_retry_at
            && let Err(error) = self.platform.save(&self.runtime, &self.data_dir, false)
        {
            self.save_retry_at = Instant::now() + Duration::from_secs(5);
            self.runtime
                .phone
                .notice(format!("音乐设置保存失败：{error}"));
        }
        self.flush();
        if let Some(w) = &self.window {
            if let Err(e) = self.runtime.poll_services() {
                self.runtime.phone.status = e;
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

pub fn run() -> Result<(), Box<dyn std::error::Error>> {
    let mut data_dir = storage::default_dir();
    let mut smoke = None;
    let mut config_path = None;
    let mut check_config = false;
    let mut disabled = Vec::new();
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
                audio::native::export(&tone, &output)?;
                return Ok(());
            }
            "--print-media-helper" => {
                println!("{}", option_env!("TA1278_MEDIA_HELPER").unwrap_or(""));
                return Ok(());
            }
            "--check-config" => check_config = true,
            "--config" => {
                config_path = Some(PathBuf::from(args.next().ok_or("--config needs a path")?))
            }
            "--data-dir" => data_dir = PathBuf::from(args.next().ok_or("--data-dir needs a path")?),
            "--storage-dir" => {
                storage_dir = Some(PathBuf::from(
                    args.next().ok_or("--storage-dir needs a directory")?,
                ));
            }
            "--read-only-storage" => read_only = true,
            "--disable" => disabled.push(
                args.next()
                    .ok_or("--disable needs a comma-separated list")?,
            ),
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
                    "cargo run -p phone-desktop -- [--config JSON] [--check-config] [--data-dir PATH] [--storage-dir DIRECTORY] [--read-only-storage] [--disable music,files,network,javascript,pictures,video,sounds] [--smoke-test OUTPUT]\nOffline tone preview: --render-tone NAME OUTPUT.wav (creates a new file)\nArrows: navigate; Enter: center; F1/F2: softkeys; F3: call history/dial; Home: end/home; Tab: recipient/body; # (Shift+3): input mode. Mouse left/right/middle = left/right/center key."
                );
                return Ok(());
            }
            _ => return Err(format!("Unknown option: {arg}").into()),
        }
    }
    let path = config_path
        .as_deref()
        .map(PathBuf::from)
        .unwrap_or_else(|| data_dir.join("config.json"));
    let mut runtime_config = config::RuntimeConfig::read(&path, config_path.is_some())?;
    for names in disabled {
        runtime_config.capabilities.disable(&names)?;
    }
    runtime_config.storage.directory = storage_dir.or(runtime_config.storage.directory);
    runtime_config.storage.read_only |= read_only;
    let features = runtime_config.effective_features();
    if check_config {
        println!("{}", serde_json::to_string_pretty(&runtime_config)?);
        return Ok(());
    }
    storage_dir = runtime_config.storage.directory;
    read_only = runtime_config.storage.read_only;
    let _data_lock = storage::DataLock::acquire(&data_dir)?;
    let folder = if smoke.is_none() {
        match storage::setup::select(&data_dir, storage_dir, read_only)? {
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
    phone.apps = runtime_config.apps;
    phone.apply_preferences(storage::load_preferences(&data_dir)?);
    phone.browser_data = storage::load_browser(&data_dir)?;
    phone.local_data = storage::load_organizer(&data_dir)?;
    phone.apply_snake_data(storage::load_snake(&data_dir)?);
    if smoke.is_some() {
        phone.accent = 4;
    }
    let utilities = storage::session::Session::load(&data_dir, &mut phone.tools)?;
    let mut runtime = Runtime::from_phone(
        phone,
        predictor,
        Time {
            elapsed_ms: 0,
            unix_ms: storage::session::wall_ms(),
        },
    )?;
    let platform = platform::Native::from_folder(&mut runtime, folder, &data_dir)?;
    let now = Instant::now();
    let mut app = App {
        window: None,
        gpu: None,
        runtime,
        platform,
        modifiers: ModifiersState::empty(),
        save_retry_at: now,
        start: now,
        next_redraw: now,
        occluded: false,
        data_dir,
        utilities,
        smoke,
        smoke_stage: 0,
        failure: None,
    };
    #[cfg(target_os = "macos")]
    host::lifecycle::initialize()?;
    EventLoop::new()?.run_app(&mut app)?;
    if let Some(e) = app.failure {
        return Err(e.into());
    }
    Ok(())
}
