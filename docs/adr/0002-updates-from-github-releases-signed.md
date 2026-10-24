# Updates come from GitHub Releases, signed with our own key, installed only on the user's click

Installed copies find an update through a static `latest.json` at `https://github.com/ruwaifatahir/everyday-writer/releases/latest/download/latest.json` and install it with the Tauri updater plugin. Every update is signed with a private key we hold; each installed copy has the matching public key built in and refuses anything else. We chose a static file on GitHub Releases over our own update server because the app is open source, already ships there, and a plain GET sends nothing about the user. We chose ask-first over silent installs because the app quits during install, which must never interrupt a rewrite or cost the user their clipboard.

## Consequences

- The URL and the public key are baked into every installed copy. Changing either means shipping one last update from the old setup that points to the new one; losing the private key means existing installs can never update again and users must reinstall by hand. The key and its password are backed up in a password manager and on a USB drive, besides the GitHub secret.
- `releases/latest` ignores drafts, so publishing a draft release is the moment an update reaches users.
- The update signature is separate from Windows code signing; it does nothing for the SmartScreen warning on first install.
