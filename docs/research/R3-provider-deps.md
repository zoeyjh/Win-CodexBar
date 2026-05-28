# R-3: Provider Dependency Graph

## Providers to KEEP
- `claude`
- `codex`
- `copilot`

## Providers to DELETE (46)
abacus, alibaba, alibabatokenplan, amp, antigravity, augment, azureopenai, bedrock, codebuff, commandcode, crof, cursor, deepgram, deepseek, doubao, elevenlabs, factory, gemini, grok, groq, infini, jetbrains, kilo, kimi, kimik2, kiro, llmproxy, manus, mimo, minimax, mistral, nanogpt, ollama, openai, openaiapi, opencode, opencodego, openrouter, perplexity, stepfun, synthetic, t3chat, venice, vertexai, warp, windsurf, zai

## Files Requiring Modification

### Rust Core
- `rust/src/providers/mod.rs` (lines 5-54) — remove 46 module declarations
- `rust/src/core/provider.rs` — `pub enum ProviderId` (lines 13-63), `all()` (65-118), CLI name map (121-173), display name map (176-228), cookie domains (231+)
- `rust/src/core/provider_factory.rs` — iterates ProviderId::all()
- `rust/src/settings.rs` — provider_order, get_all_providers_status()
- `rust/src/settings/tests.rs` — length assertions vs ProviderId::all()
- `rust/src/cli/config.rs` — for id in ProviderId::all()
- `rust/src/cli/usage.rs` — ProviderSelection::All

### React UI (20+ files)
- `apps/desktop-tauri/src/App.tsx`
- `apps/desktop-tauri/src/floatbar/FloatBar.tsx`
- `apps/desktop-tauri/src/components/MenuSurface.tsx`
- `apps/desktop-tauri/src/components/MenuCard.tsx`
- `apps/desktop-tauri/src/components/ProviderGrid.tsx`
- `apps/desktop-tauri/src/hooks/useProviders.ts`
- `apps/desktop-tauri/src/lib/demoProviders.ts`
- `apps/desktop-tauri/src/lib/providerCharts.ts`
- `apps/desktop-tauri/src/components/providers/providerIcons.ts`
- `apps/desktop-tauri/src/components/providers/ProviderIcon.tsx`
- `apps/desktop-tauri/src/surfaces/Settings.tsx`
- `apps/desktop-tauri/src/surfaces/TrayPanel.tsx`
- `apps/desktop-tauri/src/surfaces/PopOutPanel.tsx`
- `apps/desktop-tauri/src/surfaces/settings/providers/*`

## Complexity: HIGH
Not just directory deletion. Requires careful editing of enum, factory, settings, CLI, and 20+ UI files.
