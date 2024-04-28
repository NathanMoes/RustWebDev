use crate::*;

/// Application state struct
/// This struct is used to hold the state of the application, which is currently only the questions for the API
#[derive(Clone, Debug)]
pub struct AppState {
    pub questions: Arc<RwLock<HashMap<QuestionId, Question>>>,
}

/// Implementing the AppState struct with basic functions to use for API and state management operations
impl AppState {
    pub fn new() -> Self {
        AppState {
            questions: Arc::new(RwLock::new(self::AppState::init())),
        }
    }

    /// Function to initialize the questions hashmap by reading in the questions from a json file
    pub fn init() -> HashMap<QuestionId, Question> {
        let file = include_str!("../questions.json");
        let questions: HashMap<QuestionId, Question> =
            serde_json::from_str::<HashMap<QuestionId, Question>>(file)
                .unwrap()
                .into_iter()
                .collect();
        questions
    }

    /// Function to get a question from the questions hashmap
    pub async fn get_question(&self, id: &QuestionId) -> Option<Question> {
        self.questions.read().await.get(id).cloned()
    }

    /// Function to add a question to the questions hashmap
    pub async fn add_question(self, question: Question) -> Self {
        self.questions
            .write()
            .await
            .insert(question.id.clone(), question);
        self
    }

    /// Function to delete a question from the questions hashmap
    pub async fn delete_question(self, id: &QuestionId) -> Self {
        self.questions.write().await.remove(id);
        self
    }

    /// Function to update a question in the questions hashmap
    pub async fn update_question(self, id: &QuestionId, question: Question) -> Self {
        self.questions.write().await.insert(id.clone(), question);
        self
    }
}
