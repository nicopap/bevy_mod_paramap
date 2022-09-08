#import bevy_pbr::mesh_view_bindings
#import bevy_pbr::pbr_types
#import bevy_pbr::mesh_bindings

#import bevy_pbr::utils
#import bevy_pbr::clustered_forward
#import bevy_pbr::lighting
#import bevy_pbr::shadows
#import bevy_pbr::pbr_functions

struct ParallaxMaterial {
    base_color: vec4<f32>,
    emissive: vec4<f32>,
    perceptual_roughness: f32,
    metallic: f32,
    reflectance: f32,
    // 'flags' is a bit field indicating various options. u32 is 32 bits so we have up to 32 options.
    flags: u32,
    alpha_cutoff: f32,
    height_depth: f32,
};


@group(1) @binding(0)
var<uniform> material: ParallaxMaterial;
@group(1) @binding(1)
var base_color_texture: texture_2d<f32>;
@group(1) @binding(2)
var base_color_sampler: sampler;
@group(1) @binding(3)
var emissive_texture: texture_2d<f32>;
@group(1) @binding(4)
var emissive_sampler: sampler;
@group(1) @binding(5)
var metallic_roughness_texture: texture_2d<f32>;
@group(1) @binding(6)
var metallic_roughness_sampler: sampler;
@group(1) @binding(7)
var occlusion_texture: texture_2d<f32>;
@group(1) @binding(8)
var occlusion_sampler: sampler;
@group(1) @binding(9)
var normal_map_texture: texture_2d<f32>;
@group(1) @binding(10)
var normal_map_sampler: sampler;
@group(1) @binding(11)
var height_map_texture: texture_2d<f32>;
@group(1) @binding(12)
var height_map_sampler: sampler;

struct FragmentInput {
    @builtin(front_facing) is_front: bool,
    @builtin(position) frag_coord: vec4<f32>,
    #import bevy_pbr::mesh_vertex_output
};


// NOTE: This ensures that the world_normal is normalized and if
// vertex tangents and normal maps then normal mapping may be applied.
#ifdef VERTEX_TANGENTS
#ifdef PARALLAXMATERIAL_NORMAL_MAP
fn prepare_normal_parallax(
    standard_material_flags: u32,
    world_normal: vec3<f32>,
    is_front: bool,
    world_tangent: vec4<f32>,
    uv: vec2<f32>,
) -> vec3<f32> {
    var N: vec3<f32> = world_normal;
    var T: vec3<f32> = world_tangent.xyz;
    var B: vec3<f32> = world_tangent.w * cross(N, T);

    if ((standard_material_flags & STANDARD_MATERIAL_FLAGS_DOUBLE_SIDED_BIT) != 0u) {
        if (!is_front) {
            N = -N;
            T = -T;
            B = -B;
        }
    }
    var Nt = textureSample(normal_map_texture, normal_map_sampler, uv).rgb;
    if ((standard_material_flags & STANDARD_MATERIAL_FLAGS_TWO_COMPONENT_NORMAL_MAP) != 0u) {
        // Only use the xy components and derive z for 2-component normal maps.
        Nt = vec3<f32>(Nt.rg * 2.0 - 1.0, 0.0);
        Nt.z = sqrt(1.0 - Nt.x * Nt.x - Nt.y * Nt.y);
    } else {
        Nt = Nt * 2.0 - 1.0;
    }
    // Normal maps authored for DirectX require flipping the y component
    if ((standard_material_flags & STANDARD_MATERIAL_FLAGS_FLIP_NORMAL_MAP_Y) != 0u) {
        Nt.y = -Nt.y;
    }
    N = normalize(Nt.x * T + Nt.y * B + Nt.z * N);

    return N;
}
#endif
#else
fn prepare_normal(
    standard_material_flags: u32,
    world_normal: vec3<f32>,
    is_front: bool,
) -> vec3<f32> {
    var N: vec3<f32> = normalize(world_normal);
    if ((standard_material_flags & STANDARD_MATERIAL_FLAGS_DOUBLE_SIDED_BIT) != 0u) {
        if (!is_front) { N = -N; }
    }
    return N;
}
#endif

