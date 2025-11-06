use elara_engine::{
    model::*,
    render::{frame_buffer_pack::*, image::Image, material::*, vao::*},
    state::State as EngineState,
    transform::*,
    typeface::*,
    ui::*,
    vectors::*,
};
use std::{collections::HashMap, net::UdpSocket};

pub mod assets;

use assets::*;

pub struct State {
    pub assets: Assets,

    pub font_style_body: FontStyle,
    pub font_style_header: FontStyle,
    pub font_style_nav: FontStyle,

    pub ui_context: Option<Context>,
}

impl State {
    pub fn new() -> Self {
        State {
            assets: Assets::new(),
            ui_context: None,

            font_style_body: Default::default(),
            font_style_header: Default::default(),
            font_style_nav: Default::default(),
        }
    }
}
