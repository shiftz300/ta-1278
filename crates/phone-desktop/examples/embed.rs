//! Run without a window, native adapter, network, audio output or profile directory.
use phone_runtime::{
    Request, Runtime, Time,
    config::RuntimeConfig,
    input::Key,
    messaging::{Backend, Capabilities, Command, Envelope, Event, PortError},
    persistence::{Record, Store},
    platform::{self, Documents, Instance, Page, Platform},
};
struct Inbox(Option<Envelope>);
impl Backend for Inbox {
    fn id(&self) -> &str {
        "example"
    }
    fn capabilities(&self) -> Capabilities {
        Capabilities {
            receive_messages: true,
            ..Default::default()
        }
    }
    fn submit(&mut self, _: Command) -> Result<(), PortError> {
        Err(PortError::Unsupported)
    }
    fn poll(&mut self) -> Result<Option<Envelope>, PortError> {
        Ok(self.0.clone())
    }
    fn acknowledge(&mut self, sequence: u64) -> Result<(), PortError> {
        if self
            .0
            .as_ref()
            .is_some_and(|event| event.sequence == sequence)
        {
            self.0 = None;
            Ok(())
        } else {
            Err(PortError::Unsupported)
        }
    }
}
struct LocalHost {
    instance: Instance,
    ready: Option<(Request, Result<Page, String>)>,
}
impl Platform for LocalHost {
    fn instance(&self) -> Instance {
        self.instance
    }
    fn documents(&mut self) -> Option<&mut dyn Documents> {
        Some(self)
    }
}
impl Documents for LocalHost {
    fn submit(&mut self, request: Request, _: Option<Page>) -> Result<(), String> {
        let page = platform::parse(
            &request.url,
            "<title>Local page</title><p>Content supplied by the host.</p><form action='about:reply'><input name='text'><button>Submit</button></form>",
        );
        self.ready = Some((request, Ok(page)));
        Ok(())
    }
    fn poll(&mut self) -> Option<(Request, Result<Page, String>)> {
        self.ready.take()
    }
    fn cancel(&mut self) {
        self.ready = None;
    }
}
#[derive(Default)]
struct Memory(Vec<u8>);
impl Store for Memory {
    fn read(&mut self, record: Record) -> Result<Option<Vec<u8>>, String> {
        Ok((record == Record::Snapshot && !self.0.is_empty()).then(|| self.0.clone()))
    }
    fn write(&mut self, record: Record, bytes: &[u8]) -> Result<(), String> {
        if record != Record::Snapshot {
            return Err("Example stores checkpoints only".into());
        }
        self.0 = bytes.to_vec();
        Ok(())
    }
}
fn capture(
    runtime: &mut Runtime,
    path: &std::path::Path,
) -> Result<(), Box<dyn std::error::Error>> {
    let frame = runtime.render();
    image::save_buffer(
        path,
        frame.pixels,
        frame.width,
        frame.height,
        image::ColorType::Rgba8,
    )?;
    Ok(())
}
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let output = std::path::PathBuf::from(
        std::env::args()
            .nth(1)
            .ok_or("Usage: embed OUTPUT_DIRECTORY")?,
    );
    std::fs::create_dir_all(&output)?;
    let clock = Time {
        elapsed_ms: 0,
        unix_ms: 1_793_000_000_000,
    };
    let mut config = RuntimeConfig::default();
    config
        .capabilities
        .disable("music,files,network,pictures,video,sounds")?;
    let backend = Inbox(Some(Envelope {
        sequence: 1,
        event: Event::MessageReceived {
            from: "1230000".into(),
            body: "Example message".into(),
            time_ms: clock.unix_ms,
        },
    }));
    let mut runtime = Runtime::with_backend(config.clone(), clock, Box::new(backend))?;
    capture(&mut runtime, &output.join("home.png"))?;
    runtime.press(1, Key::Center);
    runtime.release(1);
    capture(&mut runtime, &output.join("menu.png"))?;
    runtime.advance(Time {
        elapsed_ms: 100,
        unix_ms: clock.unix_ms + 100,
    })?;
    runtime.navigate("about:example")?;
    let mut host = LocalHost {
        instance: runtime.instance(),
        ready: None,
    };
    runtime.poll_platform(&mut host)?;
    capture(&mut runtime, &output.join("browser.png"))?;
    runtime.poll_services()?;
    // The same Store contract supports in-memory and durable host storage.
    let mut store = Memory::default();
    runtime.save_to(&mut store)?;
    runtime.poll_services()?;
    assert_eq!(runtime.mailbox().inbox.len(), 1);
    let mut restored = Runtime::restore_from(config, clock, &mut store, Box::new(Inbox(None)))?;
    assert_eq!(restored.mailbox().inbox.len(), 1);
    capture(&mut restored, &output.join("restored.png"))?;
    println!(
        "Rendered 4 frames and restored one synthetic message. No window, network or audio output."
    );
    Ok(())
}
