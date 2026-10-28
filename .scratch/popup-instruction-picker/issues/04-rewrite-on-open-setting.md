# 04: Rewrite on open setting

**What to build:** Settings gets a **Rewrite on open** switch in the Shortcut and startup section, on by default and applied at once like the other switches there. When it's on, the shortcut works as today. When it's off, the shortcut captures the selection and opens the popup on the Ready view, sending nothing to the AI until the user presses Rewrite or Enter. Nothing selected still leaves the popup hidden, and a selection over the limit still shows the too-long message. If the provider isn't set up, the message appears when the user presses Rewrite. Users upgrading from an older version get it on. See the spec, user stories 22 to 30.

**Blocked by:** 02 (Instruction set dropdown in the popup)

**Status:** ready-for-agent

- [ ] The settings gain Rewrite on open, default on; a saved file without it loads with it on
- [ ] The settings view the session reads exposes it
- [ ] With it off, the shortcut shows Ready with the selection and never calls the rewriter
- [ ] With it off, nothing selected stays hidden and a too-long selection shows the too-long message
- [ ] With it off and the provider not set up, the popup opens on Ready and Rewrite shows the not-set-up message with Open Settings
- [ ] Settings shows the switch with a short hint, saves it at once, and the next shortcut press follows it
- [ ] It is not in the tray menu
- [ ] Session tests cover the off cases above; a settings store test covers loading an older file
- [ ] The manual test checklist and the CHANGELOG's Unreleased section cover the setting
- [ ] `pnpm check` passes
