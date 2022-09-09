// Copy/pooped from https://github.com/jakobhellermann/bevy-inspector-egui/blob/d6cca057ca477c43e586f5d462908fec2bfe9542/src/impls/bevy_pbr.rs#L48
// with ParallaxMaterial-added fields

use bevy::{asset::HandleId, prelude::*, render::render_resource::Face};
use bevy_inspector_egui::{
    egui,
    options::{NumberAttributes, OptionAttributes},
    Context, Inspectable,
};

use crate::ParallaxMaterial;

#[rustfmt::skip]
impl Inspectable for ParallaxMaterial {
    type Attributes = ();

    fn ui(&mut self, ui: &mut egui::Ui, _: Self::Attributes, context: &mut Context) -> bool {
        let mut changed = false;
        ui.vertical_centered(|ui| {
            egui::Grid::new(context.id()).show(ui, |ui| {
                egui::Grid::new("grid").show(ui, |ui| {
                    ui.label("base_color");
                    changed |= self.base_color.ui(ui, Default::default(), context);
                    ui.end_row();

                    ui.label("emissive");
                    changed |= self.emissive.ui(ui, Default::default(), context);
                    ui.end_row();

                    ui.label("perceptual_roughness");
                    changed |= self.perceptual_roughness.ui(ui, NumberAttributes::between(0.089, 1.0).with_speed(0.01), context);
                    ui.end_row();

                    ui.label("metallic");
                    changed |= self.metallic.ui(ui, NumberAttributes::normalized().with_speed(0.01), context);
                    ui.end_row();

                    ui.label("reflectance");
                    changed |= self.reflectance.ui(ui, NumberAttributes::positive(), context);
                    ui.end_row();

                    ui.label("unlit");
                    changed |= self.unlit.ui(ui, Default::default(), context);
                    ui.end_row();

                    ui.label("cull_mode");
                    egui::ComboBox::from_id_source("cull_mode")
                        .selected_text(format!("{:?}", self.cull_mode))
                        .show_ui(ui, |ui| {
                            changed |= ui.selectable_value(&mut self.cull_mode, None, "None").changed();
                            changed |= ui.selectable_value(&mut self.cull_mode, Some(Face::Front), "Front").changed();
                            changed |= ui.selectable_value(&mut self.cull_mode, Some(Face::Back), "Back").changed();
                        });
                    ui.end_row();

                    ui.label("flip_normal_map_y");
                    changed |= self.flip_normal_map_y.ui(ui, Default::default(), context);
                    ui.end_row();

                    ui.label("double_sided");
                    changed |= self.double_sided.ui(ui, Default::default(), context);
                    ui.end_row();

                    ui.label("depth_bias");
                    changed |= self.depth_bias.ui(ui, Default::default(), context);
                    ui.end_row();

                    ui.label("height_depth");
                    changed |= self.height_depth.ui(ui, NumberAttributes::between(0.0, 1.0).with_speed(0.0025), context);
                    ui.end_row();

                    ui.label("max_height_layers");
                    changed |= self.max_height_layers.ui(ui, NumberAttributes::between(2.0, 256.0).with_speed(1.0), context);
                    ui.end_row();

                    ui.label("algorithm");
                    changed |= self.algorithm.ui(ui, default(), context);
                    ui.end_row();

                    ui.label("alpha_mode");
                    egui::ComboBox::from_id_source("alpha_mode")
                        .selected_text(format!("{:?}", self.alpha_mode))
                        .show_ui(ui, |ui| {
                            changed |= ui.selectable_value(&mut self.alpha_mode, AlphaMode::Blend, "Blend").changed();
                            let alpha_mask = match self.alpha_mode {
                                AlphaMode::Mask(m) => m,
                                _ => 0.0
                            };
                            changed |= ui.selectable_value(&mut self.alpha_mode, AlphaMode::Mask(alpha_mask), "Mask").changed();
                            changed |= ui.selectable_value(&mut self.alpha_mode, AlphaMode::Opaque, "Opaque").changed();
                        });
                    ui.end_row();

                    if let AlphaMode::Mask(ref mut alpha_mask) = self.alpha_mode {
                        ui.label("alpha_mask");
                        changed |= alpha_mask.ui(ui, NumberAttributes::positive(), context);
                        ui.end_row();
                    }
                });
            });

            ui.collapsing("Textures", |ui| {
                egui::Grid::new("Textures").show(ui, |ui| {
                    let texture_option_attributes = OptionAttributes { replacement: Some(|| Handle::weak(HandleId::random::<Image>())), ..Default::default() };

                    ui.label("base_color");
                    changed |= self.base_color_texture.ui(ui, texture_option_attributes.clone(), &mut context.with_id(0));
                    ui.end_row();

                     ui.label("normal_map");
                     changed |= self.normal_map_texture.ui(ui, default(), &mut context.with_id(1));
                     ui.end_row();

                    ui.label("metallic_roughness");
                    changed |= self.metallic_roughness_texture.ui(ui, texture_option_attributes.clone(), &mut context.with_id(2));
                    ui.end_row();

                    ui.label("emmissive");
                    changed |= self.emissive_texture.ui(ui, texture_option_attributes.clone(), &mut context.with_id(3));
                    ui.end_row();

                    ui.label("occlusion texture");
                    changed |= self.occlusion_texture.ui(ui, texture_option_attributes, &mut context.with_id(4));
                    ui.end_row();

                    ui.label("height map");
                    changed |= self.height_map.ui(ui, default(), &mut context.with_id(4));
                    ui.end_row();
                });
            });
        });
        changed
    }
}
