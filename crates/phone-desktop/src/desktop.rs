use crate::{
    audio, config, host, platform,
    runtime::{Runtime, Time},
    storage,
};
use std::path::PathBuf;
use winit::event_loop::EventLoop;

pub fn run() -> Result<(), Box<dyn std::error::Error>> {
    let mut data_dir = storage::default_dir();
    let mut smoke = None;
    let mut config_path = None;
    let mut check_config = false;
    let mut disabled = Vec::new();
    let mut storage_dir = None;
    let mut read_only = false;
    let mut keypad = false;
    let mut titlebar = false;
    let mut haptics = true;
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
            "--keypad" => keypad = true,
            "--titlebar" => titlebar = true,
            "--no-haptics" => haptics = false,
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
                    "cargo run -p phone-desktop -- [--config JSON] [--check-config] [--keypad] [--titlebar] [--no-haptics] [--data-dir PATH] [--storage-dir DIRECTORY] [--read-only-storage] [--disable music,files,network,javascript,pictures,video,sounds] [--smoke-test OUTPUT]\nOffline tone preview: --render-tone NAME OUTPUT.wav (creates a new file)\nArrows: navigate; Enter: center; F1/F2: softkeys; F3: call history/dial; Home: end/home; Tab: recipient/body; # (Shift+3): input mode. Mouse left/right/middle = left/right/center key; with --keypad, click the drawn buttons. Titlebar is hidden by default; --titlebar restores it."
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
    runtime_config.controls.keypad |= keypad;
    runtime_config.controls.titlebar |= titlebar;
    runtime_config.controls.haptics &= haptics;
    let controls = runtime_config.controls;
    if check_config {
        println!("{}", serde_json::to_string_pretty(&runtime_config)?);
        return Ok(());
    }
    storage_dir = runtime_config.storage.directory.clone();
    read_only = runtime_config.storage.read_only;
    let mut store = crate::persistence::Directory::open(&data_dir)?;
    let folder = if smoke.is_none() {
        match storage::setup::select(&data_dir, storage_dir, read_only)? {
            Some(folder) => Some(folder),
            None => return Ok(()),
        }
    } else {
        None
    };
    let mut runtime = Runtime::load_profile(
        runtime_config,
        Time {
            elapsed_ms: 0,
            unix_ms: storage::session::wall_ms(),
        },
        &mut store,
        Box::new(phone_core::services::Offline),
        crate::persistence::Resume::WallClock,
    )?;
    if smoke.is_some() {
        host::theme(&mut runtime);
    }
    let platform = platform::Native::from_folder(&runtime, folder);
    #[cfg(target_os = "macos")]
    host::lifecycle::initialize()?;
    host::App::new(runtime, platform, store, controls, smoke)?.run(EventLoop::new()?)
}
