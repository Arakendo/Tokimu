# FBX Selection v1 Feature Matrix

This matrix records why a case is admitted. It does not claim that Tokimu
currently implements or passes the capability.

| Capability pressure | Cases | Current status |
| --- | ---: | --- |
| ASCII source encoding | 10 | Fixture available |
| Binary source encoding | 13 | Fixture available |
| Big-endian binary arrays | 1 | Fixture available |
| FBX version pairs | 4 logical scenes | Fixture available |
| Static mesh and hierarchy | 5 cube encodings | Fixture available |
| UV mapping and reference modes | 2 | Fixture available |
| Shared geometry instances | 1 | Fixture available |
| Y-up and Z-up source axes | 2 | Fixture available |
| Material interpretation | 2 | Fixture available |
| Unicode object names | 2 | Fixture available |
| Animation stack and curves | 2 | Fixture available |
| Skeleton and skin clusters | 2 | Fixture available |
| Static blend shape | 1 | Fixture available |
| Animated blend-shape weight | 1 | Fixture available |
| Truncated ASCII token | 1 | Expected invalid |
| Non-finite values | 1 | Expected invalid |
| Broken skin cluster | 1 | Expected invalid |

## Deferred

- embedded textures and external media resolution;
- layered materials and legacy shading models;
- animation layers and constraints;
- NURBS, subdivision surfaces, and geometry caches;
- the generated axis matrix;
- the complete fuzz corpus;
- the separate 4.7 GB public dataset;
- optional differential comparison against `ufbx`;
- native importer, canonical model, render, or visual claims.
