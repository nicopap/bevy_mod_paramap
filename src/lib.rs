/*!
[`ParallaxMaterial`]: ParallaxMaterial
*/
#![doc = include_str!("../README.md")]
#![deny(missing_docs)]
#![warn(clippy::pedantic, clippy::nursery)]

#[cfg(feature = "inspector-def")]
mod inspector_def;

use bevy::{
    asset::load_internal_asset,
    pbr::{MaterialPipeline, MaterialPipelineKey, StandardMaterialUniform},
    prelude::*,
    reflect::TypeUuid,
    render::{
        mesh::MeshVertexBufferLayout,
        render_asset::RenderAssets,
        render_resource::{
            AsBindGroup, AsBindGroupShaderType, Face, RenderPipelineDescriptor, ShaderRef,
            ShaderType, SpecializedMeshPipelineError,
        },
    },
};

/// The shader handle for `"parallax_map.wgsl"`.
#[allow(clippy::unreadable_literal)]
const PARALLAX_MAPPING_SHADER_HANDLE: HandleUntyped =
    HandleUntyped::weak_from_u64(Shader::TYPE_UUID, 9592100656503623734);

impl From<&'_ ParallaxMaterial> for StandardMaterial {
    fn from(mat: &'_ ParallaxMaterial) -> Self {
        let opt_clone_weak = |opt: &Option<_>| opt.as_ref().map(Handle::clone_weak);
        Self {
            base_color: mat.base_color,
            base_color_texture: opt_clone_weak(&mat.base_color_texture),
            emissive: mat.emissive,
            emissive_texture: opt_clone_weak(&mat.emissive_texture),
            perceptual_roughness: mat.perceptual_roughness,
            metallic: mat.metallic,
            metallic_roughness_texture: opt_clone_weak(&mat.metallic_roughness_texture),
            reflectance: mat.reflectance,
            normal_map_texture: Some(mat.normal_map_texture.clone_weak()),
            flip_normal_map_y: mat.flip_normal_map_y,
            occlusion_texture: opt_clone_weak(&mat.occlusion_texture),
            double_sided: mat.double_sided,
            cull_mode: mat.cull_mode,
            unlit: mat.unlit,
            alpha_mode: mat.alpha_mode,
            depth_bias: mat.depth_bias,
        }
    }
}

/// The pipeline key for [`ParallaxMaterial`], this just copies the
/// [`StandardMaterialKey`] bevy impl.
///
/// [`StandardMaterialKey`]: bevy::pbr::StandardMaterialKey
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct ParallaxMaterialKey {
    relief_mapping: bool,
    cull_mode: Option<Face>,
}
impl From<&'_ ParallaxMaterial> for ParallaxMaterialKey {
    fn from(material: &ParallaxMaterial) -> Self {
        Self {
            relief_mapping: material.algorithm == ParallaxAlgo::ReliefMapping,
            cull_mode: material.cull_mode,
        }
    }
}

/// The GPU representation of the uniform data of a [`ParallaxMaterial`].
#[derive(Clone, Default, ShaderType)]
pub struct ParallaxMaterialUniform {
    /// Doubles as diffuse albedo for non-metallic, specular for metallic and a mix for everything
    /// in between.
    pub base_color: Vec4,
    /// Use a color for user friendliness even though we technically don't use the alpha channel
    /// Might be used in the future for exposure correction in HDR
    pub emissive: Vec4,
    /// Linear perceptual roughness, clamped to [0.089, 1.0] in the shader
    /// Defaults to minimum of 0.089
    pub roughness: f32,
    /// From [0.0, 1.0], dielectric to pure metallic
    pub metallic: f32,
    /// Specular intensity for non-metals on a linear scale of [0.0, 1.0]
    /// defaults to 0.5 which is mapped to 4% reflectance in the shader
    pub reflectance: f32,
    /// The shader flags.
    pub flags: u32,
    /// When the alpha mode mask flag is set, any base color alpha above this cutoff means fully opaque,
    /// and any below means fully transparent.
    pub alpha_cutoff: f32,
    /// The depth of the height map.
    pub height_depth: f32,
    /// In how many layers to split the height maps for Steep parallax mapping.
    ///
    /// If your `height_depth` is >0.1 and you are seeing jaggy edges,
    /// increase this value. However, this incures a performance cost.
    pub max_height_layers: f32,
}

