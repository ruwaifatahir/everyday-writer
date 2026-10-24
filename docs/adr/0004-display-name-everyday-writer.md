# The app is called "Everyday Writer"; the slug stays internal

From 1.2.0 the name people see (installer, Start menu, Installed apps, tray, windows, notifications) is "Everyday Writer", matching how Windows apps name themselves. `everyday-writer` stays only where nobody reads it: the Cargo package, the binary and the repo URLs. The identifier `com.ruwaifatahir.everydaywriter` is kept, because settings, the API key in Credential Manager and single-instance all hang off it.

## Considered Options

- **Installer code that removes the old copy:** rejected. The app was two days old with a handful of installs, so the extra NSIS code wasn't worth it.
- **Keep the slug as the product name and only fix in-app text:** rejected. The installer and Installed apps would still show the slug.

## Consequences

- The Windows installer keys its uninstall entry and install folder by product name, so 1.2.0 installs as a new app next to an existing `everyday-writer`. The 1.2.0 release notes ask people to uninstall the old one by hand. Until they do, both copies start at login and the second only opens Settings in the first.
- Never rename the product again without migration code; the same split would happen.
