# Project conventions

- Support macOS, Windows and Linux through the shared winit/wgpu host. Keep OS-specific adapters separate and document unsupported services.
- Keep firmware, private profiles and research captures outside this repository. README demo captures belong in `docs/screens/`.
- Keep dependency features limited to used services and native desktop targets. Font conversion (fontTools) and screenshot comparison (Pillow) are optional local tools, not CI/build/runtime requirements; check in generated font resources and their hashes.
- When changing the version, use `python3 scripts/version.py bump patch` (or `minor`, `major`, `X.Y.Z`) to update the workspace manifest and Cargo.lock together. Run locked checks.
- After an authorized version-change commit/push to `main`, CI must create the matching annotated `vX.Y.Z` tag and dispatch Release. Verify the tag and release workflow; never move or overwrite an existing version tag.
- Build/test success is not manual UI acceptance on another OS. Report the distinction.