impl AsBindGroupShaderType<ParallaxMaterialUniform> for ParallaxMaterial {
    fn as_bind_group_shader_type(&self, images: &RenderAssets<Image>) -> ParallaxMaterialUniform {
        let standard_material: StandardMaterial = self.into();
        let standard_uniform: StandardMaterialUniform =
            standard_material.as_bind_group_shader_type(images);
        ParallaxMaterialUniform {
            base_color: standard_uniform.base_color,
            emissive: standard_uniform.emissive,
            roughness: standard_uniform.roughness,
            metallic: standard_uniform.metallic,
            reflectance: standard_uniform.reflectance,
            flags: standard_uniform.flags,
            alpha_cutoff: standard_uniform.alpha_cutoff,
            height_depth: self.height_depth,
            max_height_layers: self.max_height_layers,
        }
    }
}

/// A shameless clone of bevy's [default PBR material] with an additional field:
/// `height_map`.
///
/// `height_map` is a greyscale image representing the height of the object at a given
/// pixel. Works like the original [`StandardMaterial`] otherwise.
///
/// **WARNING**: this material _assumes_ the mesh has tangents set. If your mesh doesn't
/// have tangents, bad unspecified things will happen.
///
/// [default PBR material]: StandardMaterial
#[derive(AsBindGroup, Debug, Clone, TypeUuid)]
#[uuid = "5bc9c7a3-fb25-4202-b91f-bc4c7d300d82"]
#[bind_group_data(ParallaxMaterialKey)]
#[uniform(0, ParallaxMaterialUniform)]
pub struct ParallaxMaterial {
    /// Doubles as diffuse albedo for non-metallic, specular for metallic and a mix for everything
    /// in between. If used together with a base_color_texture, this is factored into the final
    /// base color as `base_color * base_color_texture_value`
    pub base_color: Color,

    /// The "albedo" of the material, when `Some`, this will be the texture applied to the mesh.
    #[texture(1)]
    #[sampler(2)]
    pub base_color_texture: Option<Handle<Image>>,

    // Use a color for user friendliness even though we technically don't use the alpha channel
    // Might be used in the future for exposure correction in HDR
    /// Color the material "emits" to the camera.
    ///
    /// This is typically used for monitor screens or LED lights.
    /// Anything that can be visible even in darkness.
    ///
    /// The emissive color is added to what would otherwise be the material's visible color.
    /// This means that for a light emissive value, in darkness,
    /// you will mostly see the emissive component.
    ///
    /// The default emissive color is black, which doesn't add anything to the material color.
    ///
    /// Note that **an emissive material won't light up surrounding areas like a light source**,
    /// it just adds a value to the color seen on screen.
    pub emissive: Color,

    /// Same as emissive, but based off a texture
    #[texture(3)]
    #[sampler(4)]
    pub emissive_texture: Option<Handle<Image>>,

    /// Linear perceptual roughness, clamped to [0.089, 1.0] in the shader
    /// Defaults to minimum of 0.089
    /// If used together with a roughness/metallic texture, this is factored into the final base
    /// color as `roughness * roughness_texture_value`
    pub perceptual_roughness: f32,

    /// From [0.0, 1.0], dielectric to pure metallic
    /// If used together with a roughness/metallic texture, this is factored into the final base
    /// color as `metallic * metallic_texture_value`
    pub metallic: f32,

