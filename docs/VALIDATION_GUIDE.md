# Validation Guide

Validation should be deterministic and phase-based.

Validation should return a `ValidationReport`, not panic.

Important validations:

- required root fields
- unique pipeline step identifiers
- valid graph edges
- no prohibited cycles
- resolvable contract references
- valid data flow endpoints
- valid control flow dependencies
- valid quality gate placement
- valid failure semantic scopes
- extension namespace validity

Invalid contracts should produce diagnostics instead of hard failures where possible.
