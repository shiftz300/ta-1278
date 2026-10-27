# Scope and limits

The project is a user-space phone interaction runtime and a desktop host. Research has moved out of this repository. Runtime resources, input code, tests and supported behavior are retained; full original-system reconstruction is not a release requirement.

Home, Messages, Contacts, Settings and Files are core applications. Other implemented apps are optional and enabled by default. Telephone has a local dialer and attempted-call history, without an active backend. See [configuration](config.md).

Messages support local sending/receiving state and a replaceable service interface. No real message or call backend is provided. The [browser](browser-capabilities.md) handles text pages and basic GET forms; complex web compatibility is excluded. Music plays local MP3/WAV files. Storage maps one user-selected folder directly. See [local features](local-completion.md) and [storage](storage.md).

The original T9 runtime and its dictionaries remain in the application. This does not imply complete semantic reconstruction of the algorithm. See [input](input-engine.md).

The shared runtime accepts keys, time, messages and documents and returns RGBA frames. Remaining embedding work includes transient UI snapshots, media pause/restore coordination, full replacement of media adapters, and native checks on other operating systems. Restoring user data starts at Home and does not rewind external IO. See [runtime](runtime.md).
