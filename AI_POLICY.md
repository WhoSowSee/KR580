# AI Policy

AI assistance is allowed. You remain responsible for everything you submit,
including code, tests, documentation, and reports.

## Review and verification

- Read the full diff and understand how the change fits into KR580. Be prepared
  to explain your decisions during review.
- Check suggested APIs, CLI flags, and references against the implementation or
  primary documentation. Check emulator behavior against the relevant documents
  under `docs/` and regression tests.
- Run the applicable checks in
  [CONTRIBUTING.md](CONTRIBUTING.md#checking-your-work). Report only checks you
  actually ran and explain any you skipped.
- For a bug fix, add or update a regression test when the behavior can be
  automated. Check that its assertions cover the intended behavior. Use the
  [testing guide](docs/testing.md) for platform and manual UI checks.
- Remove unrelated edits, unused helpers, and generated boilerplate before
  submitting.

For documentation and translations, check facts and examples and edit the
wording yourself. For CPU, device, or file-format claims, identify the program,
initial state, or input used to verify them.

## Disclosure

If AI generated or substantially rewrote code, tests, documentation, or technical
explanations, disclose that assistance in the PR or issue description. Include:

- The tool or model used, if known.
- Which parts it helped produce.
- How you reviewed and verified the result.

A short note is enough; chat transcripts are not required. Minor autocomplete
suggestions and spelling corrections do not need a separate note.

For contributions authored by the project maintainer (`@WhoSowSee`), a separate
AI disclosure note is optional. Review and verification requirements still apply.

## Issues and review replies

For bug reports, include what you observed and how to reproduce it. Mark
suspected causes as hypotheses. Feature requests should describe your use case
and the proposed behavior.

Read and verify AI-assisted descriptions and review replies before posting them.
Address the actual question or feedback.

The maintainer may request changes or close a contribution that lacks human
review, contains unsupported claims, or cannot be explained by its author.
