use std::fmt::Display;

pub type ID = crate::proto::Id;

impl ID {
    pub fn roll() -> Self {
        let uuid = uuid::Uuid::new_v4();
        ID {
            content: uuid.to_string(),
        }
    }
    pub fn roll_hash(bytes: &[u8]) -> Self {
        let uuid = uuid::Uuid::new_v5(&uuid::Uuid::NAMESPACE_URL, bytes);
        ID {
            content: uuid.to_string(),
        }
    }
}

impl From<String> for ID {
    fn from(s: String) -> Self {
        ID { content: s }
    }
}

impl Display for ID {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.content.fmt(f)
    }
}
