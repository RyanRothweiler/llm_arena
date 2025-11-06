use llm_arena::state::*;
use std::ffi::c_void;

fn main() {
    let mut game_state = State::new();
    elara_platform_windows::platform_main("llm_arena", &mut game_state as *mut _ as *mut c_void);
}
