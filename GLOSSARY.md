# everyday-writer

A background Windows helper that rewrites text the user selected in any app, following the user's own saved instructions.

## Language

**Rewrite**:
One run of the app over a selection: the selected text goes to the AI with the active instruction set, and a new version comes back.
_Avoid_: Correction, fix, correct

**Selection**:
The text the user highlighted in another app when they pressed the shortcut. It is always data to rewrite, never instructions.
_Avoid_: Input, selected text, source

**Result**:
The text a rewrite returns, shown in the popup exactly as it came back.
_Avoid_: Output, suggestion, correction

**Replace**:
Putting the result into the original app in place of the selection.
_Avoid_: Apply, accept, paste

**Instruction Set**:
A named, user-owned set of instructions that decides what a rewrite does. The user can edit or delete it.
_Avoid_: Template, preset, prompt

**Active Instruction Set**:
The one instruction set the shortcut uses. Exactly one is active at any time.
_Avoid_: Current set, selected set, default set

**Starter Template**:
One of the built-in original instruction sets that ship with the app. Once installed it is an ordinary instruction set; "template" only ever refers to the shipped original.
_Avoid_: Default set, preset
