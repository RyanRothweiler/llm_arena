use elara_engine::{render::image::Image, typeface::*, ui::*, vectors::*};

pub mod assets;

use assets::*;

pub struct State {
    pub assets: Assets,

    pub image_circle: Image,

    pub font_style_body: FontStyle,
    pub font_style_header: FontStyle,
    pub font_style_nav: FontStyle,

    pub ui_context: Option<Context>,

    pub prompt: String,

    pub squares: Vec<VecTwo>,
    pub circles: Vec<VecTwo>,
}

impl State {
    pub fn new() -> Self {
        State {
            assets: Assets::new(),
            ui_context: None,

            image_circle: Image::new(),

            font_style_body: Default::default(),
            font_style_header: Default::default(),
            font_style_nav: Default::default(),

            prompt: String::new(),

            squares: vec![],
            circles: vec![],
        }
    }
}
