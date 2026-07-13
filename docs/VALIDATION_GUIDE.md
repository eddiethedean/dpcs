# Validation Guide

Validation is deterministic and phase-based. It returns a `ValidationReport`
and does not panic on invalid contracts.

## Phases (current)

1. Document — unsupported `dpcsVersion` warnings (`DPCS-DOC-002`)
2. Canonical Object Model — identity, uniqueness, interface completeness, reserved extension keys
3. Structural — remaining shape (for example empty step `type`)
4. Graph — edges, cycles
5. References — resolvable contract references
6. Data Flow — endpoint validity against declared ports
7. Control Flow — step dependency endpoints
8. Quality / Failure — identity owned by COM; placement deferred
9. Extensions — reserved for future namespace rules (reserved root collisions are COM-012)

## Important validations today

- required root identity (`dpcsVersion`, `id`, `version`)
- unique identifiers within each addressable object kind
- unique interface port ids across inputs and outputs
- complete interface ports (`name`, `contractRef`, `purpose`) when ports are present
- valid graph edges and no prohibited cycles
- resolvable contract references
- valid data-flow endpoints (exact declared step ports; implicit ports when undeclared)
- valid control-flow step dependencies

Invalid contracts produce diagnostics instead of hard failures where possible.
