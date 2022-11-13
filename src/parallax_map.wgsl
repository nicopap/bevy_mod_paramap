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
    flags: u32,
    alpha_cutoff: f32,
    height_depth: f32,
    max_height_layers: f32,
};

struct FragmentInput {
    @builtin(front_facing) is_front: bool,
    @builtin(position) frag_coord: vec4<f32>,
    #import bevy_pbr::mesh_vertex_output
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


// NOTE: This ensures that the world_normal is normalized and if
// vertex tangents and normal maps then normal mapping may be applied.
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

fn sample_height(uv: vec2<f32>) -> f32 {
    return textureSample(height_map_texture, height_map_sampler, uv).r;
}

// An implementation of parallax mapping, see https://en.wikipedia.org/wiki/Parallax_mapping
// Code derived from: https://web.archive.org/web/20150419215321/http://sunandblackcat.com/tipFullView.php?l=eng&topicid=28
fn parallaxed_uv(
    depth: f32,
    max_layer_count: f32,
    // The original uv
    uv: vec2<f32>,
    // The vector from camera to the surface of material
    V: vec3<f32>,
) -> vec3<f32> {

    // Steep parallax mapping
    // ======================
    // split the height map into `layer_count` layers,
    // When V hits the surface of object (excluding displacement),
    // if not bellow or on surface including displacement (textureSample), then
    // look forward (-= delta_uv) according to V and distance between hit surface and
    // height map surface, repeat until bellow surface.
    //
    // where `layer_count` is selected smartly between `min_layer_count` and
    // `max_layer_count` according to the steepness of V.
    let MIN_LAYER_COUNT = 2.0;
    let MAX_ITER = 1000;

    let view_steepness = abs(dot(vec3<f32>(0.0, 0.0, 1.0), V));
    let layer_count = mix(max_layer_count, MIN_LAYER_COUNT, view_steepness);
    let layer_height = 1.0 / layer_count;
    let delta_uv = depth * V.xy / V.z / layer_count;
    var uv = uv;

    var current_layer_height = 0.0;
    var current_height = sample_height(uv);
    // In the original, this is a `while`, but
    // this is a failsafe to avoid locking the dev's computer when they accidentally
    // cause `current_height <= current_layer_height` to never happen.
    for (var i: i32 = 0; i < MAX_ITER; i++)  {
        if (current_height <= current_layer_height) {
            break;
        }
        current_layer_height += layer_height;
        uv -= delta_uv;
        current_height = sample_height(uv);
    }
    
#ifdef RELIEF_MAPPING
    // Relief mapping
    // ==============
    // "refine" the rough result from the steep parallax mapping
    // with a binary search between the layer selected by steep parallax
    // and next one of point closest to height map surface.
    // This eliminates the jaggy step artifacts from steep parallax
    let MAX_STEPS: i32 = 5;

    var delta_uv = delta_uv / 2.0;
    var delta_height = layer_height / 2.0;
    uv += delta_uv;
    current_layer_height -= delta_height;
    for (var i: i32 = 0; i < MAX_STEPS; i++) {
        delta_uv = delta_uv / 2.0;
        delta_height /= 2.0;
        current_height = sample_height(uv);

        if (current_height > current_layer_height) {
            uv -= delta_uv;
            current_layer_height += delta_height;
        } else {
            uv += delta_uv;
            current_layer_height -= delta_height;
        }
    }
#else    
    // Parallax Occlusion mapping
    // ==========================
    // "refine" steep mapping simply by interpolating between the
    // previous layer's height and the computed layer height.
    // Only requires a single lookup, unlike relief mapping, but
    // may incure artifacts on very steep relief.

    // TODO: there is probably a way to use the sampler instead
    // of interpolating by hand here.
    let previous_uv = uv + delta_uv;
    let next_height = current_height - current_layer_height;
    let previous_height = sample_height(previous_uv) - current_layer_height + layer_height;

    let weight = next_height / (next_height - previous_height);

    let uv = mix(uv, previous_uv, weight);

    let current_layer_height = current_layer_height
        + mix(next_height, previous_height, weight);
#endif

    return vec3<f32>(uv, current_layer_height);
}


@fragment
fn fragment(in: FragmentInput) -> @location(0) vec4<f32> {
    let is_orthographic = view.projection[3].w == 1.0;
    let V = calculate_view(in.world_position, is_orthographic);
    let tangent_V = vec3<f32>(
        dot(V, in.world_tangent.xyz),
        dot(V, -cross(in.world_normal, in.world_tangent.xyz) * sign(in.world_tangent.w)),
        dot(V, in.world_normal),
    );
    let tangent_V = normalize(tangent_V);
    let uv =  parallaxed_uv(material.height_depth, material.max_height_layers, in.uv, tangent_V);
    let height_depth = uv.z;
    let uv = uv.xy;
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
            let metallic_roughness = textureSample(metallic_roughness_texture, metallic_roughness_sampler, uv);
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
            in.world_tangent,
            uv,
        );
        pbr_input.V = V;

        output_color = pbr(pbr_input);
        #ifdef TONEMAP_IN_SHADER
            output_color = tone_mapping(output_color);
        #endif
    }

    return output_color;
}
