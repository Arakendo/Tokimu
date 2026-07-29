# FBX Selection v1 Feature Matrix

This matrix records why a case is admitted. It does not claim that Tokimu
currently implements or passes the capability.

| Capability pressure | Cases | Current status |
| --- | ---: | --- |
| ASCII source encoding | 10 | Modern source graph, static geometry, source transforms, material, Unicode, and one transformed-skin profile agree with binary; legacy name-based exports remain explicit record-level evidence; malformed inputs reject at syntax or skinning boundaries |
| Binary source encoding | 13 | Fixture available |
| Big-endian binary arrays | 1 | Source and static-geometry evidence decoded |
| FBX version pairs | 4 logical scenes | Fixture available |
| Static mesh and hierarchy | 5 cube encodings | Fixture available |
| UV mapping and reference modes | 2 | Binary source layer preserved; legacy ASCII UV arrays decode structurally while name-based Connect graph interpretation remains deferred |
| Shared geometry instances | 1 | Fixture available |
| Y-up and Z-up source axes | 2 | Distinct source axes and finite unit metadata preserved |
| Material interpretation and texture references | 2 | ASCII/binary source property, texture path, and binding counts agree; no Tokimu material semantics admitted |
| Unicode object names | 2 | Binary and ASCII source identity preserved through provider-local source evidence |
| Animation stack and curves | 2 | Modern binary source stacks, layers, curves, keys, and raw attributes preserved; legacy 5.8 ASCII animation records decode while its binary peer remains unsupported-source evidence |
| Skeleton and skin clusters | 2 | Binary source clusters and `LimbNode` parent links preserved; one paired ASCII/binary transformed-skin profile agrees on influence, hierarchy, and `Link_Mode` presence/value observations without evaluating deformation |
| Static blend shape | 1 | Binary source blend-shape channel and target evidence preserved |
| Animated blend-shape weight | 1 | Binary morph and animation evidence preserved separately |
| Truncated ASCII token | 1 | Expected invalid |
| Non-finite values | 1 | Expected invalid |
| Broken skin cluster | 1 | Valid ASCII source and static geometry; expected skinning-stage rejection for a cluster with no joint-model connection |

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
