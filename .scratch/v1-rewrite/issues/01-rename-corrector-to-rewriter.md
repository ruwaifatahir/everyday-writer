# 01: Rename Corrector to Rewriter

**What to build:** A prefactor so the code uses the glossary's words. The rewrite interface, its prompt type, its error type, the genai-backed implementation and the fake are renamed from "correct/correction" to **Rewriter**, rewrite prompt and rewrite error. Doc comments and module docs talk about rewrites, selections and results, not corrections. Behaviour is unchanged.

Spec: `.scratch/v1-rewrite/spec.md`

**Blocked by:** None (can start immediately)

**Status:** ready-for-agent

- [ ] No type, function, test name or doc comment calls the feature "correct", "correction" or "fix" (the starter name "Fix mistakes only" is the only exception)
- [ ] Terms match `GLOSSARY.md`: Rewrite, Selection, Result, Replace, Instruction Set
- [ ] ADR 0001 names the Rewriter instead of the Corrector
- [ ] All existing tests pass under their new names, with no behaviour change
- [ ] `pnpm check` passes
