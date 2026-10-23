# everyday-writer

Background Windows app: select text in any app, press a shortcut, and an LLM fixes typos and grammar. One click replaces the original.

## Build

Prerequisites: Rust via rustup (stable MSVC), Microsoft C++ Build Tools ("Desktop development with C++"), WebView2, Node 24 and pnpm.

```sh
pnpm install
pnpm tauri dev     # run the app
pnpm tauri build   # build the NSIS installer
```

## License

Dual-licensed under [MIT](LICENSE-MIT) or [Apache-2.0](LICENSE-APACHE), at your option.
