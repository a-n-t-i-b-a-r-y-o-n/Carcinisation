use std::fs::File;
use std::io::Read;
use std::path::PathBuf;

use anyhow::Result;
use prost_reflect::prost_types::FileDescriptorProto;
use regex::Regex;
use crate::plugin::file_descriptor_set::message::parse_message;
use crate::plugin::file_descriptor_set::metadata::{parse_package, parse_syntax};
use crate::plugin::file_descriptor_set::service::parse_service;

/// Remove single-line and block comments from the protobuf content
fn remove_comments(proto_content: &str) -> Result<String> {
    let mut output = String::from(proto_content);

    // Regex for comments
    let comment_regex = Regex::new(r"//.+|/\*[\n.]+\*/")?;

    // Iterate comment captures
    for capture in comment_regex.captures_iter(proto_content) {
        // Iterate valid matches
        for regex_match in capture.iter().filter_map(|m| m) {
            // Remove the match from the output string
            output = output.replace(regex_match.as_str(), "");
        }
    }

    Ok(output)
}

/// Sanitize semicolons at each line ending
fn sanitize_semicolons(proto_content: &str) -> Result<String> {
    let mut output = String::from(proto_content);

    // Regex for duplicate semicolons and semicolons with leading spaces
    let semicolon_regex = Regex::new(r"\s+;|;{2,}")?;

    // Iterate comment captures
    for capture in semicolon_regex.captures_iter(proto_content) {
        // Iterate valid matches
        for regex_match in capture.iter().filter_map(|m| m) {
            // Replace the match with a sane semicolon
            output = output.replace(regex_match.as_str(), ";");
        }
    }

    Ok(output)
}

/// Parse a .proto file to a FileDescriptorProto
pub(super) fn parse_protobuf_file(proto_file: &PathBuf) -> Result<FileDescriptorProto> {
    // Get the input filename
    let filename = proto_file.file_name().unwrap()
        .display().to_string();

    // Read the input file
    let mut input_file = File::open(proto_file)?;
    let mut input_content = String::new();
    input_file.read_to_string(&mut input_content)?;

    // Scrub comments
    let input_content = remove_comments(&input_content)?;

    // Sanitize semicolons
    let input_content= sanitize_semicolons(&input_content)?;

    // Regex for parsing tokens
    let token_regex = Regex::new(r"(?<token>\S+)")?;

    // Use the regex to get a full list of tokens
    let mut tokens = token_regex.captures_iter(&input_content)
        .filter_map(|c| c.name("token"))
        .map(|m| m.as_str())
        .into_iter()
        .peekable();

    // File descriptor to build
    let mut file_descriptor_proto = FileDescriptorProto {
        name: Some(filename),
        package: None,
        dependency: vec![],
        public_dependency: vec![],
        weak_dependency: vec![],
        message_type: vec![],
        enum_type: vec![],
        service: vec![],
        extension: vec![],
        options: None,
        source_code_info: None,
        syntax: None,
    };

    // Package name
    let mut package = String::new();

    // Iterate tokens
    loop {
        // Get the next token or break
        let token = match tokens.next() {
            Some(token) => token,
            None => break,
        };

        match token {
            "syntax" => {
                file_descriptor_proto.syntax = Some(parse_syntax(&mut tokens)?);
            }
            "package" => {
                package = parse_package(&mut tokens)?;
                file_descriptor_proto.package = Some(package.clone());
            }
            "service" => {
                let service = parse_service(&package, &mut tokens)?;

                file_descriptor_proto.service.push(service);
            }
            "message" => {
                let message = parse_message(&package, &mut tokens)?;

                file_descriptor_proto.message_type.push(message);
            }
            unknown => {
                println!("Unexpected token in proto file: {}", unknown);
            }
        };
    }

    Ok(file_descriptor_proto)
}

