use anyhow::{anyhow, Result};
use prost_reflect::prost_types::{MethodDescriptorProto, ServiceDescriptorProto};

/// Parse an RPC input or output type
fn parse_input_output<'t, T>(tokens: &mut T) -> Result<(bool, String)>
where
    T: Iterator<Item = &'t str>
{
    let mut message_type = "";
    let mut streaming = false;

    // Iterate tokens
    loop {
        match tokens.next() {
            Some(token) => {
                match token {
                    ")" => continue,
                    "(" => continue,
                    ");" | ";" => return Ok((false, String::new())),
                    "(stream" | "stream" => {
                        streaming = true;
                        continue;
                    }
                    message => {
                        message_type = message
                            .trim_end_matches(';')
                            .trim_start_matches('(')
                            .trim_end_matches(')');

                        // Capture the trailing token, if any
                        if !message.contains(')') {
                            _ = tokens.next();
                        }

                        break;
                    }
                }
            }
            None => break
        }
    }

    Ok((streaming, String::from(message_type)))
}

/// Parse an RPC method
fn parse_method<'t, T>(prefix: &str, tokens: &mut T) -> Result<MethodDescriptorProto>
where
    T: Iterator<Item = &'t str>
{
    let name = match tokens.next() {
        Some(token) => token,
        None => return Err(anyhow!("Failed to parse RPC name"))
    };

    // Method descriptor to build
    let mut method_descriptor_proto = MethodDescriptorProto {
        name: Some(String::from(name)),
        input_type: None,
        output_type: None,
        options: None,
        client_streaming: None,
        server_streaming: None,
    };

    // Input request type
    let (client_stream, input_type) = parse_input_output(tokens)?;

    // "returns"
    _ = tokens.next();

    // Output response type
    let (server_stream, output_type) = parse_input_output(tokens)?;

    // Set client and server streaming as expected
    if client_stream { method_descriptor_proto.client_streaming = Some(true); }
    if server_stream { method_descriptor_proto.server_streaming = Some(true); }

    // Set the fully-qualified input type
    if !input_type.is_empty() {
        method_descriptor_proto.input_type = Some(format!("{}.{}", prefix, input_type));
    }

    // Set the fully-qualified output type
    if !output_type.is_empty() {
        method_descriptor_proto.output_type = Some(format!("{}.{}", prefix, output_type));
    }

    Ok(method_descriptor_proto)
}

/// Parse an RPC service
pub(super) fn parse_service<'t, T>(package: &str, tokens: &mut T) -> Result<ServiceDescriptorProto>
where
    T: Iterator<Item = &'t str>
{
    // Service name
    let name = match tokens.next() {
        Some(token) => token,
        None => return Err(anyhow!("Failed to parse service name"))
    };

    // Opening curly brace
    _ = tokens.next();

    // Service descriptor to build up
    let mut service_descriptor_proto = ServiceDescriptorProto {
        name: Some(String::from(name)),
        method: vec![],
        options: None,
    };

    // Iterate tokens
    loop {
        // Get the next token or break
        let token = match tokens.next() {
            Some(token) => token,
            None => break,
        };

        match token {
            "rpc" => {
                // Parse the RPC method
                let method = parse_method(&format!(".{}", package), tokens)?;

                // Append to our service
                service_descriptor_proto.method.push(method);
            }
            "}" => break,
            unknown => println!("Unexpected token in service: {}", unknown),
        }
    }

    Ok(service_descriptor_proto)
}