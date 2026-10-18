# Experimental native Agent Team

Feature branch of the full Omarchy fork. Extends `shell/plugins/agents/Panel.qml` in place; the manifest stays `omarchy.agents`. Usage extraction, account management, provider defaults and subscription meters stay upstream. **Orchestrate** closes the panel and opens your default coding agent in your configured terminal through `omarchy-agent-prompt`, exactly like the Theme/Plugin starter actions.

The terminal session asks for the project, brief and reviewer, then submits the review to the Rust broker and reads its status/results. The broker still runs the existing read-only lead/reviewer workflow; the terminal conversation is its front end, not a second QML form. The embedded Tasks screen and its polling processes have been removed. The terminal handoff is a seeded agent conversation, so its conversational behavior still needs live acceptance; backend permission and account controls remain enforced in Rust.

## XPS install

Run inside Omarchy Quattro on Linux x86_64, from a terminal:

```bash
git clone --depth 1 --branch feat/agents-orchestration https://github.com/tcballard/omarchy.git "$HOME/omarchy-agent-team-trial" && bash "$HOME/omarchy-agent-team-trial/tools/agent-team/scripts/install"
```

This builds the Rust helper locally (installing the native `rust` package if Cargo is absent), backs up the installed Agents panel, applies only the Agents additions and helper, starts a systemd user service, then restarts the existing shell. No whole-OS branch switch. The installer requires the Agents files from upstream Quattro `a6dcc31b6429fcf25e9e6bf90179f84c602cbf4e`; mismatches abort before changing installed files. It disables the earlier extracted trial if present, so there is only one active Agents interface. Stable upstream is not changed remotely. Local Omarchy updates may replace the overlay; restore before updating.

Open your existing Agents icon → **Orchestrate**. Your normal default terminal and coding-agent session open. Tell it the specific project directory and review brief and choose an independent reviewer. Codex or Claude can lead; Grok can review. The conversation checks the broker's advertised capabilities and asks you to choose a supported lead if your current default cannot run the broker workflow. It does not silently switch accounts or providers.

Both chosen providers must already be installed and signed in. The Rust broker pins their accounts when submitted. Closing the terminal leaves broker jobs running; reopen Orchestrate to ask for their status or cancellation. A first-time or unavailable backend can be set up from that conversation with the existing bundled setup script; failures stay visible.

## Restore

```bash
bash "$OMARCHY_PATH/tools/agent-team/scripts/restore"
```

Stops/removes the experimental user service and helper and restores the original panel. Retains account settings, usage data and private review history. Refuses to overwrite QML changed after installation. The earlier extracted trial stays disabled.

## Boundaries

Read-only review prototype, not autonomous code editing. Codex runs in its read-only CLI sandbox; Claude is limited to Read/Glob/Grep and the lead's broker tools, requires `--restricted` support, disables hooks and rejects an unsupported CLI. Grok uses its native read-only sandbox, Read/Grep/list tools, deny-by-default permissions and MCP denial; unknown CLI/inspect formats are rejected. Provider CLI permissions remain the enforcement boundary. A project directory and a prompt are not a filesystem jail: use trusted repositories without secrets and do not treat this prototype as safe processing for hostile repository instructions. No API key routing, automatic failover, credential pooling or claim that subscriptions are interchangeable. Configured CLI credential helpers remain provider-owned; real subscription billing/authentication must be checked on the XPS.

Four concurrent tasks is a resource ceiling, not evidence of support for all four subscriptions. The four subscriptions are Codex, Claude, Grok and Kimi. This build pins one selected account per chosen provider and does not expose per-task named-account selection. Execution adapters cover **Codex and Claude as leads/reviewers, and Grok as a reviewer**. Diagnostics discovers all four. Grok is available as a reviewer with native read-only sandbox, extension preflight and terminal-result validation. Kimi execution remains blocked. [Provider contracts and next slice](PROVIDERS.md) record the inspected upstream interfaces, modern/legacy Kimi split, account boundaries and acceptance requirements. Grok reviews require an extension-free selected config/project: active hooks, MCP, LSP or enabled plugins cause a visible refusal; no config is automatically edited. Usage meters for other providers are preserved and do not imply execution support.

## Component fit

Inspected AgentWire at `6a4bd226a0e5c97346b82e12f3257b6e84a678ae` and WayfinderRouter at `075f17c5b3e50ed881a197fa95430f4e1cbc73f1`. AgentWire records/replays Codex App Server, ACP and MCP traffic; it is useful for future protocol diagnostics, but its proxy is not a task scheduler or an adapter for these CLI JSON streams. Wayfinder's gateway and Codex control surface route models/manage gateway-owned sessions; introducing it here would add another account/transport boundary without implementing task lifecycle. Neither is vendored as an unused dependency. Reuses the actual Omarchy account-home helper and the already published trial's Rust broker/MCP transport instead. MIT provenance: extracted trial `1242066`, built from the same upstream Agents revision.

## Verification and next acceptance

Run `tools/agent-team/tests/run` and `bash test/shell.d/agents-panel-test.sh`. Deterministic provider executables exercise real broker/MCP calls; they do not contact providers or consume subscriptions. Tests cover both lead directions, retry deduplication and project boundaries, cancellation, output budgets, missing results, authentication failures, restart, malformed history and account snapshots. Installer tests use a disposable fake desktop.

Before calling this usable on the XPS, check: original subscription meters/account controls; Orchestrate mouse/keyboard launch, panel dismissal, default terminal and agent selection, conversational input and reopening; real Codex → Claude, Claude → Codex, Codex → Grok and Claude → Grok reviews; visible authentication/compatibility failures; cancellation; closing/reopening while work continues; shell restart; restore. Qt parsing and fixture tests do not establish real Quattro rendering, CLI compatibility or subscription interoperability. No live provider or XPS acceptance has been completed in this build environment.

## Update an existing trial

If this feature is already installed, restore the overlay before installing the new checkout. This keeps the original upstream backup intact:

```bash
git -C "$HOME/omarchy-agent-team-trial" pull --ff-only && bash "$OMARCHY_PATH/tools/agent-team/scripts/restore" && bash "$HOME/omarchy-agent-team-trial/tools/agent-team/scripts/install"
```

This stops any running prototype reviews. Their history is retained; interrupted tasks are not automatically replayed.
