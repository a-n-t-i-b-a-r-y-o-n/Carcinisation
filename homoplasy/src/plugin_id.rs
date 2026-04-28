use std::fmt::{Display, Formatter};
use std::hash::{Hash, Hasher};
use uuid::Uuid;

#[derive(Debug)]
pub struct PluginID {
    uuid: Uuid
}

impl PluginID {
    pub fn new() -> Self {
        Self {
            uuid: Uuid::new_v4(),
        }
    }
}

impl Display for PluginID {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.uuid)
    }
}

impl PartialEq<Self> for PluginID {
    fn eq(&self, other: &Self) -> bool {
        self.uuid == other.uuid
    }
}

impl Eq for PluginID { }

impl Hash for PluginID {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.uuid.hash(state)
    }
}