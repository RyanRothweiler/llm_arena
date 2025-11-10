use elara_engine::render::{material::*, shader::*};

pub mod asset_library;

pub use asset_library::*;

pub struct Assets {
    pub asset_library: AssetLibrary,
    pub missing_material: Material,
}

impl Assets {
    pub fn new() -> Self {
        Self {
            asset_library: AssetLibrary::new(),
            missing_material: Material::new(),
        }
    }

    fn _build_pbr_material(
        name_base: &str,
        asset_library: &AssetLibrary,
        pbr_shader: Shader,
    ) -> Material {
        let mut mat = Material::new();

        mat.shader = Some(pbr_shader);
        mat.uniforms.insert(
            "tex".to_string(),
            UniformData::Texture(TextureInfo {
                image_id: asset_library
                    .get_texture(&format!("{}_base_color", name_base))
                    .gl_id
                    .unwrap(),
                texture_slot: 0,
            }),
        );

        mat.uniforms.insert(
            "normalTex".to_string(),
            UniformData::Texture(TextureInfo {
                image_id: asset_library
                    .get_texture(&format!("{}_normal", name_base))
                    .gl_id
                    .unwrap(),
                texture_slot: 1,
            }),
        );

        mat.uniforms.insert(
            "metallicTex".to_string(),
            UniformData::Texture(TextureInfo {
                image_id: asset_library
                    .get_texture(&format!("{}_metallic", name_base))
                    .gl_id
                    .unwrap(),
                texture_slot: 2,
            }),
        );

        mat.uniforms.insert(
            "roughnessTex".to_string(),
            UniformData::Texture(TextureInfo {
                image_id: asset_library
                    .get_texture(&format!("{}_roughness", name_base))
                    .gl_id
                    .unwrap(),
                texture_slot: 3,
            }),
        );

        mat.uniforms.insert(
            "aoTex".to_string(),
            UniformData::Texture(TextureInfo {
                image_id: asset_library
                    .get_texture(&format!("{}_ao", name_base))
                    .gl_id
                    .unwrap(),
                texture_slot: 4,
            }),
        );

        mat.uniforms
            .insert("ambientRed".to_string(), UniformData::Float(0.05));

        return mat;
    }
}
