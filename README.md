
# pixelquix
It's like pixelfix, but quix.

- Written in Rust for low-level speed
- Uses `rayon` for work-stealing parallelism
- Based on Jump Flood for impressive speed even up to 8K
- Configurable alpha threshold to deal with blurry edges
- Configurable edge mode for dealing with clamped and repeating textures
- Outputs a variety of formats, including distance fields, opaque colour, and UV
- Licensed under MIT, for your usage pleasure

![Comparing the outputs of pixelquix on a high resolution image.](preview.png)
