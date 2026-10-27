# Build size

Release builds use `opt-level = "z"`, fat LTO, one codegen unit, stripped symbols and abort-on-panic. Debug artifacts and `target/` are development outputs, not the distributable app.

The main payloads are original and fallback fonts, machine code and linked libraries, the sound bank, T9 code/dictionaries, icons and small audio resources. Fonts, T9 data and the sound bank are losslessly compressed at build time with roundtrip checks. Optional independent dictionaries remain in the source tree but are removed from the standard release executable by dead-code elimination.

```sh
sh scripts/package-macos.sh
python3 scripts/report-package-size.py
```

The report is written to `target/packed-build-size.json`. It measures the actual signed bundle, locates packed resource bytes, verifies decompression and separates Mach-O code sections from other data. Bundle metadata, helper code and license/source notices are included. It requires the built macOS bundle and its matching Cargo build outputs. This is a size measurement, not a performance benchmark.

Full firmware, research reports, raw captures and Cargo caches are not packaged. Historical measurements and comparisons live in the separate research archive.
