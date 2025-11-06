#![allow(static_mut_refs, clippy::all)]

use elara_engine::game_methods::*;
use elara_render_opengl::OglRenderApi;
use llm_arena::state::*;
use std::ffi::c_void;
use wasm_bindgen::prelude::*;
use web_sys::{console, BeforeUnloadEvent, ClipboardEvent, KeyboardEvent, MouseEvent, WheelEvent};

static mut GAME_STATE: Option<State> = None;
static mut GAME_METHODS: Option<GameMethods<OglRenderApi>> = None;

pub fn log(input: &str) {
    console::log_1(&input.into());
}

fn test_llm() {
    wasm_bindgen_futures::spawn_local(llm_test_async());
}

async fn llm_test_async() {
    log("doing test send");
}

#[wasm_bindgen(start)]
pub fn start() {
    unsafe {
        GAME_STATE = Some(State::new());
        GAME_METHODS = Some(GameMethods::<OglRenderApi> {
            init: llm_arena::game_init,
            update: llm_arena::game_loop,
        });

        let state_ref = GAME_STATE.as_mut().unwrap();
        let methods_ref = GAME_METHODS.as_ref().unwrap();

        let state_ptr = state_ref as *mut _ as *mut c_void;
        elara_platform_web::start(state_ptr, methods_ref);

        test_llm();
    }
}

#[wasm_bindgen]
pub fn main_loop() {
    unsafe {
        let state_ref = GAME_STATE.as_mut().unwrap();
        let methods_ref = GAME_METHODS.as_ref().unwrap();

        let state_ptr = state_ref as *mut _ as *mut c_void;
        elara_platform_web::update(state_ptr, methods_ref);
    }
}

#[wasm_bindgen]
pub fn key_down(event: KeyboardEvent) {
    elara_platform_web::key_down(event);
}

#[wasm_bindgen]
pub fn key_up(event: KeyboardEvent) {
    elara_platform_web::key_up(event);
}

#[wasm_bindgen]
pub fn mouse_down(event: MouseEvent) {
    elara_platform_web::mouse_down(event);
}

#[wasm_bindgen]
pub fn mouse_up(event: MouseEvent) {
    elara_platform_web::mouse_up(event);
}

#[wasm_bindgen]
pub fn mouse_move(event: MouseEvent) {
    elara_platform_web::mouse_move(event);
}

#[wasm_bindgen]
pub fn on_before_unload(event: BeforeUnloadEvent) {
    elara_platform_web::on_before_unload(event);
}

#[wasm_bindgen]
pub fn paste_handler(event: ClipboardEvent) {
    elara_platform_web::paste_handler(event);
}

#[wasm_bindgen]
pub fn mouse_wheel_handler(event: WheelEvent) {
    elara_platform_web::mouse_wheel_handler(event);
}
