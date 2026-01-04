# config-schema

This package demonstrates how **arb** can generate configuration artifacts from a single source of truth (schema + data).

From one config definition, it generates:

- `config.example.yaml` — a readable example config file
- `docs/config.md` — human documentation for config keys
- `config.schema.json` — JSON Schema for machine validation

The point is to keep config examples, docs, and validation in sync **by construction**.

## Files

- `schema.yaml` — schema for config definitions used by this package
- `examples/config.yaml` — example config definition data
- `templates/` — templates that generate the outputs

## Try it

From the repo root:

```cmd
arb compile --package packages/config-schema ^
  --data packages/config-schema/examples/config.yaml ^
  --out out-config
```

Then look in:
- out-config/config.example.yaml
- out-config/docs/config.md
- out-config/config.schema.json