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

## 3.4 Project Layout (Recommended)

arb is designed to be run from within a **project directory**.

A project directory represents one logical use of arb (for example: documentation generation, code scaffolding, or configuration output).

### Recommended Project Structure

project/
  data.yaml        # input data for the package
  out/             # generated output files


Additional files (such as README files, notes, or version control metadata) may exist alongside these, but arb only operates on files explicitly referenced at runtime.

arb does not require a fixed project configuration file in v1.

--- 

### 3.5 Packages and Package Resolution

Packages define how structured data is rendered into output files.

A package consists of:

schema.yaml

templates/ directory

optional examples/ directory

---

# Package Resolution Order

When a package name is referenced, arb resolves it using the following order:

Local package directory
If a packages/<package-name>/ directory exists relative to the project root, it is used.

User package cache
If not found locally, arb looks for the package in its managed user cache.

Explicit package path (if provided)
An explicit filesystem path to a package may be used to bypass resolution.

If a package cannot be resolved, compilation fails.

---

### 3.6 Package Cache

arb maintains a user-local package cache to store installed or downloaded packages.

The package cache location is implementation-defined, but MUST be:

user-specific

isolated from project directories

managed entirely by arb

Typical locations include:

Windows: %LOCALAPPDATA%\arb\packages\

macOS: ~/Library/Application Support/arb/packages/

Linux: ~/.local/share/arb/packages/

Users are not required to interact with the package cache directly.

---

### 3.7 Multiple Projects

arb supports multiple projects by design.

Each project:

resides in its own directory

references packages independently

produces output only within its specified output directory

arb does not maintain global project state beyond the package cache.

This design prevents cross-project interference and filesystem clutter.

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

## 5. Template Language (v1)

arb templates are plain text files with embedded directives called **tags**.

The template language is intentionally minimal. It provides structure and repetition without introducing scripting, evaluation, or side effects.

---

## 5.1 Template Files

- Template files use the `.arb` extension.
- Templates are UTF-8 encoded text.
- Any text not part of a tag is copied verbatim to the output.

---

## 5.2 Tags Overview

The following tags are supported in v1:

| Tag | Purpose |
|---|---|
| `{var}path{/var}` | Insert a value |
| `{rep}path{/rep}` | Repeat a block for each item in a list |
| `{if}path{/if}` | Conditionally render a block |
| `{inc}file.arb{/inc}` | Include another template |

Tags MUST be properly nested and closed.

---

## 5.3 Paths

Paths are used to reference data values.

### Path Syntax
- Dot-separated identifiers: `tool_name`, `commands.name`
- `.` refers to the **current context**

Paths are resolved relative to the current context unless otherwise specified.

Invalid paths result in a compile-time error.

---

## 5.4 Variable Tag

### Syntax
{var}path{/var}

css
Copy code

### Behavior
- Resolves `path` to a value in the current context.
- Inserts the string representation of the value into the output.
- If the value does not exist, compilation fails.

Example:

Project name: {var}tool_name{/var}

---

### 5.5 Repeat Tag
Syntax

Copy code
{rep}path
  ... body ...
{/rep}

# Behavior

path MUST resolve to a list.

The body is rendered once for each item in the list.

During each iteration:

the current list item becomes the new context

the previous context is restored after the iteration completes

Example:


{rep}commands
- {var}name{/var}
{/rep}

---

### 5.6 Current Context (.)

Inside a {rep} block, the current item becomes the active context.

If the current item is a scalar value, it is accessed using:

{var}.{/var}

Example:

{rep}flags
- {var}.{/var}
{/rep}

---

### 5.7 Conditional Tag

Syntax

{if}path
  ... body ...
{/if}

# Behavior
Renders the body only if the value at path is truthy.

If the value does not exist, it is treated as false.

# Truthiness Rules
A value is considered true if:

boolean true

non-empty string

non-zero number

non-empty list

object with at least one field

Otherwise, the value is false.

Example:

{if}homepage
Homepage: {var}homepage{/var}
{/if}

---

### 5.8 Include Tag

Syntax

{inc}relative/path/file.arb{/inc}

# Behavior
Includes another template at the current position.

The included template is rendered using the current context.

Include paths are literal strings (no variable substitution in v1).

Include resolution rules and safety constraints are defined separately.

---

### 5.9 Nesting Rules

Tags MAY be nested.

{rep} and {if} tags define a nested scope.

{var} and {inc} are leaf tags.

Improperly nested or unclosed tags result in a compile-time error.

---

### 5.10 Error Handling (Template Phase)

Template errors MUST report:

template file name

line and column (where available)

tag type

referenced path or include target

clear explanation of the failure

Examples of template errors:

referencing a missing value

repeating over a non-list

invalid nesting

include file not found

Template errors MUST halt compilation immediately.

---

### 5.11 Language Scope

The template language intentionally excludes:

expressions

arithmetic

string manipulation

function calls

variable assignment

user-defined logic

Any future extensions MUST preserve the core guarantees defined in this specification.

---

## 6. Includes and Limits (v1)

This section defines how template includes work and the limits enforced to ensure safe and predictable compilation.

---

## 6.1 Include Resolution

The include tag inserts the rendered contents of another template file.

### Include Syntax
{inc}relative/path/file.arb{/inc}

markdown
Copy code

### Resolution Rules
- Include paths are **literal strings**.
- Include paths are resolved **relative to the directory of the including template**.
- Path separators use `/` and are normalized for the host operating system.
- The resolved path MUST remain within the package `templates/` directory.
- Attempts to escape the `templates/` directory result in a compile-time error.

