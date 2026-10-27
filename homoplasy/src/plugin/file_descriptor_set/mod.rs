use std::path::PathBuf;
use anyhow::Result;
use prost_reflect::DescriptorPool;
use prost_reflect::prost_types::FileDescriptorSet;

use crate::plugin::file_descriptor_set::parser::parse_protobuf_file;

mod field;
mod message;
mod metadata;
mod parser;
mod service;

/// Generate a FileDescriptorSet from a given .proto file
pub(crate) fn generate_file_descriptor_set(proto_file: &PathBuf) -> Result<FileDescriptorSet> {

    let file_descriptor = parse_protobuf_file(proto_file)?;

    let file_descriptor_set = FileDescriptorSet {
        file: vec!(file_descriptor),
    };

    Ok(file_descriptor_set)
}

/// Generate a DescriptorPool from a given .proto file
pub(crate) fn generate_descriptor_pool(proto_file: &PathBuf) -> Result<DescriptorPool> {
    let file_descriptor_set = generate_file_descriptor_set(proto_file)?;
    
    let descriptor_pool = DescriptorPool::from_file_descriptor_set(file_descriptor_set)?;
    
    Ok(descriptor_pool)
}