use crate::state::*;
use elara_engine::{
    build_vars::*,
    color::*,
    input::*,
    platform_api::*,
    rect::*,
    render::{light::*, load_image_cursor, material::*, render_command::*, render_pack::*},
    state::State as EngineState,
    time::*,
    transform::*,
    typeface::*,
    ui,
    vectors::*,
};
use elara_render_opengl::*;
use std::{
    collections::HashMap,
    ffi::c_void,
    sync::{LazyLock, Mutex},
};
use tokio::runtime::Runtime;

pub mod ai_level_gen;
pub mod state;

use ai_level_gen::*;
use assets::*;

const GEN_RANGE: f64 = 300.0;

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

    gs.image_circle =
        load_image_cursor(include_bytes!("../resources/circle.png"), render_api).unwrap();

    // init world camera
    {
        let cam = &mut es.render_system.get_pack(RenderPackID::NewWorld).camera;

        cam.transform.local_position = VecThreeFloat::new(1.0, 27.0, 20.0);
        cam.pitch = 55.0;
        cam.yaw = 90.0;
        cam.move_target_position = cam.transform.local_position;
    }

    // init shop
    {
        let cam = &mut es.render_system.get_pack(RenderPackID::Shop).camera;

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
                    let resp = classify(&gs.prompt).await;

                    if let Ok(resp) = &resp {
                        gs.squares.clear();
                        gs.circles.clear();

                        for c in 0..resp.square_count {
                            let r = GEN_RANGE * f64::sqrt((platform_api.rand)());
                            let theta = (platform_api.rand)() * 2.0 * 3.14159;

                            let rand_pos = VecTwo::new(r * f64::cos(theta), r * f64::sin(theta));

                            gs.squares.push(rand_pos);
                        }

                        for c in 0..resp.circle_count {
                            let r = GEN_RANGE * f64::sqrt((platform_api.rand)());
                            let theta = (platform_api.rand)() * 2.0 * 3.14159;

                            let rand_pos = VecTwo::new(r * f64::cos(theta), r * f64::sin(theta));

                            gs.circles.push(rand_pos);
                        }
                    }

                    *AI_GEN_STATUS.lock().unwrap() = LevelGenerationStatus { status: Some(resp) };
                });
            }

            if let Ok(status) = AI_GEN_STATUS.lock() {
                if let Some(resp) = &status.status {
                    match resp {
                        Ok(level_gen_data) => {
                            if level_gen_data.valid {
                                ui::text(
                                    "Level Successfully Generated",
                                    &mut ui_frame_state,
                                    &mut gs.ui_context.as_mut().unwrap(),
                                );
                            } else {
                                ui::text(
                                    &format!("Prompt is invalid. {}", level_gen_data.error),
                                    &mut ui_frame_state,
                                    &mut gs.ui_context.as_mut().unwrap(),
                                );
                            }
                        }

                        Err(error) => {
                            ui::text(
                                "Error getting response",
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

    // render level
    {
        // render squares
        for pos in &gs.squares {
            let r = Rect::new_center(*pos, VecTwo::new(30.0, 30.0));

            let mut mat = Material::new();
            mat.shader = Some(es.shader_color);
            mat.set_color(COLOR_WHITE);

            es.render_system.add_command(
                RenderCommand::new_rect(&r, -1.0, 0.0, &mat),
                RenderPackID::World,
            );
        }

        // render circles
        for pos in &gs.circles {
            let r = Rect::new_center(*pos, VecTwo::new(30.0, 30.0));

            let mut mat = Material::new();
            mat.shader = Some(es.color_texture_shader);
            mat.set_color(COLOR_WHITE);
            mat.set_image(gs.image_circle.gl_id.unwrap());

            es.render_system.add_command(
                RenderCommand::new_rect(&r, -1.0, 0.0, &mat),
                RenderPackID::World,
            );
        }
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
