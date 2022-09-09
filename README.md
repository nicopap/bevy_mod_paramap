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

### Examples

This repo contains two examples. to modify [`ParallaxMaterial`] values
dynamically at run time, add the `inspector-def` feature flag.

```bash
cargo run --example <example_name> --features "inspector-def"
```

- [`earth3d`]: a spinning view of the earth. Takes advantage of height map,
  but also of all the bevy PBR fields. This a good demonstration of bevy's
  capabilities.
  \
  You can orbit the earth by holding down the right mouse button, and zoom
  in/out with the mouse wheel.

https://user-images.githubusercontent.com/26321040/189361740-1a0876d2-9b39-49f3-a8cb-8837601b5b39.mp4

- [`cube`]: A spinning cube with a parallaxed material in a basic 3d scene,
  mouse left click to switch point of view.

https://user-images.githubusercontent.com/26321040/189361802-3db6aa98-fa7f-4440-b5a7-20d73a36ac23.mp4
  
### Bugs and limitations

- This doesn't implement silhouetting, so the meshes's silhouette will not
  change with the height map
  - As a result, height maps that are top-heavy should be favored (tree bark, bricks)
- This doesn't implement self-shadowing, resulting in potentially surprising sharp cutoffs
  - self-shadowing is described in the [sunblackcat] article, but bevy's lighting system requires
    handling several light types and multiple light sources at once, which is more complex than
    I can handle right now.
- The height map is inverted from the more common usage
  
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
    - Search keywords are a bit silly, I found that "invert displacement map from normal map" gave
      satisfactory results on ddg.

## License

Earth images in `assets/earth` are public domain and taken from Wikimedia. I edited them myself, you
are free to re-use the edited version however you want without restrictions.

- [height map]: `elevation_water.png` and `elevation_surface.png` adjust the values to highlight
  different topological features of earth, the `normal_map.jpg` is also derived from it.
- [albedo] (aka base color) is a scalled-down version of the 2002 Nasa blue marble earth satellite
  view using a equirectangular projection. `metallic_roughness.png` and `base_color.jpg` are derived
  from that image.
- [emissive texture] is from the 2012 Nasa blue marble project. It's a night time satellite view of
  earth.

Copyright of code and assets go to their respective authors.

Original code is copyright © 2022 Nicola Papale

This software is licensed under Apache 2.0.


[parallax mapping]: https://en.wikipedia.org/wiki/Parallax_mapping
[parallax occlusion mapping]: https://en.wikipedia.org/wiki/Parallax_occlusion_mapping
[sunblackcat]: https://web.archive.org/web/20150419215321/http://sunandblackcat.com/tipFullView.php?l=eng&topicid=28
[height map]: https://commons.wikimedia.org/wiki/File:Earth_dry_elevation.png
[albedo]: https://commons.wikimedia.org/wiki/File:Blue_Marble_2002_bg21600.png
[emissive texture]: https://commons.wikimedia.org/wiki/File:Composite_map_of_the_world_2012.jpg
[`ParallaxMaterial`]: https://docs.rs/bevy_mod_paramap/0.1.0/bevy_mod_paramap/struct.ParallaxMaterial.html
[`algorithm`]: https://docs.rs/bevy_mod_paramap/0.1.0/bevy_mod_paramap/struct.ParallaxMaterial.html#algorithm
[`ParallaxAlgo`]: https://docs.rs/bevy_mod_paramap/0.1.0/bevy_mod_paramap/enum.ParallaxAlgo.html
[`cube`]: https://github.com/nicopap/bevy_mod_paramap/blob/main/examples/cube.rs
[`earth3d`]: https://github.com/nicopap/bevy_mod_paramap/blob/main/examples/earth3d.rs
