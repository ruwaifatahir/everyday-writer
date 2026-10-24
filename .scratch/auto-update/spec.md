Labels: ready-for-agent

# Auto-update

## Problem Statement

Once installed, everyday-writer never changes. When a new version ships, the user only finds out by visiting the GitHub Releases page by chance, then has to download the installer and click through it again. Most people never do, so they keep running old versions with bugs that are already fixed. The app also never says which version it is, so the user can't even tell they're behind.

## Solution

The app checks GitHub for an update by itself: about 10 seconds after it starts, then every 24 hours. When it finds one, it shows one quiet Windows notification ("everyday-writer 1.1.0 is available"), and the tray item "Check for updates" becomes "Update to 1.1.0". Nothing installs until the user asks.

Clicking the notification (while it is on screen) or the tray item opens Settings at a new **Updates** section. It shows the installed version, the "Check for updates automatically" switch, a **Check for updates** button and, when there is one, "Version 1.1.0 is available" with **Update now**. Update now downloads the update with a progress readout, then a small installer progress window appears with no questions, and the app restarts on the new version. A single notification confirms "Updated to 1.1.0".

A manual check from the tray or the Settings button shows its result only inside Settings: "Checking…", then "You're up to date" or the available update. The app never updates while a rewrite or replace is running, and never throws away unsaved Settings changes without asking.

## User Stories

### Finding out

1. As a user, I want the app to check for an update on its own, so that I stay current without thinking about it.
2. As a user, I want the first check to wait about 10 seconds after the app starts, so that startup with Windows stays fast.
3. As a user, I want the app to check again every 24 hours while it runs, so that I find out about an update even if I never restart my PC.
4. As a user, I want one quiet Windows notification when an automatic check finds an update, so that I notice it without being interrupted.
5. As a user, I want that notification only once per version, so that the app doesn't nag me every day about the same update.
6. As a user, I want a new notification when an even newer version appears, so that I learn about 1.2.0 even if I ignored 1.1.0.
7. As a user, I want the tray item to read "Update to 1.1.0" when an update is known, so that I can still find it after the notification is gone.
8. As a user, I want clicking the notification while it's on screen to open Settings at Updates, so that I land exactly where I can act.
9. As a user, I want automatic checks to stay silent when they fail (offline, GitHub down), so that a network blip never bothers me.
10. As a privacy-minded user, I want the check to send nothing about me, so that updating doesn't track me.
11. As a privacy-minded user, I want to turn automatic checks off in Settings, so that the app never contacts GitHub unless I ask.
12. As a user who turned automatic checks off, I want "Check for updates" in the tray to keep working, so that I can still update when I choose.
13. As a user who turned automatic checks off after an update was found, I want "Update to 1.1.0" to stay in the tray, so that switching off future checks doesn't hide what's already known.

### Checking by hand

14. As a user, I want a "Check for updates" item in the tray menu, so that I can check right now.
15. As a user, I want that item to open Settings at Updates and show "Checking…", so that I can see it's working.
16. As a user, I want "You're up to date" when there's nothing new, so that I know the check worked.
17. As a user, I want "Version 1.1.0 is available" with Update now when there is something, so that I can update in the same place.
18. As a user, I want a Check for updates button in the Updates section, so that I can check again without going back to the tray.
19. As a user, I want a manual check that fails to say "Couldn't check for updates" in Settings, so that I know to try later.
20. As a user, I want a manual check never to pop a Windows notification, so that I'm not told twice about something I'm already looking at.

### Knowing my version

21. As a user, I want Settings to show "everyday-writer 1.0.0", so that I know which version I'm running.

### Installing

22. As a user, I want nothing to install until I click Update now, so that the app never closes on me unexpectedly.
23. As a user, I want the tray item and the notification to only open Settings, never install, so that a stray click can't restart the app.
24. As a user, I want Update now to show "Downloading… 45%", so that I know how long it will take.
25. As a user, I want the Update now button and the tray item disabled while downloading, so that I can't start it twice.
26. As a user, I want the install to run without installer pages, only a small progress window, so that updating is one click.
27. As a user, I want no admin prompt during an update, so that I don't need to approve anything.
28. As a user, I want the app to restart on the new version by itself, so that I don't have to find and relaunch it.
29. As a user, I want one notification "Updated to 1.1.0" after the restart, so that I know it worked.
30. As a user, I want no "Updated" notification on a fresh install, so that the first run isn't noisy.
31. As a user, I want my settings, instruction sets, API key and start-with-Windows choice kept across the update, so that nothing needs setting up again.

### Not losing work

32. As a user, I want Update now to wait until a running rewrite or replace has finished, so that the app never quits halfway and leaves my clipboard unrestored.
33. As a user with unsaved changes in Settings, I want to be asked "Save your changes before updating?" with Save / Discard / Cancel, so that my edits aren't silently lost.
34. As a user, I want Cancel to leave everything as it was, so that I can finish editing first.
35. As a user, I want a result still showing in the popup to simply close when I update, so that the update isn't blocked by something I can rerun.

