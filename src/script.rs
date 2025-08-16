use lalrpop_util::lalrpop_mod;
use std::{fs::File, io::Read};

use crate::renderer::{
    self, Renderable, RenderableType, Renderer, CreationError, Mapping
};

lalrpop_mod!(vis);

pub fn parse_file(file_path: &str, renderer: &Renderer) -> Vec<Box<dyn Renderable>> {
    let mut file = File::open(file_path).expect("could not open script file");
    let mut file_data = String::new();
    file.read_to_string(&mut file_data).expect("could not read script file");
    let scene = vis::SceneParser::new()
        .parse(&file_data)
        .expect("could not parse script file");
    scene.into_iter().map(|(renderable_type, params)| {
        create_renderable(renderable_type, params, renderer).expect("could not create renderable")
    }).collect()
}

fn create_renderable(renderable_type: RenderableType, params: Mapping, renderer: &Renderer) -> Result<Box<dyn Renderable>, CreationError> {
    Ok(match renderable_type {
        RenderableType::Circle =>   Box::new(renderer::circle::CircleRenderable::new(params, renderer)?),
        RenderableType::Quad =>     Box::new(renderer::quad::QuadRenderable::new(params, renderer)?),
    })
}