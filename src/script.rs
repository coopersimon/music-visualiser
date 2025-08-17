use lalrpop_util::lalrpop_mod;
use std::{fs::File, io::Read};

use crate::renderer::{
    self, Renderable, RenderableType, Renderer, CreationError, Mapping
};

#[derive(Debug)]
pub enum UserError {
    UnrecognizedRenderable(String),
    UnrecognizedAudioParam(String),
    UnrecognizedRenderParam(String)
}

#[derive(Debug)]
pub enum ParseError {

}

#[derive(Debug)]
pub enum ScriptError {
    FileError(std::io::Error),
    ParseError(ParseError),
    CreationError(CreationError)
}

lalrpop_mod!(vis);

// TODO: improve error handling here.
pub fn parse_file(file_path: &str, renderer: &Renderer) -> Vec<Box<dyn Renderable>> {
    let mut file = File::open(file_path).expect("could not open script file");
    let mut file_data = String::new();
    file.read_to_string(&mut file_data).expect("could not read script file");
    let scene = match vis::SceneParser::new().parse(&file_data) {
        Ok(s) => s,
        Err(e) => match e {
            lalrpop_util::ParseError::InvalidToken { location } => {
                let s = get_line_for_location(&file_data, location);
                panic!("got error: {}", s)
            },
            _ => panic!("error: {:?}", e)
        }
    };
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

// Given a location into a file, get the entire line that the location is on.
fn get_line_for_location<'a>(file_data: &'a str, location: usize) -> &'a str {
    let mut line_start = 0;
    for (i, c) in file_data.chars().enumerate().take(location) {
        if c == '\n' {
            line_start = i + 1;
        }
    }
    if let Some((line_end, _)) = file_data.chars().enumerate()
        .skip(location)
        .find(|(_, c)| *c == '\n') {
        &file_data[line_start..line_end]
    } else {
        &file_data[line_start..]
    }
}