### When things go wrong

36. As a user, I want a failed download or a failed signature check to show "Update failed. Try again." in Settings, so that I know it didn't happen.
37. As a user, I want the old version to keep working after a failed update, so that a bad network never breaks the app.
38. As a user, I want "Update to 1.1.0" back in the tray after a failure, so that I can retry later.
39. As a user, I want the app to refuse any update not signed with the project's key, so that nobody can push me a fake update even if the download is tampered with.
40. As a user, I want the app never to move to an older version, so that an update can't downgrade me.

### Releasing (maintainer)

41. As the maintainer, I want each release build to produce the signed installer and `latest.json` automatically, so that publishing an update needs no manual file work.
42. As the maintainer, I want releases to stay drafts until I publish them, so that I can test a build before any user gets it.
43. As the maintainer, I want the signing key protected by a password and backed up offline, so that losing my laptop never strands existing installs.
44. As the maintainer, I want CI to fail if the signing secrets are missing on a release, so that I never publish an update existing installs can't verify.
45. As the maintainer, I want a manual test step that updates an installed older build to the new one, so that I know the real install-and-restart path works before publishing.

## Implementation Decisions

### Release and delivery

- Follows ADR 0002: updates come from a static `latest.json` at `https://github.com/ruwaifatahir/everyday-writer/releases/latest/download/latest.json`, verified with the project's own signing key, installed only on the user's click.
- Add `tauri-plugin-updater` (2.x), registered Rust-side only, like the other plugins. No JS guest package and no frontend updater permissions: the frontend talks to our own commands. Confirm at build time that no `updater:*` capability is needed for Rust-only use; if Tauri requires one, grant it to no window.
- Keep the dependency small: default features off, with only the TLS feature that matches the `reqwest` already in the graph. Leave out `zip` (not needed for NSIS without `v1Compatible`). Check `cargo tree -d` and the installer size before and after. `cargo deny check` must pass; add any new licenses to the allowlist only after review.
- Bundle config: `createUpdaterArtifacts: true`. Updater config: the endpoint above, the public key, and `windows.installMode: "passive"`. The NSIS install stays per-user (the default), so updates need no admin prompt.
- Release workflow: pass `TAURI_SIGNING_PRIVATE_KEY` and `TAURI_SIGNING_PRIVATE_KEY_PASSWORD` from repo secrets to the build. Turn on `uploadUpdaterJson`, keep `uploadUpdaterSignatures`, and set `updaterJsonPreferNsis: true`. Keep `releaseDraft: true`. Add a step that fails the job if the secrets are empty.
- `CONTRIBUTING.md` gets the one-time key setup (generate, password, back up to a password manager and USB, add the two secrets). It also gets a release step: before publishing a draft, update an installed older build to it.
- `latest.json` `notes` may be left empty: "what's new" is out of scope.

### The update controller (one new deep module)

The only module that knows the update rules. It mirrors the rewrite session: it receives events, talks to the outside only through ports, and is tested with fakes.

Events in:
- `Started`: the app launched.
- `Tick`: a scheduled check is due. The controller owns the schedule: first check 10 seconds after `Started`, then every 24 hours, only while automatic checks are on.
- `CheckRequested`: from the tray item or the Settings button.
- `UpdateNowRequested`: from Settings, after any unsaved-changes prompt is resolved in the page.
- `AutoCheckChanged(bool)`.

Ports:
- **Update source**: `check() -> Result<Option<AvailableUpdate>, CheckError>` and `install(update, progress) -> Result<Never, InstallError>`. A successful install does not return: the real one exits the process. `AvailableUpdate` holds only the version (and an opaque handle). The real implementation wraps the Tauri updater with a timeout.
- **Notifier**: `update_available(version)` and `updated_to(version)`. Exactly these two Windows notifications exist.
- **Updates view**: renders an `UpdatesState` to the Settings section and the tray item label.
- **Busy gate**: `wait_until_idle()`. The real one is backed by the rewrite session, which gains a way to report that no rewrite or replace is running and to wait for that. While the gate is held, a new shortcut press shouldn't start a rewrite.
- **Update memory**: `auto_check: bool` (default on), `last_notified_version`, `last_run_version`.

State shown, a tagged enum serialized for the Settings page like `PopupState`:
- `Idle` (unknown, nothing found yet)
- `Checking`
- `UpToDate`
- `Available { version }`
- `Downloading { percent }`
- `Failed { kind: Check | Install }`

The tray shows "Update to {version}" for `Available` and `Failed{Install}`, a disabled "Downloading update…" while downloading, and otherwise "Check for updates".

