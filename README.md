# Abstract Render Blocks (arb)

**One source of truth. Many outputs.**

arb is a **deterministic recursive text compiler** for structured generation.

It transforms validated structured data (YAML) and template packages into one or more output files in a **repeatable, predictable, and safe** way.

arb is not a scripting language, not a templating engine with logic, and not an AI generator.
It is a compiler with strict guarantees.

---

## Why arb exists

Most generation systems fail in one or more of these ways:

- logic leaks into templates
- outputs drift out of sync
- partial output is produced on error
- regeneration is unsafe
- templates become unmaintainable over time

arb is designed for cases where:

- the same information must appear in many files
- consistency matters more than convenience
- regeneration must be safe
- failures must be explicit and early

---

## Core guarantees

arb provides the following guarantees:

- **Deterministic output**  
  Same inputs → same bytes.

- **Schema-first validation**  
  Input data is validated before any template is rendered.

- **No code execution**  
  Templates cannot execute code or invoke system commands.

- **Safe recursion**  
  Includes are cycle-checked and bounded by depth and size limits.

- **Fail-fast behavior**  
  On error, no partial output is produced. arb does not delete previous output unless a compile completes successfully.

These guarantees are defined in [`SPEC.md`](SPEC.md).

---

## Packages

arb itself is a compiler.  
All real functionality lives in **packages**.

### 📦 docs-suite (flagship package)

The repository ships with one full-featured package:

packages/docs-suite

**docs-suite** turns a single validated YAML file into a complete, shippable documentation set:

- `README.md`
- `docs/overview.md`
- `docs/quickstart.md`
- `docs/configuration.md`
- `docs/faq.md`
- `docs/troubleshooting.md`
- `CONTRIBUTING.md`
- `SECURITY.md`
- `CODE_OF_CONDUCT.md`
- `CHANGELOG.md`
- `man/*.1`
- optional `mkdocs.yml` and MkDocs-ready docs

All outputs are generated from one source of truth.

### 📦 cli-rust

A Rust CLI generation package:

packages/cli-rust

**cli-rust** generates a fully working Rust command-line interface using
`clap`, from a single validated YAML definition.

It produces:

- Rust `clap` command and subcommand enums
- nested subcommands (e.g. `tool remote add`)
- arguments, flags, and options
- generated help and usage output
- structured command dispatch scaffolding

This package demonstrates arb being used as a **code generator**, including
support for nested command trees.

### 📦 config-schema

A configuration definition and validation package:

packages/config-schema

**config-schema** turns a single configuration specification into multiple
artifacts that stay in sync by construction.

It produces:

- `config.example.yaml` (human-readable example config)
- `docs/config.md` (configuration reference documentation)
- `config.schema.json` (JSON Schema for machine validation)

This package demonstrates arb being used to generate configuration examples,
documentation, and validation from a single source of truth.

---

## Try it in 60 seconds

### 1. Install

```cmd
### Install (from source)

```cmd
git clone https://github.com/AbstractRenderBlocks/arb.git
cd arb
cargo install --path crates/arb-cli
```
> `arb-cli` will be published to crates.io once the v0.1 release is tagged.

### 2. Create a project directory

```cmd
mkdir arb-demo
cd arb-demo
```

### 3. Copy an example data file

```cmd
copy ..\packages\docs-suite\examples\data.example1.yaml data.yaml
```

### 4. Compile

```cmd
arb compile --package ..\packages\docs-suite --data data.yaml --out out
```

### 5. Inspect output

```cmd
dir out
type out\README.md
type out\man\arb.1
```

---

## CLI commands

arb validate — Validate input data against a package schema

arb compile — Validate data and render templates into output files

arb init — Generate a starter data file from a package schema

---

## Status

arb v1 is intentionally minimal and design-locked.
The goal is correctness, safety, and predictability — not feature breadth.

Future versions may add capabilities, but v1 packages will remain supported.

---

#License

Apache-2.0