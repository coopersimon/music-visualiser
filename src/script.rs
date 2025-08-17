use lalrpop_util::lalrpop_mod;
use std::{fs::File, io::Read};

use crate::renderer::{
    self, Renderable, RenderableType, Renderer, CreationError, Mapping, Scene
};

lalrpop_mod!(vis);

#[derive(Debug)]
pub enum ScriptParseError {
    UnrecognizedRenderable(String),
    UnrecognizedAudioParam(String),
    UnrecognizedRenderParam(String),
    // TODO: give this error a bit more info
    Line(String)
}

impl ScriptParseError {
    fn from_parse_error(err: lalrpop_util::ParseError<usize, vis::Token<'_>, ScriptParseError>, file_data: &str) -> Self {
        use ScriptParseError::*;
        match err {
            lalrpop_util::ParseError::User { error } => error,
            lalrpop_util::ParseError::InvalidToken { location } => Line(get_line_for_location(file_data, location).to_string()),
            lalrpop_util::ParseError::ExtraToken { token } => Line(get_line_for_location(file_data, token.0).to_string()),
            lalrpop_util::ParseError::UnrecognizedEof { location, .. } => Line(get_line_for_location(file_data, location).to_string()),
            lalrpop_util::ParseError::UnrecognizedToken { token, .. } => Line(get_line_for_location(file_data, token.0).to_string()),
        }
    }
}

#[derive(Debug)]
pub enum ScriptError {
    FileError(std::io::Error),
    ParseError(ScriptParseError),
    CreationError(CreationError)
}

impl From<std::io::Error> for ScriptError {
    fn from(value: std::io::Error) -> Self {
        ScriptError::FileError(value)
    }
}

impl From<ScriptParseError> for ScriptError {
    fn from(value: ScriptParseError) -> Self {
        ScriptError::ParseError(value)
    }
}

impl From<CreationError> for ScriptError {
    fn from(value: CreationError) -> Self {
        ScriptError::CreationError(value)
    }
}

pub fn parse_file(file_path: &str, renderer: &Renderer) -> Result<Scene, ScriptError> {
    let mut file = File::open(file_path)?;
    let mut file_data = String::new();
    file.read_to_string(&mut file_data)?;
    let scene = vis::SceneParser::new().parse(&file_data)
        .map_err(|e| ScriptParseError::from_parse_error(e, &file_data))?;
    Ok(Scene {
        render_list: scene.into_iter().map(|(renderable_type, params)| {
            create_renderable(renderable_type, params, renderer)
        }).collect::<Result<Vec<_>, _>>()?
    })
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