---

## 6.2 Include Context

- Included templates are rendered using the **current context**.
- No new scope is introduced by an include.
- All variable, repeat, and conditional behavior applies normally inside the included template.

Example:

{rep}commands
{inc}partials/command.arb{/inc}
{/rep}

Inside command.arb, {var}name{/var} refers to the current command.

---

### 6.3 Include Cycles

arb MUST detect include cycles.

An include cycle occurs when a template directly or indirectly includes itself.

# Behavior
When a cycle is detected, compilation MUST fail.

The error MUST report the include chain that caused the cycle.

Example cycle:

README.md.arb → header.arb → README.md.arb

---

### 6.4 Include Depth Limit

arb enforces a maximum include depth to prevent runaway recursion.

Default
Maximum include depth: 32

Behavior
If the include depth exceeds the maximum, compilation MUST fail.

The error MUST indicate that the include depth limit was exceeded.

Implementations MAY allow this limit to be configured, but a default MUST exist.

---

### 6.5 Expansion Depth Limit

arb enforces a maximum expansion depth for nested template directives.

Expansion depth includes:

nested {rep} blocks

nested {if} blocks

nested includes

# Default
Maximum expansion depth: 128

# Behavior
If the expansion depth exceeds the maximum, compilation MUST fail.

The error MUST report that the expansion depth limit was exceeded.

---

### 6.6 Output Size Limit

arb enforces a maximum output size per generated file.

Default
Maximum output size per file: 10 MB

# Behavior
If the rendered output exceeds the maximum size, compilation MUST fail.

The error MUST report:

the output file name

the configured size limit

the size reached before termination

---

### 6.7 Asset Copying

In addition to compiling .arb template files, arb handles non-template assets.

# Rules
Any file within the templates/ directory that does not end with .arb MUST be copied verbatim to the output directory.

Directory structure under templates/ MUST be preserved.

Existing files in the output directory MAY be overwritten.

This allows packages to include static assets such as stylesheets or images.

---

### 6.8 Failure Semantics

For all include- and limit-related errors:

compilation MUST stop immediately

no partial output MUST be produced

a clear error message MUST be emitted

arb MUST NOT attempt to continue compilation after a limit violation or include error.

---

## 7. CLI Behavior (v1)

arb is primarily operated via a command-line interface.

The CLI provides a minimal set of commands that expose the core compiler pipeline without embedding project-specific logic.

---

## 7.1 General CLI Rules

- arb commands MUST be deterministic.
- Commands MUST NOT modify input data or package contents.
- Commands MUST NOT rely on global mutable state beyond the package cache.
- All commands MUST return a non-zero exit code on failure.

---

## 7.2 Command Overview

The following commands are defined in v1:

| Command | Purpose |
|---|---|
| `arb validate` | Validate input data against a package schema |
| `arb compile` | Validate data and render templates into output files |
| `arb init` | Generate a starter data file from a package schema |

---

## 7.3 Common Options

The following options are common to all commands where applicable:

| Option | Description |
|---|---|
| `--package <name-or-path>` | Package name or explicit path |
| `--data <file>` | Path to input data file |
| `--out <dir>` | Output directory |
| `--verbose` | Enable verbose diagnostic output |
| `--quiet` | Suppress non-error output |

Paths MAY be relative or absolute.

---

## 7.4 `arb validate`

### Purpose
Validate input data against the package schema without rendering templates.

### Behavior
- Loads the package schema
- Loads the input data file
- Validates data against the schema
- Reports validation success or failure

### Output
- On success: a confirmation message
- On failure: one or more schema validation errors

### Side Effects
- No files are created or modified

---

## 7.5 `arb compile`

### Purpose
Validate input data and generate output files from templates.

### Behavior
- Performs all steps of `arb validate`
- Loads and parses templates
- Renders templates using validated data
- Writes output files to the output directory

### Output
- Generated files written to the output directory
- Optional diagnostic output if enabled

### Side Effects
- Existing files in the output directory MAY be overwritten

---

## 7.6 `arb init`

### Purpose
Generate a starter data file based on the package schema.

### Behavior
- Loads the package schema
- Generates a data file skeleton:
  - required fields MUST be included
  - optional fields MAY be included with placeholder values
- Writes the generated file to the specified path

### Output
- A YAML data file suitable for use with `arb validate` and `arb compile`

---

## 7.7 Error Handling

All CLI errors MUST:
- write a clear message to standard error
- include the command name
- include relevant file paths
- exit with a non-zero status code

Errors MAY include:
- schema validation errors
- template compilation errors
- include resolution errors
- filesystem errors

---

## 7.8 Exit Codes

- `0` — success
- `1` — validation or compilation error
- `2` — usage or argument error
- `>2` — implementation-defined fatal errors

---

## 7.9 CLI Stability

The v1 CLI interface is considered **stable** once implemented.

Future versions MAY add new commands or options but MUST preserve existing behavior for v1-compatible usage.

---

### 8. Errors (Schema Phase)

Schema validation errors MUST report:
- the path to the failing data element
- the expected type or requirement
- a clear explanation of the failure

Schema validation errors MUST be reported before any template processing occurs.

---

### 9. Specification Scope
This document defines v1 behavior only.

Future versions may extend the schema language or template system, but:

core guarantees MUST be preserved

v1-valid packages MUST remain supported



