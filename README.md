# Stylus

An ANSI art and animation tool, and the ANSIapps theme's workshop. Free
and open source (MIT), by ansiapps. Built on
[icy_tools](https://github.com/mkrueger/icy_tools). macOS first.

It opens every classic ANSI and ASCII art format with SAUCE, draws by
hand, converts images, ASCII and text to ANSI, generates seeded art,
plays and makes ANSImations, and is where the ANSIapps theme pack is
drawn, checked for contrast and exported.

```sh
npm install
npm run tauri dev      # run the app
cargo test --manifest-path stylus-core/Cargo.toml
cargo test --manifest-path src-tauri/Cargo.toml
scripts/license-scan.py
scripts/compare-ansilove.sh ~/path/to/corpus   # needs ansilove
```

See `CLAUDE.md` for the rules and layout, and Diskette's
`docs/stylus-notes.md` for the design.

## License

MIT, for the app and the theme pack in `theme/`. The ANSIapps theme's font
is IBM VGA 8x16 by VileR, CC BY-SA 4.0, shipped unmodified in
`public/fonts/ansiapps/`. See `LICENSE`.
