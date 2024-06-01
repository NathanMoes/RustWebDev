use crate::*;
use std::collections::HashSet;

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
pub struct QuestionId(pub i32);

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
#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, ToSchema)]
pub struct Question {
    #[schema(example = "1")]
    pub id: QuestionId,
    #[schema(example = "What is rust?")]
    pub title: String,
    #[schema(example = "I want to know what rust is, can someone tell me?")]
    pub content: String,
    #[schema(example = "rust, programming, beginner")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tags: Option<HashSet<String>>,
}

/// An update question struct
///
/// This struct represents a question that can be updated via the API
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
#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, ToSchema)]
pub struct UpdateQuestion {
    #[schema(example = "1")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<QuestionId>,
    #[schema(example = "What is rust?")]
    pub title: String,
    #[schema(example = "I want to know what rust is, can someone tell me?")]
    pub content: String,
    #[schema(example = "rust, programming, beginner")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tags: Option<HashSet<String>>,
}

impl FromStr for QuestionId {
    type Err = std::io::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.parse::<i32>() {
            Ok(id) => Ok(QuestionId(id)),
            Err(_) => Err(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "Invalid id",
            )),
        }
    }
}

// Credit to knock knock for the format_tags function
pub fn format_tags(tags: &HashSet<String>) -> String {
    let taglist: Vec<&str> = tags.iter().map(String::as_ref).collect();
    taglist.join(", ")
}

/// Implementing the From trait for the Question struct to convert it to a string
impl From<&Question> for String {
    fn from(question: &Question) -> Self {
        let mut text: String = question.id.0.clone().to_string();
        text += "Question: \n";
        text += &format!("Title: {}\n", question.title);
        text += &format!("Content: {}\n", question.content);

        let mut annotations: Vec<String> = vec![format!("id: {}", question.id.0)];
        if let Some(tags) = &question.tags {
            annotations.push(format!("tags: {:?}", format_tags(tags)));
        }
        let annotations_text = annotations.join("; ");
        text += &format!("[{}]\n", annotations_text);
        text
    }
}

/// Implementing the Clone trait for the Question struct
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
