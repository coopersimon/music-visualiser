use lalrpop_util::lalrpop_mod;
use std::{fs::File, io::Read};

use crate::renderer::{
    self, object::{ObjectRenderable, ObjectType}, Renderer, CreationError, Mapping, Display
};

lalrpop_mod!(vis);

#[derive(Debug)]
pub enum ScriptError {
    FileError(std::io::Error),
    UnrecognizedObject(String),
    UnrecognizedAudioParam(String),
    UnrecognizedRenderParam(String),
    CreationError(CreationError),
    // TODO: give this error a bit more info
    Line(String)
}

impl ScriptError {
    fn from_parse_error(err: lalrpop_util::ParseError<usize, vis::Token<'_>, ScriptError>, file_data: &str) -> Self {
        use ScriptError::*;
        match err {
            lalrpop_util::ParseError::User { error } => error,
            lalrpop_util::ParseError::InvalidToken { location } => Line(get_line_for_location(file_data, location).to_string()),
            lalrpop_util::ParseError::ExtraToken { token } => Line(get_line_for_location(file_data, token.0).to_string()),
            lalrpop_util::ParseError::UnrecognizedEof { location, .. } => Line(get_line_for_location(file_data, location).to_string()),
            lalrpop_util::ParseError::UnrecognizedToken { token, .. } => Line(get_line_for_location(file_data, token.0).to_string()),
        }
    }
}

impl From<std::io::Error> for ScriptError {
    fn from(value: std::io::Error) -> Self {
        ScriptError::FileError(value)
    }
}

impl From<CreationError> for ScriptError {
    fn from(value: CreationError) -> Self {
        ScriptError::CreationError(value)
    }
}

pub fn parse_file(file_path: &str, renderer: &Renderer) -> Result<Display, ScriptError> {
    let mut file = File::open(file_path)?;
    let mut file_data = String::new();
    file.read_to_string(&mut file_data)?;
    vis::DisplayParser::new().parse(renderer, &file_data)
        .map_err(|e| ScriptError::from_parse_error(e, &file_data))
}

pub fn create_object(object_type: ObjectType, params: Mapping, renderer: &Renderer) -> Result<Box<dyn ObjectRenderable>, CreationError> {
    Ok(match object_type {
        ObjectType::Circle => Box::new(renderer::object::circle::CircleRenderable::new(params, renderer)?),
        ObjectType::Quad =>   Box::new(renderer::object::quad::QuadRenderable::new(params, renderer)?),
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