# Experimental native Agent Team

Feature branch of the full Omarchy fork. Extends `shell/plugins/agents/Panel.qml` in place; the manifest stays `omarchy.agents`. Usage extraction, account management, provider defaults and subscription meters stay upstream. Adds Subscriptions / Tasks navigation, a project and brief form, lead selection, running tasks, cancellation and combined results.

## XPS install

Run inside Omarchy Quattro on Linux x86_64, from a terminal:

```bash
git clone --depth 1 --branch feat/agents-orchestration https://github.com/tcballard/omarchy.git "$HOME/omarchy-agent-team-trial" && bash "$HOME/omarchy-agent-team-trial/tools/agent-team/scripts/install"
```

This builds the Rust helper locally (installing the native `rust` package if Cargo is absent), backs up the installed Agents panel, applies only the Agents additions and helper, starts a systemd user service, then restarts the existing shell. No whole-OS branch switch. The installer requires the Agents files from upstream Quattro `a6dcc31b6429fcf25e9e6bf90179f84c602cbf4e`; mismatches abort before changing installed files. It disables the earlier extracted trial if present, so there is only one active Agents interface. Stable upstream is not changed remotely. Local Omarchy updates may replace the overlay; restore before updating.

Open your existing Agents icon → Tasks. Check providers. Choose a specific, trusted project directory and lead provider. Start with a small review such as: “Review the error handling; delegate an independent review to the other provider and combine the concrete findings.” Both Codex and Claude must already be installed and signed in through their own CLIs. The helper never logs in, transfers tokens, switches accounts or chooses an alternate provider on failure. Accounts are pinned for the entire review when submitted; existing panel account controls still govern later reviews.

The lead receives bounded MCP tools to delegate to the other provider, poll its status and cancel it. The worker cannot delegate again or inspect unrelated task history. An omitted or failed independent review cannot produce a completed two-agent workflow. The lead writes the consolidated answer; individual child results remain inspectable.

## Restore

```bash
bash "$OMARCHY_PATH/tools/agent-team/scripts/restore"
```

Stops/removes the experimental user service and helper and restores the original panel. Retains account settings, usage data and private review history. Refuses to overwrite QML changed after installation. The earlier extracted trial stays disabled.

## Boundaries

Read-only review prototype, not autonomous code editing. Codex runs in its read-only CLI sandbox; Claude is limited to Read/Glob/Grep and the lead's broker tools, requires `--restricted` support, disables hooks and rejects an unsupported CLI. Provider CLI permissions remain the enforcement boundary. A project directory and a prompt are not a filesystem jail: use trusted repositories without secrets and do not treat this prototype as safe processing for hostile repository instructions. No API key routing, automatic failover, credential pooling or claim that subscriptions are interchangeable. Configured CLI credential helpers remain provider-owned; real subscription billing/authentication must be checked on the XPS.

Four concurrent tasks is a resource ceiling, not evidence of support for all four subscriptions. Four subscriptions may mean multiple accounts on the same provider; this build pins one selected account per provider, and does not yet expose per-task named-account selection. Execution adapters currently cover **Codex and Claude only**. The other subscriptions are now confirmed as **Grok and Kimi**. Diagnostics discovers all four, but Grok/Kimi execution remains blocked until their separate adapters and permission boundaries are implemented and tested. [Provider contracts and next slice](PROVIDERS.md) record the inspected upstream interfaces, modern/legacy Kimi split, account boundaries and acceptance requirements. Usage meters for other providers are preserved and do not imply execution support.

## Component fit

Inspected AgentWire at `6a4bd226a0e5c97346b82e12f3257b6e84a678ae` and WayfinderRouter at `075f17c5b3e50ed881a197fa95430f4e1cbc73f1`. AgentWire records/replays Codex App Server, ACP and MCP traffic; it is useful for future protocol diagnostics, but its proxy is not a task scheduler or an adapter for these CLI JSON streams. Wayfinder's gateway and Codex control surface route models/manage gateway-owned sessions; introducing it here would add another account/transport boundary without implementing task lifecycle. Neither is vendored as an unused dependency. Reuses the actual Omarchy account-home helper and the already published trial's Rust broker/MCP transport instead. MIT provenance: extracted trial `1242066`, built from the same upstream Agents revision.

## Verification and next acceptance

Run `tools/agent-team/tests/run` and `bash test/shell.d/agents-panel-test.sh`. Deterministic provider executables exercise real broker/MCP calls; they do not contact providers or consume subscriptions. Tests cover both lead directions, retry deduplication and project boundaries, cancellation, output budgets, missing results, authentication failures, restart, malformed history and account snapshots. Installer tests use a disposable fake desktop.

Before calling this usable on the XPS, check: original subscription meters/account controls; Tasks focus, scrolling, Escape and repeated opening; real Codex → Claude and Claude → Codex reviews; visible authentication/compatibility failures; cancellation; closing/reopening while work continues; shell restart; restore. Qt parsing and fixture tests do not establish real Quattro rendering, CLI compatibility or subscription interoperability. No live provider or XPS acceptance has been completed in this build environment.
