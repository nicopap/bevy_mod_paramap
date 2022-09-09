## Bevy parallax mapping

[parallax mapping] is a graphical effect adding the impression of depth to
simple 2d textures by moving the texture's pixel around according to the
viewport's perspective.

This is **NOT** a "parallax" à la Super Mario World. This is intended for 3D
rendering. This technic has been used with success in the Demon's Souls
Bluepoint remake.

This crate adds a custom material that extends the default bevy PBR material
with [parallax mapping]. The [`ParallaxMaterial`] asset is a straight copy
of bevy's PBR material with the addition of the `height_map: Handle<Image>`
filed (it's not `Option` since you might as well use the default shader if
there is no height maps).

`height_map` is a greyscale image representing the height of the object at the
specific pixel.

[`ParallaxMaterial`] allows selecting the algorithm used for parallaxing. By
setting the [`algorithm`] field to the [`ParallaxAlgo`] of your choosing, you
may opt into using Relief Mapping. By default, [`ParallaxMaterial`] uses the
Parallax Occlusion Mapping (POM) method. (see the shader source code for
explanation on what the algorithms do)

### Literature

The code is basically copied from the [sunblackcat] implementation linked
on Wikipedia.

Optimization leads include:

1. <https://www.diva-portal.org/smash/get/diva2:831762/FULLTEXT01.pdf>
2. <https://www.gamedevs.org/uploads/quadtree-displacement-mapping-with-height-blending.pdf>
3. <https://developer.nvidia.com/gpugems/gpugems3/part-i-geometry/chapter-4-next-generation-speedtree-rendering>
4. <https://old.reddit.com/r/GraphicsProgramming/comments/pgkogk/whatever_happened_to_quadtree_displacement_mapping/>

Note that (1) says that (2) is slower than POM, while (3) is beyond out-of-scope
for a small opensource crate (unless you want to pay me).

### TODO

- [ ] Useability
  - [X] bevy-inspector-egui definition (behind compile flag)
  - [ ] Generic over shader (should be possible to use with a
        traditional phong shader)
  - [ ] Conversion methods `from_standard(StandardMaterial, height_map)`
  - [ ] Automatic `height_map` computation based on a `normal_map` if possible
    - <https://old.reddit.com/r/gamedev/comments/fffskm/convert_normal_map_to_displacement_map/>
    - <https://forums.unrealengine.com/t/invert-normal-to-height/145496>
    - <https://houdinigubbins.wordpress.com/2019/08/09/from-normal-to-height/>
    - <https://stannum.io/blog/0IwyJ->
    - Search keywords are a bit silly, since search engines use a "bag of word" models and will
      just spit out the ten thousand "height map to normal map" tutorial, while leaving out what
      I actually mean, I found that "invert displacement map from normal map" gave satisfactory
      results on ddg. (also "displacement map" gives much better quality results than "height map")

## License

Copyright © 2022 Nicola Papale

This software is licensed under Apache 2.0.


[parallax mapping]: https://en.wikipedia.org/wiki/Parallax_mapping
[parallax occlusion mapping]: https://en.wikipedia.org/wiki/Parallax_occlusion_mapping
[`ParallaxMaterial`]: https://docs.rs/bevy_mod_paramap/0.1.0/bevy_mod_paramap/struct.ParallaxMaterial.html
[sunblackcat]: https://web.archive.org/web/20150419215321/http://sunandblackcat.com/tipFullView.php?l=eng&topicid=28
[`algorithm`]: https://docs.rs/bevy_mod_paramap/0.1.0/bevy_mod_paramap/struct.ParallaxMaterial.html#algorithm
[`ParallaxAlgo`]: https://docs.rs/bevy_mod_paramap/0.1.0/bevy_mod_paramap/enum.ParallaxAlgo.html
