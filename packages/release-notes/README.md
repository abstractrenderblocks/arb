packages/release-notes/README.md
# release-notes

release-notes is an arb example package that turns a single validated YAML release definition
into multiple release artifacts.

## Package

packages/release-notes


## Input

- `examples/release.yaml`

## Output

This package generates:

- `CHANGELOG.md`
- `release.md` (GitHub Release body)
- `release.txt` (plain text announcement)

## Run

```cmd
cd /d E:\arb
E:\arb\target\debug\arb.exe compile --package packages\release-notes --data packages\release-notes\examples\release.yaml --out out-release
```

##Notes

arb v1 templates are intentionally simple and deterministic. This package keeps
“logic” out of templates by requiring structured data up front (sections, lists,
and optional blocks like user impact, known issues, and media).