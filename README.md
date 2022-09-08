## Bevy parallax mapping

[parallax mapping] and [parallax occlusion mapping] are graphical effects
adding the impression of depths to simple 2d textures by moving the texture's
pixel around according to the viewport's perspective.

This is **NOT** a "parallax" à la Super Mario World, this is intended for 3D
rendering. This technic has been used with success in the Demon's Souls
Bluepoint remake[citation needed].

This crate adds a custom material that extends the default bevy PBR material
with [parallax mapping]. The [`ParallaxMaterial`] asset is a straight copy
of bevy's PBR material with the addition of the `height_map: Handle<Image>`
filed (it's not `Option` since you might as well use the default shader if
there is no height maps).

`height_map` is a greyscale image representing the height of the object at the
specific pixel.

This crate's implementation uses Relief Parallax mapping with self occlusion.

### TODO

- [ ] Implement the shader
  - [X] default bevy PBR reproduction
  - [X] basic parallax mapping
  - [X] offset limiting
  - [ ] steep parallax mapping
  - [ ] relief parallax mapping
  - [ ] parallax occlusion mapping
  - [ ] Self shadowing
- [ ] Useability
  - [ ] Generic over shader (should be possible to use with a
        traditional phong shader)
  - [ ] Conversion methods `from_standard(StandardMaterial, height_map)`
  - [ ] Automatic `height_map` computation based on a `normal_map` if possible
    - https://old.reddit.com/r/gamedev/comments/fffskm/convert_normal_map_to_displacement_map/
    - https://forums.unrealengine.com/t/invert-normal-to-height/145496
    - https://houdinigubbins.wordpress.com/2019/08/09/from-normal-to-height/
    - https://stannum.io/blog/0IwyJ-
    - Search keywords are a bit silly, since search engines use a "bag of word" models and will
      just spit out the ten thousand "height map to normal map" tutorial, while leaving out what
      I actually mean, I found that "invert displacement map from normal map" gave satisfactory
      results on ddg. (also "displacement map" gives much better quality results than "height map")

[parallax mapping]: https://en.wikipedia.org/wiki/Parallax_mapping
[parallax occlusion mapping]: https://en.wikipedia.org/wiki/Parallax_occlusion_mapping
[`ParallaxMaterial`]: https://docs.rs/bevy_mod_paramap/0.1.0/bevy_mod_paramap/struct.ParallaxMaterial.html
