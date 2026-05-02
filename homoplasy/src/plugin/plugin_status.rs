use serde_repr::{Deserialize_repr, Serialize_repr};
use strum_macros::{EnumString, VariantNames};

#[derive(
    Copy, Clone,
    Debug, Default,
    PartialEq,
    EnumString, VariantNames,
    Deserialize_repr, Serialize_repr
)]
#[strum(ascii_case_insensitive)]
#[repr(usize)]
pub enum PluginStatus {
    #[default]
    NotStarted  = 1,
    Starting    = 2,
    Active      = 3,
    Stopped     = 4,
    Crashed     = 5,
}