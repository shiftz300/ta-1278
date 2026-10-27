# Runtime configuration

Use one JSON file for application switches, host capabilities and phone-storage setup. The default enables every existing optional application. [Full example](../config/default.json).

```sh
cargo run --release --locked -p phone-desktop -- --config config/default.json
# Validate and print configuration without a window, folder chooser or audio:
target/release/phone-desktop --config config/default.json --check-config
```

Without `--config`, the app reads `config.json` from its private profile (`~/Library/Application Support/TA1278 Replica/` on macOS, or `--data-dir`). A missing default file uses defaults. An explicitly named missing file, unknown key, unsupported version or invalid JSON stops startup without replacing the file. The limit is 64 KiB. Omitted fields keep their defaults. Restart to apply changes; there is no live reload.

For example:

```json
{
  "version": 1,
  "apps": {
    "radio": false,
    "recorder": false,
    "counter": false
  },
  "capabilities": {
    "sounds": false
  },
  "storage": {
    "directory": "./phone-files",
    "read_only": true
  }
}
```

## Application flags

Home, Messages, Contacts, Settings and Files remain available; they have no application flag. Telephone provides local dialing and attempted-call history without a transport. The excluded Alipay and Ximalaya entries remain absent.

| `apps` key | Application |
|---|---|
| `browser` | Browser |
| `music` | Music |
| `calendar` | Calendar |
| `alarms` | Alarms |
| `snake` | Snake |
| `radio` | FM radio UI, still without hardware reception |
| `flashlight` | Flashlight UI |
| `recorder` | Recorder UI, still duration records without microphone audio |
| `calculator` | Calculator |
| `utilities` | Other: timer, stopwatch and converter |
| `counter` | Counter |
| `pictures` | Picture viewing and image capability |
| `video` | Video viewing and video capability |

Disabled applications disappear from the main menu. Their direct routes are blocked, stored Home shortcuts are hidden, and their shortcut choices are disabled. The Other menu also removes a disabled flashlight. Configuration does not delete messages, playlists, contacts, preferences or application state. Re-enabling an application makes its retained state available again.

Disabled alarms/calendar do not create new reminders; their pending reminders do not take focus or play sound. Pending reminders now record their source. Old pending reminders cannot reliably be classified, so they remain saved but suppressed unless **both** alarms and calendar are enabled. Due snoozes remain saved while suppressed. Disabling utilities suppresses timer alerts and stops live timer/stopwatch ticks; existing restart elapsed-time accounting remains unchanged. Re-enabling can therefore surface an expired saved timer.

## Capabilities and storage

`capabilities` controls `music`, `files`, `network`, `javascript`, `pictures`, `video`, `sounds`. These control services, independently of whether an application's entry is present. `javascript: false` (or `--disable javascript`) retains the static browser without running page scripts. Disabling an application also disables its corresponding music/network/image/video service. `sounds: false` mutes notification/effect/ringtone output; music and video playback have separate controls. To prohibit all playback, also disable `music` and `video` capabilities.

`storage.directory` selects the one mapped folder. It must exist; a relative path is resolved beside the JSON file. `null` retains the saved folder or first-launch chooser. `storage.read_only` blocks file mutations, not private profile saves.

CLI `--storage-dir` overrides the configured directory. `--read-only-storage` and `--disable ...` can further restrict the JSON settings regardless of argument order. `--data-dir` still selects the private profile. `--check-config` prints the configuration with those CLI overrides applied and exits before profile locking, data loading, device access or media startup.

The runtime config is not an application database: phone settings and user state remain in their existing private JSON files. App switches are runtime flags and do **not** remove code or assets from the binary.
