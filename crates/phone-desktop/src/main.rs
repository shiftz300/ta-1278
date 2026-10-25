mod canvas;
mod gpu;
mod host_input;
mod phone;
mod storage;
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
    modifiers: ModifiersState,
    ime: bool,
    ime_allowed: bool,
    preedit: String,
    save_retry_at: Instant,
    start: Instant,
    last: Instant,
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
        if self.phone.dirty && Instant::now() >= self.save_retry_at {
            self.save_retry_at = Instant::now() + Duration::from_secs(5);
            match storage::save(&self.data_dir, &self.phone.mailbox) {
                Ok(()) => {
                    self.phone.dirty = false;
                    self.save_retry_at = Instant::now();
                }
                Err(e) => {
                    self.phone.status = format!("Save failed: {e}");
                    eprintln!("{}", self.phone.status);
                    self.phone
                        .notice(format!("Save failed; data remains in memory. {e}"));
                }
            }
        }
    }
    fn quit(&mut self, event_loop: &ActiveEventLoop) {
        if self.phone.screen == phone::Screen::Compose {
            let m = self.phone.message();
            if !m.to.is_empty() || !m.body.is_empty() {
                let index = self.phone.mailbox.save(self.phone.compose.draft, m);
                self.phone.compose.draft = Some(index);
                self.phone.dirty = true;
            }
        }
        self.save_retry_at = Instant::now();
        self.flush();
        if self.phone.dirty {
            self.phone
                .notice("Save failed. Window stays open so you can keep your text.");
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
                views::render(&mut self.canvas, &mut self.phone);
                if !self.preedit.is_empty() {
                    self.canvas.rect(4, 250, 232, 44, canvas::BLUE);
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
                if self.smoke.is_some() {
                    self.smoke_frame(event_loop);
                }
            }
            _ => {}
        }
    }
    fn about_to_wait(&mut self, event_loop: &ActiveEventLoop) {
        let now = Instant::now();
        let dt = now.duration_since(self.last).as_secs_f32().min(0.25);
        self.last = now;
        for e in self.input.tick(self.now()) {
            self.dispatch(e);
        }
        self.phone.tick(self.now(), dt);
        if let Some(url) = self.phone.pending_url.take() {
            let id = self.phone.browser.start();
            if let Err(e) = self.web.load(id, url) {
                self.phone.browser.complete(id, Err(e));
            }
        }
        while let Ok((id, result)) = self.web.results.try_recv() {
            if self.phone.browser.complete(id, result)
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
            w.request_redraw();
        }
        event_loop.set_control_flow(ControlFlow::WaitUntil(now + Duration::from_millis(33)));
    }
}
impl App {
    /// Synthetic route checks on an actual winit/wgpu surface. Not physical-device QA.
    fn smoke_frame(&mut self, event_loop: &ActiveEventLoop) {
        use phone::{Compose, Screen};
        use phone_core::input::{Key, Phase};
        let dir = self.smoke.clone().unwrap();
        if self.gpu.as_ref().is_none_or(|g| g.presented == 0) {
            return;
        }
        let name = match self.smoke_stage {
            0 => "home",
            1 => "menu",
            2 => "sms",
            3 => "browser",
            4 => "music",
            _ => {
                eprintln!(
                    "SMOKE PASS: {} presented frames",
                    self.gpu.as_ref().unwrap().presented
                );
                event_loop.exit();
                return;
            }
        };
        if let Err(e) = self.canvas.save(&dir.join(format!("{name}.png"))) {
            self.fail(event_loop, e.to_string());
            return;
        }
        match self.smoke_stage {
            0 => self.dispatch(Event {
                key: Key::SoftLeft,
                phase: Phase::Down,
            }),
            1 => {
                self.phone.compose = Compose::default();
                self.phone.compose.to.insert("12345");
                self.phone
                    .compose
                    .body
                    .insert("Hello Nokia!\n短信编辑测试。\n240 × 320");
                self.phone.compose.recipient = false;
                self.phone.go(Screen::Compose);
            }
            2 => self.phone.go(Screen::Browser),
            3 => {
                self.phone.go(Screen::Music);
                self.dispatch(Event {
                    key: Key::Center,
                    phase: Phase::Down,
                });
            }
            _ => {}
        }
        self.smoke_stage += 1;
    }
}
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut data_dir = storage::default_dir();
    let mut smoke = None;
    let mut args = std::env::args().skip(1);
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--data-dir" => data_dir = PathBuf::from(args.next().ok_or("--data-dir needs a path")?),
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
                    "cargo run -p phone-desktop -- [--data-dir PATH] [--smoke-test OUTPUT]\nArrows: navigate; Enter: center; F1/F2: softkeys; F3: call; Home: end/home; Tab: recipient/body; F4: input mode. Mouse left/right/middle = left/right/center key."
                );
                return Ok(());
            }
            _ => return Err(format!("Unknown option: {arg}").into()),
        }
    }
    let mailbox = storage::load(&data_dir)?;
    let now = Instant::now();
    let mut app = App {
        window: None,
        gpu: None,
        canvas: canvas::Canvas::new()?,
        phone: phone::Phone::new(mailbox),
        input: Input::default(),
        web: web::Web::new(),
        modifiers: ModifiersState::empty(),
        ime: false,
        ime_allowed: false,
        preedit: String::new(),
        save_retry_at: now,
        start: now,
        last: now,
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
