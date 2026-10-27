use std::str::FromStr;
use anyhow::{anyhow, Result};
use convert_case::ccase;
use prost_reflect::prost_types::field_descriptor_proto::{Label, Type};
use prost_reflect::prost_types::FieldDescriptorProto;

/// Map protobuf types from their string representation
fn map_field_type(type_name: &str) -> Type {
    match type_name {
        "bool"      => Type::Bool,
        "bytes"     => Type::Bytes,
        "double"    => Type::Double,
        "enum"      => Type::Enum,
        "float"     => Type::Float,
        "fixed32"   => Type::Fixed32,
        "fixed64"   => Type::Fixed64,
        "group"     => Type::Group,
        "int32"     => Type::Int32,
        "int64"     => Type::Int64,
        "sfixed32"  => Type::Sfixed32,
        "sfixed64"  => Type::Sfixed64,
        "sint32"    => Type::Sint32,
        "sint64"    => Type::Sint64,
        "string"    => Type::String,
        "uint32"    => Type::Uint32,
        "uint64"    => Type::Uint64,
        _           => Type::Message,
    }
}

/// Parse a message field or oneof option
pub(super) fn parse_field_or_option<'t, T>(package: &str, oneof_index: Option<i32>, tokens: &mut T) -> Result<FieldDescriptorProto>
where
    T: Iterator<Item = &'t str>
{
    let mut type_name = match tokens.next() {
        Some(token) => String::from(token),
        None => return Err(anyhow!("Failed to parse field type name in {} package", package))
    };

    let field_type = map_field_type(&type_name);

    // Qualify the type name for messages
    if field_type == Type::Message {
        type_name = format!(".{}.{}", package, type_name);
    }

    let field_name = match tokens.next() {
        Some(token) => token,
        None => return Err(anyhow!("Failed to parse {}-type field name in {} package", type_name, package))
    };

    let json_name = ccase!(camel, &field_name);

    // Equals sign
    _ = tokens.next();

    let field_number = match tokens.next() {
        Some(token) => {
            let token = token.trim_end_matches(';');
            i32::from_str(token)?
        }
        None => return Err(anyhow!("Failed to parse {} field number", field_name))
    };

    // Field descriptor to return
    let field_descriptor = FieldDescriptorProto {
        name: Some(String::from(field_name)),
        number: Some(field_number),
        label: Some(Label::Optional.into()),        // TODO: Handle labels instead of assuming optional
        r#type: Some(field_type.into()),
        type_name: if field_type == Type::Message { Some(type_name) } else { None },
        extendee: None,
        default_value: None,
        oneof_index,
        json_name: Some(json_name),
        options: None,
        proto3_optional: None,
    };

    Ok(field_descriptor)
}