#ifdef VERTEX_TANGENTS
#ifndef PARALLAXMATERIAL_NORMAL_MAP
fn prepare_normal_parallax(
    standard_material_flags: u32,
    world_normal: vec3<f32>,
    is_front: bool,
) -> vec3<f32> {
    var N: vec3<f32> = normalize(world_normal);
    if ((standard_material_flags & STANDARD_MATERIAL_FLAGS_DOUBLE_SIDED_BIT) != 0u) {
        if (!is_front) { N = -N; }
    }
    return N;
}
#endif
#endif

fn parallaxed_texture(
    height_depth: f32,
    uv: vec2<f32>,
    V: vec3<f32>,
) -> vec2<f32> {
    let height_depth = 0.08;
    // The vector from camera to position
    let initial_height = textureSample(height_map_texture, height_map_sampler, uv).r;
    // original parallax mapping uses V.xy / V.z, removing V.z is offset limiting,
    // and reduces artifacts
    let texture_offset = -height_depth * initial_height * V.xy ;
    return uv + texture_offset;
}
@fragment
fn fragment(in: FragmentInput) -> @location(0) vec4<f32> {
    let is_orthographic = view.projection[3].w == 1.0;
    let V = calculate_view(in.world_position, is_orthographic);
    let uv =  parallaxed_texture(material.height_depth, in.uv, V);
    var output_color: vec4<f32> = material.base_color;
#ifdef VERTEX_COLORS
    output_color = output_color * in.color;
#endif
    if ((material.flags & STANDARD_MATERIAL_FLAGS_BASE_COLOR_TEXTURE_BIT) != 0u) {
        let texture_color = textureSample(base_color_texture, base_color_sampler, uv);
        output_color = output_color * texture_color;
    }

    // NOTE: Unlit bit not set means == 0 is true, so the true case is if lit
    if ((material.flags & STANDARD_MATERIAL_FLAGS_UNLIT_BIT) == 0u) {
        // Prepare a 'processed' StandardMaterial by sampling all textures to resolve
        // the material members
        var pbr_input: PbrInput;

        pbr_input.material.base_color = output_color;
        pbr_input.material.reflectance = material.reflectance;
        pbr_input.material.flags = material.flags;
        pbr_input.material.alpha_cutoff = material.alpha_cutoff;

        // TODO use .a for exposure compensation in HDR
        var emissive: vec4<f32> = material.emissive;
        if ((material.flags & STANDARD_MATERIAL_FLAGS_EMISSIVE_TEXTURE_BIT) != 0u) {
            emissive = vec4<f32>(emissive.rgb * textureSample(emissive_texture, emissive_sampler, uv).rgb, 1.0);
        }
        pbr_input.material.emissive = emissive;

        var metallic: f32 = material.metallic;
        var perceptual_roughness: f32 = material.perceptual_roughness;
        if ((material.flags & STANDARD_MATERIAL_FLAGS_METALLIC_ROUGHNESS_TEXTURE_BIT) != 0u) {
            let metallic_roughness = textureSample(metallic_roughness_texture, metallic_roughness_sampler, in.uv);
            // Sampling from GLTF standard channels for now
            metallic = metallic * metallic_roughness.b;
            perceptual_roughness = perceptual_roughness * metallic_roughness.g;
        }
        pbr_input.material.metallic = metallic;
        pbr_input.material.perceptual_roughness = perceptual_roughness;

        var occlusion: f32 = 1.0;
        if ((material.flags & STANDARD_MATERIAL_FLAGS_OCCLUSION_TEXTURE_BIT) != 0u) {
            occlusion = textureSample(occlusion_texture, occlusion_sampler, uv).r;
        }
        pbr_input.occlusion = occlusion;

        pbr_input.frag_coord = in.frag_coord;
        pbr_input.world_position = in.world_position;
        pbr_input.world_normal = in.world_normal;

        pbr_input.is_orthographic = is_orthographic;

        pbr_input.N = prepare_normal_parallax(
            material.flags,
            in.world_normal,
            in.is_front,
#ifdef VERTEX_TANGENTS
#ifdef PARALLAXMATERIAL_NORMAL_MAP
            in.world_tangent,
            uv,
#endif
#endif
        );
        pbr_input.V = V;

        output_color = tone_mapping(pbr(pbr_input));
    }

    return output_color;
}