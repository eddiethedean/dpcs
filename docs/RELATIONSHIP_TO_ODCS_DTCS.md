# Relationship to ODCS and DTCS

DPCS is the composition layer.

```text
ODCS
  defines what data is

DTCS
  defines how data changes

DPCS
  defines how transformations compose into pipelines
```

DPCS should reference ODCS and DTCS artifacts by contract reference.

Do not duplicate ODCS dataset semantics.

Do not duplicate DTCS transformation semantics.
