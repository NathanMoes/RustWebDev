use crate::*;

/// A question id struct
///
/// This struct is used to represent the id of a question. Why, because the book said so, that's why.
/// ##Example:
/// ```
/// {
/// "id": "1"
/// }
/// ```
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq, Hash)]
pub struct QuestionId(pub String);

/// A question struct
///
/// This struct represents a question that can be asked and (future) answered via the API
/// ##Example:
/// ```
/// {
///    "id": "1",
///    "title": "What is cargo toml?",
///    "content": "I want to know what toml is and how it relates to cargo. Can someone explain?",
///    "tags": ["rust", "toml", "cargo"]
/// }
/// ```
///
#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct Question {
    pub id: QuestionId,
    pub title: String,
    pub content: String,
    pub tags: Option<Vec<String>>,
}

impl FromStr for QuestionId {
    type Err = std::io::Error;

    fn from_str(id: &str) -> Result<Self, Self::Err> {
        match id.is_empty() {
            false => Ok(QuestionId(id.to_string())),
            true => Err(Error::new(ErrorKind::InvalidInput, "No id provided")),
        }
    }
}

impl Clone for Question {
    fn clone(&self) -> Self {
        Question {
            id: self.id.clone(),
            title: self.title.clone(),
            content: self.content.clone(),
            tags: self.tags.clone(),
        }
    }
}
