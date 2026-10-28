use crate::runtime::Time;
use crate::{config, gpu, host, platform, runtime::Runtime, storage};
#[cfg(feature = "desktop")]
#[path = "smoke.rs"]
mod smoke;
use super::keypad::{Keypad, Pointer, Viewport};
use std::{
    path::PathBuf,
    sync::Arc,
    time::{Duration, Instant},
};
use winit::{
    application::ApplicationHandler,
    dpi::LogicalSize,
    event::{ElementState, MouseButton, TouchPhase, WindowEvent},
    event_loop::{ActiveEventLoop, ControlFlow, EventLoop},
    keyboard::{KeyCode, ModifiersState, PhysicalKey},
    window::{Window, WindowId},
};
pub(crate) struct App {
    window: Option<Arc<Window>>,
    gpu: Option<gpu::Gpu>,
    runtime: Runtime,
    platform: platform::Native,
    modifiers: ModifiersState,
    save_retry_at: Instant,
    start: Instant,
    next_redraw: Instant,
    occluded: bool,
    store: crate::persistence::Directory,
    smoke: Option<PathBuf>,
    #[cfg(feature = "desktop")]
    smoke_stage: usize,
    failure: Option<String>,
    controls: config::Controls,
    keypad: Option<Keypad>,
    cursor: (f64, f64),
    suspended: bool,
    #[cfg(target_os = "android")]
    pub android: Option<winit::platform::android::activity::AndroidApp>,
    last_haptic: Option<Instant>,
}
impl App {
    pub fn new(
        runtime: Runtime,
        platform: platform::Native,
        store: crate::persistence::Directory,
        controls: config::Controls,
        smoke: Option<PathBuf>,
    ) -> Result<Self, String> {
        let now = Instant::now();
        Ok(Self {
            window: None,
            gpu: None,
            runtime,
            platform,
            modifiers: ModifiersState::empty(),
            save_retry_at: now,
            start: now,
            next_redraw: now,
            occluded: false,
            store,
            smoke,
            #[cfg(feature = "desktop")]
            smoke_stage: 0,
            failure: None,
            keypad: controls.keypad.then(Keypad::new).transpose()?,
            controls,
            cursor: (-1., -1.),
            suspended: false,
            last_haptic: None,
            #[cfg(target_os = "android")]
            android: None,
        })
    }
    pub fn run(mut self, event_loop: EventLoop<()>) -> Result<(), Box<dyn std::error::Error>> {
        event_loop.run_app(&mut self)?;
        if let Some(e) = self.failure {
            return Err(e.into());
        }
        Ok(())
    }
    fn viewport(&self) -> Viewport {
        let window = self.window.as_ref().unwrap();
        let size = window.inner_size();
        if cfg!(target_os = "android") && self.controls.keypad {
            return Viewport::fullscreen(size.width, size.height);
        }
        Viewport::fit(size.width, size.height, self.controls.keypad)
    }
    fn pointer_down(&mut self, id: Pointer, x: f64, y: f64) {
        let viewport = self.viewport();
        let Some(pad) = &mut self.keypad else {
            return;
        };
        if let Some(key) = pad.hit(viewport, x, y)
            && let Some(source) = pad.press(id, key)
        {
            self.runtime.press(source, key);
            if self.controls.haptics
                && self
                    .last_haptic
                    .is_none_or(|t| t.elapsed() >= Duration::from_millis(40))
            {
                self.last_haptic = Some(Instant::now());
                #[cfg(target_os = "android")]
                if let Some(app) = &self.android {
                    crate::android::feedback(app);
                }
            }
        }
    }
    fn pointer_up(&mut self, id: Pointer, cancel: bool) {
        if let Some(source) = self.keypad.as_mut().and_then(|p| p.release(id)) {
            if cancel {
                self.runtime.cancel_source(source);
            } else {
                self.runtime.release(source);
            }
        }
    }
    fn pointer_move(&mut self, id: Pointer, x: f64, y: f64) {
        let viewport = self.viewport();
        if let Some(pad) = &self.keypad
            && let Some(key) = pad.key(id)
            && pad.hit(viewport, x, y) != Some(key)
        {
            self.pointer_up(id, true);
        }
    }
    fn cancel(&mut self) {
        self.runtime.cancel_input();
        if let Some(pad) = &mut self.keypad {
            pad.clear();
        }
    }

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
        if Instant::now() < self.save_retry_at {
            return;
        }
        if let Err(error) = self.runtime.flush(&mut self.store, false) {
            self.save_retry_at = Instant::now() + Duration::from_secs(5);
            self.runtime.report(error);
        }
    }
    fn quit(&mut self, event_loop: &ActiveEventLoop) {
        if self.runtime.close(&mut self.store) {
            event_loop.exit();
        }
    }
    fn fail(&mut self, event_loop: &ActiveEventLoop, e: String) {
        eprintln!("{e}");
        self.failure = Some(e);
        event_loop.exit();
    }
}
impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        self.suspended = false;
        self.occluded = false;
        self.next_redraw = Instant::now();
        if self.window.is_some() {
            return;
        }
        let (width, height) = if self.controls.keypad {
            let available = event_loop.primary_monitor().map_or(864., |monitor| {
                monitor.size().height as f64 / monitor.scale_factor() - 96.
            });
            let height = available.clamp(576., 864.);
            (height * 240. / 576., height)
        } else {
            (480., 640.)
        };
        let attributes = Window::default_attributes()
            .with_decorations(self.controls.titlebar)
            .with_title("TA-1278 · Phone interaction")
            .with_inner_size(LogicalSize::new(width, height))
            .with_min_inner_size(LogicalSize::new(
                240.,
                if self.controls.keypad { 576. } else { 320. },
            ));
        #[cfg(target_os = "macos")]
        let attributes = {
            use winit::platform::macos::WindowAttributesExtMacOS;
            attributes.with_accepts_first_mouse(true)
        };
        match event_loop.create_window(attributes) {
            Ok(window) => {
                let window = Arc::new(window);
                window.set_ime_allowed(false);
                match pollster::block_on(gpu::Gpu::new(window.clone(), self.controls.keypad)) {
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
    fn suspended(&mut self, _: &ActiveEventLoop) {
        self.cancel();
        self.runtime.suspend_media();
        if let Err(e) = self.runtime.flush(&mut self.store, true) {
            self.runtime.report(e);
        }
        // Android invalidates NativeWindow on suspend. Drop its Surface before returning.
        self.gpu = None;
        self.window = None;
        self.suspended = true;
    }
    fn exiting(&mut self, _: &ActiveEventLoop) {
        self.runtime.suspend_media();
        if let Err(e) = self.runtime.flush(&mut self.store, true) {
            eprintln!("Save: {e}");
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
                self.cancel();
                if let Some(gpu) = &mut self.gpu {
                    gpu.resize(size.width, size.height);
                }
            }
            WindowEvent::Focused(focused) => {
                if !focused {
                    self.cancel();
                    self.modifiers = ModifiersState::empty();
                }
                // Android Pause itself is not forwarded by winit. Focus loss arrives before
                // backgrounding (and before NativeWindow destruction), so silence and save here.
                #[cfg(target_os = "android")]
                {
                    self.occluded = !focused;
                    if !focused {
                        self.runtime.suspend_media();
                        if let Err(e) = self.runtime.flush(&mut self.store, true) {
                            self.runtime.report(e);
                        }
                    } else {
                        self.next_redraw = Instant::now();
                    }
                }
            }
            WindowEvent::ModifiersChanged(m) => self.modifiers = m.state(),
            // Ignore even late IME events queued before the window disabled composition.
            WindowEvent::Ime(_) => {}
            WindowEvent::CursorMoved { position, .. } => {
                self.cursor = (position.x, position.y);
                self.pointer_move(Pointer::Mouse, position.x, position.y);
            }
            WindowEvent::CursorLeft { .. } => self.pointer_up(Pointer::Mouse, true),
            WindowEvent::Touch(touch) => match touch.phase {
                TouchPhase::Started => {
                    self.pointer_down(Pointer::Touch(touch.id), touch.location.x, touch.location.y)
                }
                TouchPhase::Moved => {
                    self.pointer_move(Pointer::Touch(touch.id), touch.location.x, touch.location.y)
                }
                TouchPhase::Ended => self.pointer_up(Pointer::Touch(touch.id), false),
                TouchPhase::Cancelled => self.pointer_up(Pointer::Touch(touch.id), true),
            },
            WindowEvent::MouseInput { state, button, .. } if self.keypad.is_some() => {
                if button == MouseButton::Left {
                    if state == ElementState::Pressed {
                        self.pointer_down(Pointer::Mouse, self.cursor.0, self.cursor.1);
                    } else {
                        self.pointer_up(Pointer::Mouse, false);
                    }
                }
            }
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
                // Winit 0.30 reports Android Back without a physical KeyCode.
                // Accept this named control only, never logical text from the host IME.
                #[cfg(target_os = "android")]
                if event.logical_key
                    == winit::keyboard::Key::Named(winit::keyboard::NamedKey::BrowserBack)
                {
                    if event.state == ElementState::Released {
                        self.release(0x30000);
                    } else if !event.repeat {
                        self.press(0x30000, phone_core::input::Key::SoftRight);
                    }
                    return;
                }
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
                        Some(host::input::Keyboard::Shell(key)) if !event.repeat => {
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
                #[cfg(feature = "desktop")]
                let before = self.gpu.as_ref().map_or(0, |g| g.presented);
                let viewport = self.viewport();
                self.runtime.render();
                if let Some(gpu) = &mut self.gpu
                    && let Err(e) = gpu.render(
                        self.keypad
                            .as_mut()
                            .map_or(self.runtime.canvas.pixels.as_slice(), |pad| {
                                pad.compose(&self.runtime.canvas.pixels)
                            }),
                        viewport,
                    )
                {
                    self.fail(event_loop, e);
                }
                #[cfg(feature = "desktop")]
                if !event_loop.exiting()
                    && self.smoke.is_some()
                    && self.gpu.as_ref().is_some_and(|g| g.presented > before)
                {
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
        if self.suspended || (cfg!(target_os = "android") && self.occluded) {
            event_loop.set_control_flow(ControlFlow::Wait);
            return;
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
        if let Err(error) = self.runtime.poll_platform(&mut self.platform) {
            self.runtime.report(error);
        }
        self.flush();
        if let Some(w) = &self.window {
            if let Err(e) = self.runtime.poll_services() {
                self.runtime.report(e);
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
