# web-site

The **web-site** package generates a complete static website using Abstract Render Blocks.

It is intentionally minimal, deterministic, and opinionated. The goal is not visual polish, but to demonstrate how ARB can be used to generate real, shippable artifacts — in this case, a documentation website — from a single validated data source.

This package is also a canonical example of using ARB to document ARB itself.

---

## What this package generates

Given a single `data.yaml` file, this package generates a small static site consisting of:

- Home page (merged overview + philosophy)
- How It Works (execution model and failure behavior)
- Building Packages (full authoring guide)
- Packages (inventory of available ARB packages)
- 404 page
- Shared site styling (`site.css`)

All pages are generated from plain `.arb` templates with no runtime dependencies.

---

## Why this package exists

This package demonstrates that ARB is not limited to code or configuration generation.

Using the same tool, schema discipline, and template system, you can:

- Generate documentation
- Generate websites
- Generate configuration
- Generate code
- Generate release notes

If you can describe it as structured data and render it deterministically, ARB can generate it.

---

## Package layout

packages/web-site/
schema.yaml
templates/
index.html.arb
how-it-works.html.arb
building-packages.html.arb
packages.html.arb
not-found.html.arb
site.css
examples/
data.yaml


Notes:

- Templates are plain text HTML with ARB directives.
- Files starting with `_` are treated as non-output templates (partials).
- The example data file is complete and should compile without modification.

---

## How to use

From the repository root:

```bash
arb compile \
  --package packages/web-site \
  --data packages/web-site/examples/data.yaml \
  --out out
```

The generated site will be written to out/.

No additional build steps are required.

Design principles

This package intentionally follows these rules:

No schema inference

No implicit defaults

No template logic beyond {var}, {rep}, {if}, and {inc}

No partial output

No silent failures

If something is missing or malformed, the compile should fail.

## Status

This package is considered complete.

It is not meant to be a general-purpose website generator.
It is meant to be a clear, working example of what ARB enables when packages are designed correctly.