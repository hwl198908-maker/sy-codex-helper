# Codex Config Schema

## Sources Checked

- Local `%USERPROFILE%\.codex\config.toml` was inspected on 2026-07-04.
- Local `%USERPROFILE%\.codex\auth.json` was inspected on 2026-07-04 without copying secret values.
- OpenAI Codex Config Reference: https://developers.openai.com/codex/config-reference
- OpenAI Codex Advanced Config: https://developers.openai.com/codex/config-advanced#custom-model-providers
- OpenAI Codex Authentication: https://developers.openai.com/codex/auth

## config.toml

### User-level location

Codex user-level config lives at `%USERPROFILE%\.codex\config.toml` on Windows, equivalent to `~/.codex/config.toml` in the docs.

### Managed keys for this app

The manager should write or update only these keys:

- `model`: selected default model.
- `model_provider`: selected provider id, using `custom` for the first version.
- `[model_providers.custom]`: custom provider table.
- `model_providers.custom.name`: display name.
- `model_providers.custom.base_url`: provider Base URL.
- `model_providers.custom.wire_api`: use `responses` for the Responses API path.
- `model_providers.custom.requires_openai_auth`: set `true` when the provider should use the same API key stored by Codex auth.
- `model_providers.custom.http_headers`: optional static headers, including optional User-Agent only if Codex accepts it in this map during implementation.

### First-version provider block

```toml
model = "gpt-5.5"
model_provider = "custom"

[model_providers.custom]
name = "custom"
base_url = "https://example.com/v1"
wire_api = "responses"
requires_openai_auth = true
```

### Protocol notes

Official Codex config currently documents `wire_api = "responses"` as the supported custom-provider protocol. The app UI can keep a Chat Completions option as product intent, but the config writer must not emit unsupported `wire_api` values until Codex documents them.

## auth.json

### File location

When file credential storage is used, Codex stores credentials at `%USERPROFILE%\.codex\auth.json`.

### API key shape observed locally

```json
{
  "OPENAI_API_KEY": "<redacted>"
}
```

The manager must never log or document the real key value.

### First-version write rule

For `requires_openai_auth = true`, write the entered API key into the `OPENAI_API_KEY` field in `auth.json`, preserving unrelated fields when present.

## Notes

- Do not define custom providers with reserved ids: `openai`, `ollama`, or `lmstudio`.
- Prefer `requires_openai_auth = true` for the first OpenAI-compatible proxy flow because it matches the local working config and official authentication guidance for LLM proxy servers backed by OpenAI auth.
- `experimental_bearer_token` exists but is documented as discouraged, so the first version should not use it.
- `env_key` is useful for environment-variable authentication, but it does not satisfy the product requirement that saving in the manager completes configuration without extra user environment setup.
- The config writer should back up existing `config.toml` and `auth.json` before modifying them.