Rules:
- An automatic check notifies only if the version found is newer than `last_notified_version`, and then records it. A manual check never notifies.
- Failed automatic checks leave the visible state as it was and log only a non-identifying error kind. Failed manual checks show `Failed{Check}`.
- Turning auto-check off cancels the schedule but keeps a known `Available` state.
- Update now: wait for the busy gate, close the popup, then install with progress.
- On `Started`: if `last_run_version` exists and is older than the running version, send `updated_to(running)`. Then always record the running version.
- Comparison is plain semver, newer only. That is the updater default, so downgrades are impossible.

### Wiring

- Tray menu model: a new item between "Instruction set" and "Settings", with its own id and `TrayAction`, labelled from `UpdatesState`. `refresh()` rebuilds it when the state changes.
- Notifications: register the notification plugin's click handler at startup, before any toast is shown. A click on the update-available toast opens Settings at Updates. Clicks on other toasts (such as shortcut failed) keep their current behaviour. A click is only reported while the toast is on screen. That limit is accepted, because the tray item is the reliable path.
- The settings window gains a way to open scrolled to Updates. If it is already open, it focuses and scrolls there.
- New commands for the Settings page: get the update state and version, check now, update now, set auto-check. The page listens for update-state events. Each command gets an allow permission in the settings capability and an entry in the handler list.
- Update memory lives in the existing settings file under new fields with serde defaults, so older files still load. Saving the Settings form must not overwrite these fields.
- Exit path: the updater exits the process itself after launching the installer, so the existing exit-request guard is not involved. Clipboard safety comes from the busy gate, not from an exit hook.

### Settings page

- A new **Updates** section in the existing plain HTML and TypeScript page, matching its styles: version line, switch, Check for updates button, status line, Update now button. The switch applies immediately, like start-with-Windows.
- Unsaved-changes prompt (Save / Discard / Cancel) lives in the page, which already knows the form's dirty state. Only after Save or Discard does it call update now.

### Security notes (for the threat model)

- Assets: the signing private key and the integrity of what installed copies run.
- Fake updates are blocked by the signature check against the built-in public key. A tampered `latest.json` or installer is rejected.
- A stolen private key plus control of the release would allow a malicious update. Mitigations: password-protected key, GitHub secrets, manual publish of drafts, 2FA on the GitHub account.
- The check sends no identifiers beyond what any HTTPS GET reveals (IP, user agent). The URL template uses no `{{current_version}}`.

## Testing Decisions

- Good tests here drive the update controller through its events and assert only what the user would see. That means the `UpdatesState` rendered, the notifications sent, whether install was called and in what order relative to the busy gate, and what was stored in memory. Never assert internal fields.
- **Update controller**: fake update source (returns an update, nothing, or an error; reports scripted progress; records installs), fake notifier, fake view, fake busy gate (can be held busy), in-memory update memory. Timers run under Tokio's paused clock (`start_paused`), never real sleeps. Cases cover every rule above:
  - first check at 10 s, then every 24 h
  - no schedule when auto-check is off
  - notify once per version, and again for a newer one
  - manual checks never notify
  - silent automatic failures
  - Update now waits for idle
  - install failure returns to Available with `Failed{Install}`
  - "Updated to" only after a real version increase, and not on first install
- Prior art: the rewrite session's tests (fake ports, oneshot gates to hold a step open, `Notify` to observe a step starting).
- **Tray menu model**: extend the existing label and `action()` tests for the new item in each state.
- **Settings file**: a file without the new fields loads with auto-check on and empty memory, and saving the form keeps the memory fields. Prior art: the existing settings store tests.
- **Not automated** (added to `docs/manual-test-checklist.md`, run on the installed NSIS build):
  - Install an older build, publish a test release, and confirm the notification.
  - Click the notification and confirm Settings opens at Updates.
  - Run Update now and confirm the progress window, the restart and the "Updated to" toast.
  - Confirm settings, key and autostart survive the update.
  - Answer the unsaved-changes prompt with each of Save, Discard and Cancel.
  - Start an update mid-rewrite and confirm it waits.
  - Run checks offline.
  - Swap in a tampered `latest.json` signature and confirm the install is refused.

## Out of Scope

- Beta or prerelease channels.
- "Skip this version".
- Release notes or "what's new" before or after updating.
- Silent or forced updates, minimum-version gates, staged rollouts.
- Catching notification clicks from the Action Center or after a restart (needs a COM activator).
- Windows code signing and the first-install SmartScreen warning.
- winget, Scoop or Microsoft Store distribution, and detecting those installs.
- macOS and Linux.

## Further Notes

- The PRD (`docs/PRD.md`) lists "Update notices inside the app" as not included, and the v1 spec marks in-app update notices out of scope. Update both when this ships.
- v1.0.0 installs have no updater, so their users must install the first updater-enabled version by hand. Mention this in that release's notes and the README.
- The notification plugin docs still say actions are mobile-only. The 2.6.0+ changelog and source show desktop click reporting. Verify on the installed build early, because story 8 depends on it.
- A released build can't be updated if the private key is lost (see ADR 0002).