    /// A texture representing both `metallic` and `preceptual_roughness`.
    ///
    /// The blue channel is the `metallic` and green is `roughness` (we don't
    /// talk about the red channel)
    #[texture(5)]
    #[sampler(6)]
    pub metallic_roughness_texture: Option<Handle<Image>>,

    /// Specular intensity for non-metals on a linear scale of [0.0, 1.0]
    /// defaults to 0.5 which is mapped to 4% reflectance in the shader
    pub reflectance: f32,

    /// Used to fake the lighting of bumps and dents on a material.
    ///
    /// A typical usage would be faking cobblestones on a flat plane mesh in 3D.
    ///
    /// # Notes
    ///
    /// Normal mapping with `StandardMaterial` and the core bevy PBR shaders requires:
    /// - A normal map texture
    /// - Vertex UVs
    /// - Vertex tangents
    /// - Vertex normals
    ///
    /// Tangents do not have to be stored in your model,
    /// they can be generated using the [`Mesh::generate_tangents`] method.
    /// If your material has a normal map, but still renders as a flat surface,
    /// make sure your meshes have their tangents set.
    ///
    /// [`Mesh::generate_tangents`]: bevy::render::mesh::Mesh::generate_tangents
    #[texture(9)]
    #[sampler(10)]
    pub normal_map_texture: Handle<Image>,

    /// Normal map textures authored for DirectX have their y-component flipped. Set this to flip
    /// it to right-handed conventions.
    pub flip_normal_map_y: bool,

    /// Specifies the level of exposure to ambient light.
    ///
    /// This is usually generated and stored automatically ("baked") by 3D-modelling software.
    ///
    /// Typically, steep concave parts of a model (such as the armpit of a shirt) are darker,
    /// because they have little exposed to light.
    /// An occlusion map specifies those parts of the model that light doesn't reach well.
    ///
    /// The material will be less lit in places where this texture is dark.
    /// This is similar to ambient occlusion, but built into the model.
    #[texture(7)]
    #[sampler(8)]
    pub occlusion_texture: Option<Handle<Image>>,

    /// Support two-sided lighting by automatically flipping the normals for "back" faces
    /// within the PBR lighting shader.
    /// Defaults to false.
    /// This does not automatically configure backface culling, which can be done via
    /// `cull_mode`.
    pub double_sided: bool,

    /// Whether to cull the "front", "back" or neither side of a mesh
    /// defaults to `Face::Back`
    pub cull_mode: Option<Face>,

    /// Whether to shade this material.
    ///
    /// Normals, occlusion textures, roughness, metallic, reflectance and
    /// emissive are ignored if this is set to `true`.
    pub unlit: bool,

    /// How to interpret the alpha channel of the `base_color_texture`.
    ///
    /// By default, it's `Opaque`, therefore completely ignored.
    /// Note that currently bevy handles poorly semi-transparent textures. You
    /// are likely to encounter the following bugs:
    ///
    /// - When two `AlphaMode::Blend` material occupy the same pixel, only one
    ///   material's color will show.
    /// - If a different mesh is both "in front" and "behind" a non-opaque material,
    ///   bevy won't know which material to display in front, which might result in
    ///   flickering.
    pub alpha_mode: AlphaMode,

    /// Re-arange depth of material, useful to avoid z-fighting.
    pub depth_bias: f32,

    /// The height map used for parallax mapping.
    ///
    /// Black is the tallest, white deepest.
    ///
    /// To improve performance, set your `height_map`'s [`Image::sampler_descriptor`]
    /// filter mode to `FilterMode::Nearest`, as [this paper] indicates, it improves
    /// perfs a bit.
    ///
    /// [this paper]: https://www.diva-portal.org/smash/get/diva2:831762/FULLTEXT01.pdf
    #[texture(11)]
    #[sampler(12)]
    pub height_map: Handle<Image>,

    /// How deep the offset introduced by the height map should be.
    ///
    /// Default is 0.1, anything over that value may look very awkward.
    /// Lower value look less "deep."
    pub height_depth: f32,

