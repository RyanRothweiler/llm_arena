#![allow(dead_code, unused_variables)]

use std::{fs::File, io::Write};

fn main() {
    let manifest_file =
        std::fs::read_to_string("resources/manifest.txt").expect("Require asset manifest file.");
    let mut output_file = File::create("src/state/assets/asset_library/GEN_game_assets.rs")
        .expect("Couldn't open output file");

    let imp = r#"
#![allow(unused_imports)]
use elara_engine::{
    model::*,
    render::{image::*, RenderApi, *},
};
use crate::{AssetLibrary, include_model, include_texture};
use std::collections::HashMap;
"#;
    writeln!(output_file, "{}", imp).unwrap();

    writeln!(
        output_file,
        "pub fn load_game_assets(al: &mut AssetLibrary, render_api: &mut impl RenderApi) {{"
    )
    .unwrap();

    let lines: Vec<String> = manifest_file.lines().map(|line| line.to_string()).collect();

    if !lines.is_empty() {
        let mut i = 0;
        loop {
            if !lines[i].trim().is_empty() {
                let asset_type: &str = lines[i].trim();
                let id: &str = lines[i + 1].trim();

                match asset_type {
                    "texture" => {
                        let path: &str = lines[i + 2].trim();
                        i += 3;

                        write_texture(&mut output_file, id, path);
                    }
                    "model" => {
                        let path: &str = lines[i + 2].trim();
                        i += 3;

                        write_model(&mut output_file, id, path);
                    }
                    "tile" => {
                        i += 2;

                        write_model(&mut output_file, id, &format!("tiles/{}/{}.obj", id, id));
                        write_texture(
                            &mut output_file,
                            &format!("{}_base_color", id),
                            &format!("tiles/{}/BaseColor.png", id),
                        );
                        write_texture(
                            &mut output_file,
                            &format!("{}_metallic", id),
                            &format!("tiles/{}/Metallic.png", id),
                        );
                        write_texture(
                            &mut output_file,
                            &format!("{}_roughness", id),
                            &format!("tiles/{}/Roughness.png", id),
                        );
                        write_texture(
                            &mut output_file,
                            &format!("{}_ao", id),
                            &format!("tiles/{}/AO.png", id),
                        );
                        write_texture(
                            &mut output_file,
                            &format!("{}_normal", id),
                            &format!("tiles/{}/Normal.png", id),
                        );
                    }
                    "pack" => {
                        i += 2;

                        write_model(&mut output_file, id, &format!("packs/{}/{}.obj", id, id));
                        write_texture(
                            &mut output_file,
                            &format!("{}_base_color", id),
                            &format!("packs/{}/BaseColor.png", id),
                        );
                        write_texture(
                            &mut output_file,
                            &format!("{}_metallic", id),
                            &format!("packs/{}/Metallic.png", id),
                        );
                        write_texture(
                            &mut output_file,
                            &format!("{}_roughness", id),
                            &format!("packs/{}/Roughness.png", id),
                        );
                        write_texture(
                            &mut output_file,
                            &format!("{}_ao", id),
                            &format!("packs/{}/AO.png", id),
                        );
                        write_texture(
                            &mut output_file,
                            &format!("{}_normal", id),
                            &format!("packs/{}/Normal.png", id),
                        );
                    }

                    _ => panic!("Unknown asset type {}", asset_type),
                }
            }
            i += 1;

            if i >= lines.len() {
                break;
            }
        }
    }

    writeln!(output_file, "}}").unwrap();
}

fn write_texture(output_file: &mut File, id: &str, path: &str) {
    write!(output_file, "include_texture!(al, ").unwrap();
    write!(output_file, "\"{}\", ", id).unwrap();
    write!(output_file, "\"../../../../resources/{}\", ", path).unwrap();
    writeln!(output_file, "render_api);").unwrap();
}

fn write_model(output_file: &mut File, id: &str, path: &str) {
    write!(output_file, "include_model!(al, ").unwrap();
    write!(output_file, "\"{}\", ", id).unwrap();
    write!(output_file, "\"../../../../resources/{}\", ", path).unwrap();
    writeln!(output_file, "render_api);").unwrap();
}
