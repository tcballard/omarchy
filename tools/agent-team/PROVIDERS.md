# Provider execution contracts — 18 October 2026

Tom confirmed four subscriptions: **Codex, Claude, Grok and Kimi**. They are separate provider sign-ins. Detection of a CLI or a usage record does not establish entitlement, authentication, adapter compatibility or successful execution.

| Provider | Prototype status | Native integration boundary | Remaining work |
| --- | --- | --- | --- |
| Codex | Lead/reviewer, fixture-tested | `codex exec --json`, read-only sandbox, scoped MCP bridge; Omarchy-selected account home | Live XPS sign-in, CLI-version and two-agent acceptance |
| Claude | Lead/reviewer, fixture-tested | Print/stream-json, restricted tools/hooks, scoped MCP bridge; Omarchy-selected account home | Live XPS sign-in and `--restricted` compatibility acceptance |
| Grok | Discovered; execution blocked | Official `grok` headless or ACP; `GROK_HOME` and native sign-in | Implement its transport, permission/MCP isolation and terminal parser, then fixture/live acceptance |
| Kimi | Discovered; execution blocked | Current Kimi Code `kimi -p ... --output-format stream-json` or `kimi acp`; `KIMI_CODE_HOME` and native sign-in | Separate current/legacy detection, enforce read-only agent/tool and hook/MCP boundaries, parse terminal output, then fixture/live acceptance |

## Grok

Inspected official source `xai-org/grok-build` commit `2bdd1d6a6369de0e8c68132ea4539e9abd9e14a8`, particularly `crates/codegen/xai-grok-pager/docs/user-guide/14-headless-mode.md`, plus the official [headless guide](https://docs.x.ai/build/cli/headless-scripting), [permissions guide](https://docs.x.ai/build/features/permissions), and [CLI reference](https://docs.x.ai/build/cli/reference).

The source documents `--prompt-file` for externally supplied prompts: headless mode does **not** consume piped stdin as its prompt. The native streaming format and `streaming-messages-json` are distinct choices. The latter has assistant blocks and terminal `result` records similar to Claude, but that similarity is not a verified compatible adapter. Unknown events and error/turn-limit terminals must not become successful final results.

`--tools` is a built-in tool allowlist, but retains always-on MCP meta-tools. A Read/Grep/list-only flag is therefore insufficient on its own. A worker adapter must also isolate or deny external MCP servers, subagents, shell/write tools and executable hooks without mutating the user's normal config. Do not reuse Omarchy's interactive `bypassPermissions` launch mode. Omarchy already supports Grok named-account homes; pin the chosen home for the review and keep subscription auth inside the Grok CLI. A future ACP adapter must reject write/terminal permission requests and use cached native authentication without starting a login flow or falling back to an API key.

## Kimi

Inspected `MoonshotAI/kimi-code` commit `4a82a07f08d0b135c4349c20aae53a2d51e5c3fa`, its [command reference](https://moonshotai.github.io/kimi-code/en/reference/kimi-command), [custom agent contract](https://moonshotai.github.io/kimi-code/en/customization/agents.html), and [config overrides](https://moonshotai.github.io/kimi-code/en/configuration/overrides.html). The older `MoonshotAI/kimi-cli` repository is archived and points to this replacement. Its YAML agent files and print flags must not be assumed valid for current Kimi Code.

Current Kimi Code uses Markdown agent files with frontmatter. An explicit `--agent-file` outranks project agents and can enforce a Read/Grep/Glob allowlist; an omitted tools list grants all tools. `-p` runs under automatic permission policy and cannot be combined with `--plan`, so a planning prompt is not a read-only boundary. Tool restrictions are rechecked at execution, but hooks, MCP startup and provider configuration require separate review.

The current CLI uses `KIMI_CODE_HOME` (default `~/.kimi-code`) for config and native sign-in state. Omarchy's account-home helper presently supports Claude, Codex and Grok, not Kimi. Do not pass Kimi to that helper or invent a Kimi named-account registry. Preserve the explicit native Kimi home without copying its credentials. Verify the selected configured model is backed by Kimi Code OAuth: a configurable CLI can otherwise use a separate API provider, and a successful CLI call alone would not prove subscription use.

## Next implementation slice

1. Keep Codex/Claude lead roles initially. Add an explicit reviewer selector for Claude/Codex/Grok/Kimi, pin only the chosen providers, and constrain MCP delegation to that selection. This avoids requiring Claude to run a Codex→Kimi review.
2. Implement Grok/Kimi as reviewer-only adapters first, with bounded inputs, native account scope, strict read-only execution, terminal/error validation and process-group cancellation. Keep unsupported versions blocked rather than trying another provider.
3. Add fixtures for actual upstream message shapes, startup/config failures, authentication expiry, output limits, cancellation and no-result exits. Test no unrelated MCP, hook or write/terminal execution.
4. Complete XPS acceptance using the installed CLI versions and each provider's own sign-in; only then enable the corresponding execution selector. Expand lead roles separately once scoped MCP injection is verified for each CLI.

This records a concrete path to the four subscriptions. It does not claim Grok/Kimi execution is implemented or that live subscription interoperability has passed.
