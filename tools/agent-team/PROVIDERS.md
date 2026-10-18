# Provider execution contracts — 18 October 2026

Tom confirmed four subscriptions: **Codex, Claude, Grok and Kimi**. They are separate provider sign-ins. Detection of a CLI or a usage record does not establish entitlement, authentication, adapter compatibility or successful execution.

| Provider | Prototype status | Native integration boundary | Remaining work |
| --- | --- | --- | --- |
| Codex | Lead/reviewer, fixture-tested | `codex exec --json`, read-only sandbox, scoped MCP bridge; Omarchy-selected account home | Live XPS sign-in, CLI-version and two-agent acceptance |
| Claude | Lead/reviewer, fixture-tested | Print/stream-json, restricted tools/hooks, scoped MCP bridge; Omarchy-selected account home | Live XPS sign-in and `--restricted` compatibility acceptance |
| Grok | Reviewer implemented, fixture-tested | Native headless Messages JSON, read-only sandbox, `GROK_HOME` and native sign-in | Live XPS orchestration and installed-version acceptance |
| Kimi | Discovered; execution blocked | Current Kimi Code `kimi -p ... --output-format stream-json` or `kimi acp`; `KIMI_CODE_HOME` and native sign-in | Separate current/legacy detection, enforce read-only agent/tool and hook/MCP boundaries, parse terminal output, then fixture/live acceptance |

## Grok

Inspected official source `xai-org/grok-build` commit `2bdd1d6a6369de0e8c68132ea4539e9abd9e14a8`, particularly `crates/codegen/xai-grok-pager/docs/user-guide/14-headless-mode.md`, plus the official [headless guide](https://docs.x.ai/build/cli/headless-scripting), [permissions guide](https://docs.x.ai/build/features/permissions), and [CLI reference](https://docs.x.ai/build/cli/reference).

The implemented adapter uses `--prompt-file` for externally supplied prompts: headless mode does **not** consume piped stdin as its prompt. The native streaming format and `streaming-messages-json` are distinct choices. The adapter shares assistant-text parsing but separately requires a nonempty terminal `result` with `subtype: success`, `is_error: false` and `stop_reason: end_turn`. Intermediate text, unknown result shapes, missing terminals and turn-limit/error records fail visibly.

`--tools` is a built-in tool allowlist, but retains always-on MCP meta-tools. A Read/Grep/list-only flag is therefore insufficient on its own. The adapter runs `grok inspect --json` in the chosen project/account and refuses active hooks, MCP servers, LSP servers, enabled plugins, config warnings, invalid layers or missing inventory. At launch it selects `--sandbox read-only`, Read/Grep/list built-ins, `dontAsk` permissions, MCP denial and no subagents, memory, planning, leader process or auto-update. Compatibility MCP/hook sources and managed MCP discovery are disabled for the child environment. It never edits the user configuration. Inspection is a startup preflight, not a filesystem sandbox or protection against concurrent changes to the user's configuration; the native read-only sandbox enforces project write protection. Do not reuse Omarchy's interactive `bypassPermissions` launch mode. Omarchy already supports Grok named-account homes; pin the chosen home for the review and keep subscription auth inside the Grok CLI. A future ACP adapter must reject write/terminal permission requests and use cached native authentication without starting a login flow or falling back to an API key.

## Kimi

Inspected `MoonshotAI/kimi-code` commit `4a82a07f08d0b135c4349c20aae53a2d51e5c3fa`, its [command reference](https://moonshotai.github.io/kimi-code/en/reference/kimi-command), [custom agent contract](https://moonshotai.github.io/kimi-code/en/customization/agents.html), and [config overrides](https://moonshotai.github.io/kimi-code/en/configuration/overrides.html). The older `MoonshotAI/kimi-cli` repository is archived and points to this replacement. Its YAML agent files and print flags must not be assumed valid for current Kimi Code.

Current Kimi Code uses Markdown agent files with frontmatter. An explicit `--agent-file` outranks project agents and can enforce a Read/Grep/Glob allowlist; an omitted tools list grants all tools. `-p` runs under automatic permission policy and cannot be combined with `--plan`, so a planning prompt is not a read-only boundary. Tool restrictions are rechecked at execution, but hooks, MCP startup and provider configuration require separate review.

The current CLI uses `KIMI_CODE_HOME` (default `~/.kimi-code`) for config and native sign-in state. Omarchy's account-home helper presently supports Claude, Codex and Grok, not Kimi. Do not pass Kimi to that helper or invent a Kimi named-account registry. Preserve the explicit native Kimi home without copying its credentials. Verify the selected configured model is backed by Kimi Code OAuth: a configurable CLI can otherwise use a separate API provider, and a successful CLI call alone would not prove subscription use.

## Implemented Grok slice and deferred work

Codex/Claude retain lead roles. The Tasks UI now selects an independent reviewer explicitly; Grok is reviewer-only. The broker pins only the chosen lead/reviewer accounts and enforces that choice on each delegated request, including retry-key comparisons. A missing chosen CLI or a failed Grok review does not fall back to another provider. Grok is never routed through the Claude execution adapter, and no credentials are copied or converted. The native API-key environment is removed from the Grok child; actual subscription entitlement and provider-owned configuration still require live acceptance.

Fixture coverage includes both lead→Grok directions, selected-account pinning, removal of unselected provider dependencies, preflight extension/config refusals, mismatched reviewer attempts, final-result validation, authentication errors, output limits and cancellation. These fixtures run the real Rust broker and MCP bridge; they are not real Grok subscription calls. Tom confirmed Grok runs natively on his XPS; orchestration-specific acceptance has not yet been observed there.

Kimi remains detected but execution is disabled at Tom's request. No Kimi adapter or automatic fallback is added. Grok lead roles are also deferred until scoped MCP injection is implemented and verified.
