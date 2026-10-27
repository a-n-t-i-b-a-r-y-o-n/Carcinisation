use std::iter::Peekable;
use anyhow::{anyhow, Result};
use prost_reflect::prost_types::{DescriptorProto, FieldDescriptorProto, OneofDescriptorProto};
use crate::plugin::file_descriptor_set::field::parse_field_or_option;

/// Parse a oneof and its options
fn parse_oneof<'t, T>(package: &str, index: i32, tokens: &mut Peekable<T>) -> Result<(OneofDescriptorProto, Vec<FieldDescriptorProto>)>
where
    T: Iterator<Item = &'t str>,
{
    let name = match tokens.next() {
        Some(token) => token,
        None => return Err(anyhow!("Failed to parse oneof name")),
    };

    let oneof_descriptor = OneofDescriptorProto {
        name: Some(String::from(name)),
        options: None,
    };

    // Opening curly brace
    _ = tokens.next();

    // List of fields to build
    let mut fields = vec!();

    loop {
        let token = match tokens.peek() {
            Some(token) => token,
            None => break,
        };

        match *token {
            "}" => {
                // Advance the iterator to what we peeked
                _ = tokens.next();

                break;
            }
            _token => {
                let field = parse_field_or_option(&package, Some(index), tokens)?;

                fields.push(field);
            }
        }
    }

    Ok((oneof_descriptor, fields))
}

/// Parse the syntax field in a proto file
pub(super) fn parse_message<'t, T>(package: &str, tokens: &mut Peekable<T>) -> Result<DescriptorProto>
where
    T: Iterator<Item = &'t str>
{
    // Message type name
    let name = match tokens.next() {
        Some(token) => token,
        None => return Err(anyhow!("Failed to parse message name"))
    };

    // Opening curly brace
    _ = tokens.next();

    // Message descriptor to build
    let mut message_descriptor_proto = DescriptorProto {
        name: Some(String::from(name)),
        field: vec![],
        extension: vec![],
        nested_type: vec![],
        enum_type: vec![],
        extension_range: vec![],
        oneof_decl: vec![],
        options: None,
        reserved_range: vec![],
        reserved_name: vec![],
    };

    // Iterate tokens
    loop {
        // Peek at the next token or break
        let token = match tokens.peek() {
            Some(token) => token,
            None => break,
        };

        match *token {
            "message" => {
                // Advance the iterator to what we peeked
                _ = tokens.next();

                let nested = parse_message(&format!(".{}.{}", package, name), tokens)?;

                message_descriptor_proto.nested_type.push(nested);
            }
            "oneof" => {
                // Advance the iterator to what we peeked
                _ = tokens.next();

                // The index of this oneof
                let index = message_descriptor_proto.oneof_decl.len() as _;

                let package = format!("{}.{}", package, name);

                let (oneof_descriptor, fields) = parse_oneof(&package, index, tokens)?;

                message_descriptor_proto.oneof_decl.push(oneof_descriptor);
                message_descriptor_proto.field.extend(fields);
            }
            "}" => {
                // Advance the iterator to what we peeked
                _ = tokens.next();

                break;
            }
            _token => {
                // Fully-qualified package name
                let package = format!("{}.{}", package, name);

                let field = parse_field_or_option(&package, None, tokens)?;

                message_descriptor_proto.field.push(field);
            }
        }
    }

    Ok(message_descriptor_proto)
}