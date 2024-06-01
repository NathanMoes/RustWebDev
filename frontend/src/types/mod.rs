use crate::*;

#[derive(Serialize)]
pub struct Question {
    pub id: u32,
    pub title: String,
    pub content: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tags: Option<HashSet<String>>,
}