    /// Whether to use a more accurate and more expensive algorithm.
    ///
    /// We recommend that all objects use the same [`ParallaxAlgo`], to avoid
    /// duplicating and running two shaders.
    pub algorithm: ParallaxAlgo,

    /// In how many layers to split the height maps for Steep Parallax Mapping.
    ///
    /// If your `height_depth` is `>0.1` and you are seeing jaggy edges,
    /// increase this value. However, this incures a performance cost.
    ///
    /// Default is 16.0.
    ///
    /// **This must never be less than `2.0`.**
    pub max_height_layers: f32,
}

/// The algorithm to use beyond the initial Steep parallax mapping
/// to compute the displacement of a pixel.
///
/// See the shader code for implementation details and explanation
/// of the methods used.
#[cfg_attr(feature = "inspector-def", derive(bevy_inspector_egui::Inspectable))]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Default)]
pub enum ParallaxAlgo {
    /// A simple linear interpolation, consists of a single texture sample.
    #[default]
    ParallaxOcclusionMapping,
    /// An iterative discovery of up to 5 iteration of the best displacement
    /// value. Each iteration incures a texture sample.
    ReliefMapping,
}
impl Default for ParallaxMaterial {
    fn default() -> Self {
        Self {
            base_color: Color::rgb(1.0, 1.0, 1.0),
            base_color_texture: None,
            emissive: Color::BLACK,
            emissive_texture: None,
            perceptual_roughness: 0.089,
            metallic: 0.01,
            metallic_roughness_texture: None,
            reflectance: 0.5,
            occlusion_texture: None,
            normal_map_texture: default(),
            flip_normal_map_y: false,
            double_sided: false,
            cull_mode: Some(Face::Back),
            unlit: false,
            alpha_mode: AlphaMode::Opaque,
            depth_bias: 0.0,
            height_map: default(),
            height_depth: 0.1,
            max_height_layers: 16.0,
            algorithm: default(),
        }
    }
}
impl Material for ParallaxMaterial {
    fn specialize(
        _pipeline: &MaterialPipeline<Self>,
        descriptor: &mut RenderPipelineDescriptor,
        _layout: &MeshVertexBufferLayout,
        key: MaterialPipelineKey<Self>,
    ) -> Result<(), SpecializedMeshPipelineError> {
        let defs = &mut descriptor.fragment.as_mut().unwrap().shader_defs;
        if key.bind_group_data.relief_mapping {
            defs.push(String::from("RELIEF_MAPPING"));
        }
        descriptor.primitive.cull_mode = key.bind_group_data.cull_mode;
        if let Some(label) = &mut descriptor.label {
            *label = format!("parallax_{}", *label).into();
        }
        Ok(())
    }

    fn fragment_shader() -> ShaderRef {
        PARALLAX_MAPPING_SHADER_HANDLE.typed::<Shader>().into()
    }

    #[inline]
    fn alpha_mode(&self) -> AlphaMode {
        self.alpha_mode
    }

    #[inline]
    fn depth_bias(&self) -> f32 {
        self.depth_bias
    }
}

/// Add this plugin to your app to use [`ParallaxMaterial`].
pub struct ParallaxMaterialPlugin;
impl Plugin for ParallaxMaterialPlugin {
    fn build(&self, app: &mut App) {
        load_internal_asset!(
            app,
            PARALLAX_MAPPING_SHADER_HANDLE,
            "parallax_map.wgsl",
            Shader::from_wgsl
        );
        app.add_plugin(MaterialPlugin::<ParallaxMaterial>::default());
        #[cfg(feature = "inspector-def")]
        {
            use bevy_inspector_egui::RegisterInspectable;
            app.register_inspectable::<ParallaxMaterial>()
                .register_inspectable::<Handle<ParallaxMaterial>>();
        }
    }
}
