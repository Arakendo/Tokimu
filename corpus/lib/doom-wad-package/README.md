# Doom WAD Package Bridge

This corpus-only bridge composes existing bounded archive/Resource Space
mechanisms with `doom-wad-provider`. It reads one selected WAD member through a
read-only archive-derived view and passes transient member bytes to the WAD
container provider.

It does not materialize the WAD as a new Resource Space resource, teach
Resource Space about Doom, or make the WAD provider parse ZIP containers.
