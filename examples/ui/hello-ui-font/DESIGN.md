# Hello UI Font

This corpus test proves the first real font path:

```text
prepared TTF and OTF
-> ab_glyph rasterization
-> two RGBA8 texture uploads
-> two Texture2d materials
-> side-by-side GPU quads
```

It intentionally renders one glyph from each format before attempting font
fallback, shaping, or atlas batching. The selected files are the first prepared
Inter TTF and Noto OTF found in their respective provider directories.
