# Prototype verification — 18 October 2026

Base: upstream Quattro `a6dcc31b6429fcf25e9e6bf90179f84c602cbf4e`. Feature branch: `tcballard/omarchy:feat/agents-orchestration`.

Implemented: first-party Agents Tasks view, Rust broker, Codex/Claude CLI execution, parent-scoped MCP delegation, consolidated lead result and inspectable worker results. Existing manifest, Main.qml, Agent.qml, provider UI components, account helpers and subscription defaults are unchanged. Reused extracted trial commit `1242066`; fixed cross-project retry keys, account pinning across delegation, inherited provider/account environment and false completion without an independent review.

Passed locally:

- `tools/agent-team/tests/run`: rustfmt, clippy with warnings denied, build, 12 Node tests and shell syntax checks. Eleven tests exercise the real Rust broker with deterministic provider CLI fixtures; one covers installing/restoring into a disposable desktop and rejecting changed upstream files or later edits.
- `bash test/shell.d/agents-panel-test.sh`: existing account/usage/keyboard behavior regression assertions.
- Qt 6.12 qmlformat parsed all five changed/added QML files. Four task components instantiated using minimal host-contract stubs. No real Quattro window rendered.
- `cargo build --release --locked --manifest-path tools/agent-team/Cargo.toml`: Linux x86_64 build.
- `git diff --check`.

Provider fixtures implement the CLI JSON output and MCP handshake needed to verify scheduling, not the providers' implementations. Neither subscription was contacted. No claim of real authentication, CLI compatibility, subscription billing or XPS acceptance. Unix/loopback transport used by each fixture is selected by this runtime; portable tests do not certify every host configuration.

Native acceptance remains: install on matching XPS Quattro revision; inspect the original subscriptions and account controls; exercise Tasks keyboard/mouse focus, scrolling, Escape and reopening; run both provider directions using actual sign-ins; test cancellation, limits/authentication failure, service restart and restore. Record installed Omarchy/CLI versions and results before expanding the trial.

Confirmed target subscriptions: Codex, Claude, Grok and Kimi. All four now appear in diagnostics; Grok/Kimi have no execution roles and start requests are explicitly rejected. PROVIDERS.md records current source/documentation contracts and remaining adapter work. Remaining for all four subscriptions: add explicit named-account selection if needed; verify each provider's documented noninteractive transport, authentication and permission contract; implement/test its adapter without credential conversion or silent fallback; repeat live desktop/provider acceptance. AgentWire and Wayfinder were inspected and their applicable boundaries are recorded in README.md; neither is required for this CLI-based first slice.
