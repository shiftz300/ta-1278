//! Android entry point. Runtime/backend contracts are the same as on desktop.
use crate::{Runtime, Time, config, host, persistence, platform, storage};
use jni::{
    JavaVM,
    objects::{JObject, JValue},
};
use winit::{
    event_loop::EventLoop,
    platform::android::{EventLoopBuilderExtAndroid, activity::AndroidApp},
};

#[unsafe(no_mangle)]
fn android_main(app: AndroidApp) {
    if let Err(error) = run(app.clone()) {
        eprintln!("TA1278: {error}");
        let _ = java(&app, |env, activity| {
            let text = env.new_string(format!("TA1278: {error}"))?;
            env.call_method(
                activity,
                "failure",
                "(Ljava/lang/String;)V",
                &[JValue::Object(&text)],
            )?;
            Ok(())
        });
    }
}
fn run(app: AndroidApp) -> Result<(), Box<dyn std::error::Error>> {
    let internal = app
        .internal_data_path()
        .ok_or("Android internal storage unavailable")?;
    let profile = internal.join("profile");
    let media = app
        .external_data_path()
        .unwrap_or_else(|| internal.clone())
        .join("Phone");
    std::fs::create_dir_all(&media)?;
    let mut config = config::RuntimeConfig::read(&profile.join("config.json"), false)?;
    config.controls.keypad = true;
    // Video playback has no Android adapter yet.
    config.capabilities.video = false;
    let mut quiet = false;
    java(&app, |env, activity| {
        quiet = env.call_method(activity, "quiet", "()Z", &[])?.z()?;
        Ok(())
    })?;
    if quiet {
        config.capabilities.sounds = false;
        config.capabilities.music = false;
    }
    let controls = config.controls;
    let folder = platform::Folder::new(
        config.storage.directory.as_deref().unwrap_or(&media),
        config.storage.read_only,
    )?;
    let mut store = persistence::Directory::open(&profile)?;
    let runtime = Runtime::load_profile(
        config,
        Time {
            elapsed_ms: 0,
            unix_ms: storage::session::wall_ms(),
        },
        &mut store,
        Box::new(phone_core::services::Offline),
        persistence::Resume::WallClock,
    )?;
    let native = platform::Native::from_folder(&runtime, Some(folder))
        .with_decoder(std::sync::Arc::new(Decoder(app.clone())));
    let mut event_loop = EventLoop::builder();
    event_loop.with_android_app(app.clone());
    let mut host = host::App::new(runtime, native, store, controls, None)?;
    host.android = Some(app);
    host.run(event_loop.build()?)
}
fn java(
    app: &AndroidApp,
    call: impl FnOnce(&mut jni::JNIEnv<'_>, &JObject<'_>) -> jni::errors::Result<()>,
) -> Result<(), String> {
    // android-activity owns the JVM and global Activity reference while `app` lives.
    // JObject is borrowed here: it does not delete the reference on drop.
    let vm = unsafe { JavaVM::from_raw(app.vm_as_ptr().cast()) }.map_err(|e| e.to_string())?;
    let mut env = vm.attach_current_thread().map_err(|e| e.to_string())?;
    let activity = unsafe { JObject::from_raw(app.activity_as_ptr().cast()) };
    let result = call(&mut env, &activity).map_err(|e| e.to_string());
    if env.exception_check().unwrap_or(false) {
        let _ = env.exception_clear();
    }
    result
}
pub(crate) fn feedback(app: &AndroidApp) {
    // Java posts to the UI thread; never block the render/input thread or bypass user settings.
    let _ = java(app, |env, activity| {
        env.call_method(activity, "feedback", "()V", &[])?;
        Ok(())
    });
}

struct Decoder(AndroidApp);
impl platform::Decoder for Decoder {
    fn audio_duration(&self, files: &platform::Folder, path: &str) -> Result<f32, String> {
        platform::visual::SystemDecoder.audio_duration(files, path)
    }
    fn load(
        &self,
        files: &platform::Folder,
        path: &str,
        video: bool,
    ) -> Result<platform::Visual, String> {
        use std::io::Read;
        if video {
            return Err("此构建没有视频解码组件。".into());
        }
        let file = files.open(path).map_err(|e| e.to_string())?;
        const LIMIT: u64 = 16 * 1024 * 1024;
        if file.len > LIMIT {
            return Err("文件过大。".into());
        }
        let mut bytes = Vec::new();
        file.reader
            .take(LIMIT + 1)
            .read_to_end(&mut bytes)
            .map_err(|e| e.to_string())?;
        if bytes.len() > LIMIT as usize {
            return Err("文件过大。".into());
        }
        let mut pixels = Vec::new();
        java(&self.0, |env, activity| {
            let data = env.byte_array_from_slice(&bytes)?;
            let result = env
                .call_method(activity, "picture", "([B)[I", &[JValue::Object(&data)])?
                .l()?;
            if !result.is_null() {
                let array = jni::objects::JIntArray::from(result);
                let count = env.get_array_length(&array)? as usize;
                if (3..=240 * 240 + 2).contains(&count) {
                    pixels.resize(count, 0);
                    env.get_int_array_region(&array, 0, &mut pixels)?;
                }
            }
            Ok(())
        })?;
        if pixels.len() < 3 {
            return Err("无法解码此图片。".into());
        }
        let (width, height) = (pixels[0] as u32, pixels[1] as u32);
        if width == 0
            || height == 0
            || width > 240
            || height > 240
            || (width * height) as usize != pixels.len() - 2
        {
            return Err("无效的图片尺寸。".into());
        }
        let rgba = pixels[2..]
            .iter()
            .flat_map(|&argb| {
                let [a, r, g, b] = argb.to_be_bytes();
                [r, g, b, a]
            })
            .collect();
        image::RgbaImage::from_raw(width, height, rgba)
            .map(platform::Visual::Image)
            .ok_or("无效的图片。".into())
    }
}
