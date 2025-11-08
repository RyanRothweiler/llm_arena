#![allow(
    unused_imports,
    unused_variables,
    clippy::all,
    unused_mut,
    unreachable_code
)]

use crate::state::*;
use elara_engine::{
    build_vars::*,
    input::*,
    platform_api::*,
    rect::*,
    render::{RenderApi, light::*, render_pack::*, shader::*, vao::*},
    rigel_ui::*,
    state::State as EngineState,
    time::*,
    transform::*,
    typeface::*,
    ui,
    vectors::*,
};
use elara_render_opengl::*;
use kalosm::language::*;
use std::{
    collections::HashMap,
    ffi::c_void,
    io::Cursor,
    mem,
    net::UdpSocket,
    path::Path,
    slice,
    sync::{LazyLock, Mutex},
};
use tokio::runtime::Runtime;

pub mod ai_level_gen;
pub mod state;

use ai_level_gen::*;
use assets::*;

#[derive(Debug)]
pub struct LevelGenerationStatus {
    status: Option<Result<LevelGenResponse, AIError>>,
}

pub static AI_GEN_STATUS: LazyLock<Mutex<LevelGenerationStatus>> =
    LazyLock::new(|| Mutex::new(LevelGenerationStatus { status: None }));

#[unsafe(no_mangle)]
pub fn game_init(
    game_state_ptr: *mut c_void,
    es: &mut EngineState,
    render_api: &mut OglRenderApi,
    platform_api: &PlatformApi,
) {
    let gs = unsafe { &mut *(game_state_ptr as *mut State) };

    elara_engine::debug::init_context(
        es.shader_color.clone(),
        es.shader_color_ui,
        es.model_sphere.clone(),
        es.model_plane.clone(),
    );

    load_game_assets(&mut gs.assets.asset_library, render_api);

    // init world camera
    {
        let mut cam = &mut es.render_system.get_pack(RenderPackID::NewWorld).camera;

        cam.transform.local_position = VecThreeFloat::new(1.0, 27.0, 20.0);
        cam.pitch = 55.0;
        cam.yaw = 90.0;
        cam.move_target_position = cam.transform.local_position;
    }

    // init shop
    {
        let mut cam = &mut es.render_system.get_pack(RenderPackID::Shop).camera;

        cam.transform.local_position = VecThreeFloat::new(0.0, 23.0, 11.0);
        cam.pitch = 70.0;
        cam.yaw = 90.0;
        cam.move_target_position = cam.transform.local_position;
    }

    // lights
    {
        // new world light
        {
            let light = Light::new(es.components.new_transform());

            let ct: &mut Transform = &mut es.components.transforms[light.transform];
            ct.local_position.x = -2.0;
            ct.local_position.z = 10.0;
            ct.local_position.y = 15.0;

            es.render_system
                .get_pack(RenderPackID::NewWorld)
                .lights
                .push(light);
        }
    }

    // setup font styles
    {
        gs.font_style_body = FontStyle {
            size: 1.7,
            typeface: es.roboto_typeface.get_weight(TypeWeight::Regular),
        };

        gs.font_style_header = FontStyle {
            size: 4.0,
            typeface: es.roboto_typeface.get_weight(TypeWeight::Bold),
        };

        gs.font_style_nav = FontStyle {
            size: 3.0,
            typeface: es.roboto_typeface.get_weight(TypeWeight::Bold),
        };
    }
}

// Prev delta time is in seconds. So for 60 fps 0.016666.
#[unsafe(no_mangle)]
pub fn game_loop(
    prev_delta_time: f64,
    game_state_ptr: *mut c_void,
    es: &mut EngineState,
    input: &mut Input,
    render_api: &mut OglRenderApi,
    platform_api: &PlatformApi,
) {
    let frame_time = Time::from_seconds(prev_delta_time);
    let gs = unsafe { &mut *(game_state_ptr as *mut State) };

    elara_engine::debug::init_context(
        es.shader_color.clone(),
        es.shader_color_ui.clone(),
        es.model_sphere.clone(),
        es.model_plane.clone(),
    );
    elara_engine::debug::frame_start();

    // update ui_context
    {
        let ui_context = gs.ui_context.get_or_insert(ui::Context {
            mouse: input.mouse.clone(),
            keyboard: input.keyboard.clone(),
            paste: None,

            color_shader: es.shader_color_ui,
            color_shader_texture: es.color_texture_shader,

            font_body: gs.font_style_body.clone(),
            font_header: gs.font_style_header.clone(),
            font_nav: gs.font_style_nav.clone(),

            render_commands: vec![],

            button_state: HashMap::new(),
            input_fields: HashMap::new(),

            delta_time: prev_delta_time,

            selected_input_field: None,
        });

        // update per frame context data
        ui_context.mouse = input.mouse.clone();
        ui_context.keyboard = input.keyboard.clone();
        ui_context.delta_time = prev_delta_time;
        ui_context.paste = input.paste.clone();
    }

    let mut ui_frame_state = ui::FrameState::new(&input, es.window_resolution);

    // ui stuff
    {
        let r = Rect::new_top_size(VecTwo::new(0.0, 0.0), 300.0, 500.0);
        ui::begin(r, &mut ui_frame_state, &mut gs.ui_context.as_mut().unwrap());
        {
            ui::input_field(
                "Prompt",
                "prompt",
                &mut gs.prompt,
                VecTwo::new(10.0, 40.0),
                280.0,
                &gs.font_style_body.clone(),
                &gs.font_style_body.clone(),
                &mut ui_frame_state,
                gs.ui_context.as_mut().unwrap(),
                std::line!(),
            );

            ui_frame_state.cursor.y += 80.0;

            if ui::button(
                "Run Classification",
                &mut ui_frame_state,
                std::line!(),
                gs.ui_context.as_mut().unwrap(),
            ) {
                let rt = Runtime::new().unwrap();
                rt.block_on(async {
                    let resp = test_gen(&gs.prompt).await;
                    *AI_GEN_STATUS.lock().unwrap() = LevelGenerationStatus { status: Some(resp) };
                });
            }

            if let Ok(status) = AI_GEN_STATUS.lock() {
                if let Some(resp) = &status.status {
                    match resp {
                        Ok(level_gen_data) => {
                            ui::text(
                                "Level Successfully Generated",
                                &mut ui_frame_state,
                                &mut gs.ui_context.as_mut().unwrap(),
                            );
                        }

                        Err(error) => {
                            ui::text(
                                "Error generating level",
                                &mut ui_frame_state,
                                &mut gs.ui_context.as_mut().unwrap(),
                            );
                        }
                    }
                }

                ui::text(
                    &format!("{:?}", status),
                    &mut ui_frame_state,
                    &mut gs.ui_context.as_mut().unwrap(),
                );
            }
        }

        ui::end(&mut ui_frame_state, &mut gs.ui_context.as_mut().unwrap());
    }

    es.render_system
        .render_packs
        .get_mut(&RenderPackID::UI)
        .unwrap()
        .commands
        .append(&mut gs.ui_context.as_mut().unwrap().render_commands);

    // get draw calls
    if build_type_development() {
        es.render_commands_len = 0;
        for (key, value) in &es.render_system.render_packs {
            es.render_commands_len += value.commands.len() as i32;
        }
    }

    es.game_ui_debug_render_commands = elara_engine::debug::get_ui_render_list().clone();
    es.game_debug_render_commands = elara_engine::debug::get_render_list().clone();
}
