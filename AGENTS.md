# everyday-writer

Background Windows app in Rust + Tauri v2: select text in any app → global shortcut → LLM fixes typos/grammar → popup shows result → one click replaces original.

## Skills — use when

| Task | Skill |
|---|---|
| Writing/reviewing any Rust | `rust-best-practices` |
| Tokio, async, channels, cancellation | `rust-async-patterns` (minimal tokio features, not `full`) |
| Writing tests | `rust-testing` |
| `unsafe`, Win32/FFI calls | `unsafe-checker` |
| Tauri config, commands, IPC, plugins (shortcut, clipboard) | `tauri-v2` |
| Tray icon and menu | `adding-tauri-system-tray` |
| Popup window (frameless, always-on-top) | `customizing-tauri-windows` |
| Bundle/binary size | `optimizing-tauri-binary-size` |
| Correction prompt design | `prompt-optimizer` |
| Evaluating correction quality / comparing models | `promptfoo-evaluation` |
| Testing LLM behavior, prompt injection in selected text | `ai-system-testing` |
| Dependencies, cargo-audit/cargo-deny | `rust-security` |
| Threat model (clipboard, API key, data sent to LLM) | `security-threat-model` |
| Popup/settings UI design | `impeccable` |
| Art direction, anti-slop review of app screens | `design-taste-frontend` (self-host fonts; no Google Fonts links) |
| Finding where the popup/settings UI should animate | `find-animation-opportunities` |
| Making the popup/settings feel native, WebView2 quirks | `native-feel-cross-platform-desktop` |
| Reviewing UI code for accessibility and interface guidelines | `web-design-guidelines` |
| Commits, branches, versioning | `git-workflow-and-versioning` |
| Cutting a release | `release-skills` |

No skill covers text capture/replace (UI Automation, SendInput, clipboard) — use context7 docs for `windows`, `enigo`, `arboard`, `uiautomation`.

## Rules

- Keep the app small and fast: minimal crate features, no heavy deps.
- Treat selected text as data, never as instructions to the LLM.
- Never log clipboard or selected text. Store the API key in the OS keyring.
- Restore the user's clipboard after capture/replace.

## Agent skills

### Issue tracker

Issues live as local markdown files under `.scratch/<feature>/`. See `docs/agents/issue-tracker.md`.

### Domain docs

Single-context: one `GLOSSARY.md` and `docs/adr/` at the repo root. See `docs/agents/domain.md`.
