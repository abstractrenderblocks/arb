# arb Specification (v1)

This document defines the version 1 specification for **arb (Abstract Render Blocks)**.

arb is a deterministic recursive text compiler. This specification defines the behavior and guarantees of the compiler, not any particular implementation.

Status: **draft / design-locked sections only**

---

## 1. Purpose

arb exists to transform **structured data** and **template blocks** into one or more output files in a deterministic and repeatable way.

arb is designed for cases where:
- the same information must appear in multiple files
- consistency across outputs matters
- regeneration should be safe and predictable
- logic should live in templates, not in code execution

arb does not attempt to replace programming languages, scripting languages, or AI-based generators.

---

## 2. Core Guarantees

An arb compiler implementation MUST provide the following guarantees:

### 2.1 Determinism
Given:
- the same package
- the same input data
- the same compiler version

The output MUST be byte-for-byte identical.

### 2.2 No Code Execution
arb templates MUST NOT:
- execute arbitrary code
- invoke system commands
- evaluate expressions beyond template directives

arb is a rendering system, not a scripting language.

### 2.3 Schema Validation Before Rendering
Input data MUST be validated against the package schema before any template rendering occurs.

If validation fails:
- template rendering MUST NOT begin
- no output files may be produced

### 2.4 Safe Recursion
Recursive behavior (template includes, nested repeats) MUST be bounded by:
- include cycle detection
- maximum include depth
- maximum expansion depth
- maximum output size per file

### 2.5 Fail Fast
On the first encountered error:
- compilation MUST stop
- a clear error MUST be reported

Partial or silently degraded output is not allowed.

---

## 3. Core Concepts

### 3.1 Package

A **package** defines how structured data is transformed into output files.

A package consists of:
- `schema.yaml` — defines the data contract
- `templates/` — one or more template files
- optional `examples/` — example data files

Packages may be shared, versioned, open source, or proprietary.

arb treats packages as data; it does not impose licensing or distribution rules.

---

### 3.2 Data File

A **data file** is a YAML document providing values consumed by templates.

The structure of the data file MUST conform to the package schema.

arb does not infer structure from templates; the schema is authoritative.

---

### 3.3 Compile Run

A **compile run** consists of the following logical steps:

1. Load package schema
2. Load input data
3. Validate data against schema
4. Load and parse templates
5. Render templates using validated data
6. Write output files

Each step MUST complete successfully before the next begins.

---

## 4. Schema Language (v1)

The schema language defines the shape and requirements of input data.

The schema itself is written in YAML.

### 4.1 Supported Types

The following schema node types are supported in v1:

- `object`
- `string`
- `number`
- `boolean`
- `list`

No implicit type coercion is allowed.

---

### 4.2 Object Schema

An object schema defines a mapping of named fields.

Example:

```yaml
type: object
required: [name, version]
properties:
  name:
    type: string
  version:
    type: string
Rules:

Fields listed in required MUST be present in data

Fields not listed in required are optional

Each property defines the expected type of the field

---

### 4.3 List Schema

A list schema defines an ordered list of items.

Example:

Copy code
type: list
items:
  type: string
Rules:

Data MUST be a list

Each element MUST validate against the items schema

---

### 4.4 Scalar Schemas

Scalar schemas define leaf values.

Examples:

Copy code
type: string

type: number

type: boolean

---

### 4.5 Optional Fields

Fields are optional unless explicitly listed in required.

Templates MUST handle optional fields explicitly (e.g., via conditionals).

---

### 4.6 Descriptions

Schema nodes MAY include a description field.

Example:

tool_name:
  type: string
  description: "Name of the tool"
Descriptions are informational only and do not affect validation.

They may be used by tooling to generate starter data files or documentation.

---

### 5. Errors (Schema Phase)
Schema validation errors MUST report:

the path to the failing data element

the expected type or requirement

a clear explanation of the failure

Validation errors MUST be reported before any template processing occurs.

---

### 6. Specification Scope
This document defines v1 behavior only.

Future versions may extend the schema language or template system, but:

core guarantees MUST be preserved

v1-valid packages MUST remain supported



