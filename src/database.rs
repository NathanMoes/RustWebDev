use std::collections::HashSet;

use crate::*;

/// Application state struct
/// This struct is used to hold the state of the application, which is currently only the questions for the API
#[derive(Clone, Debug)]
pub struct AppState(pub PgPool);

/// Implementing the AppState struct with basic functions to use for API and state management operations
impl AppState {
    /// Function to create a new AppState
    /// This function creates a new AppState by connecting to the database and running the migrations
    /// #Example:
    /// ```
    /// let state = AppState::new().await.unwrap();
    /// ```
    /// This function returns a Result with the AppState or an error
    /// #Errors:
    /// This function can return an error if the database connection fails or the migrations fail
    /// #Panics:
    /// This function will panic if the environment variables are not set
    /// #Notes:
    /// This function is used to create the AppState for the API
    pub async fn new() -> Result<Self, Box<dyn Error>> {
        use std::env::var;

        let password = var("PG_PASSWORD")?;
        let url = format!(
            "postgres://{}:{}@{}:5432/{}",
            var("PG_USER")?,
            password.trim(),
            var("PG_HOST")?,
            var("PG_DBNAME")?,
        );
        let pool = PgPool::connect(&url).await?;
        sqlx::migrate!().run(&pool).await?;
        Ok(AppState(pool))
    }

    /// Function to get a question from the questions database, by id
    pub async fn get_question(&self, id: &QuestionId) -> Result<Option<Question>, Box<dyn Error>> {
        let row = sqlx::query(r#"SELECT * FROM questions WHERE id = $1;"#)
            .bind(id.0)
            .fetch_one(&self.0)
            .await?;

        let tags: Option<Vec<String>> = row.try_get("tags")?;
        let tags = tags.map(|tags| tags.into_iter().collect::<HashSet<String>>());

        Ok(Some(Question {
            id: QuestionId(row.get(0)),
            title: row.get(1),
            content: row.get(2),
            tags,
        }))
    }

    /// Function to get all questions from the database
    pub async fn get_all_questions(&self) -> Result<Vec<Question>, Box<dyn Error>> {
        let mut questions = Vec::new();
        let rows = sqlx::query(r#"SELECT * FROM questions;"#)
            .fetch_all(&self.0)
            .await?;
        for row in rows {
            let tags: Option<Vec<String>> = row.try_get("tags")?;
            let tags = tags.map(|tags| tags.into_iter().collect::<HashSet<String>>());
            questions.push(Question {
                id: QuestionId(row.get(0)),
                title: row.get(1),
                content: row.get(2),
                tags,
            });
        }
        Ok(questions)
    }

    /// Function to add a question to the questions database
    pub async fn add_question(self, question: Question) -> Result<(), Box<dyn Error>> {
        let tx = Pool::begin(&self.0).await?;
        let tags = question
            .tags
            .map(|tags| tags.into_iter().collect::<Vec<String>>());
        sqlx::query(r#"INSERT INTO questions (title, content, tags) VALUES ($1, $2, $3);"#)
            .bind(question.title)
            .bind(question.content)
            .bind(&tags)
            .execute(&self.0)
            .await?;

        Ok(tx.commit().await?)
    }

    /// Function to delete a question from the questions database
    pub async fn delete_question(self, id: &QuestionId) -> Result<(), Box<dyn Error>> {
        let tx = Pool::begin(&self.0).await?;
        sqlx::query(r#"DELETE FROM questions WHERE id = $1;"#)
            .bind(id.0)
            .execute(&self.0)
            .await?;
        Ok(tx.commit().await?)
    }

    /// Function to update a question in the questions database
    pub async fn update_question(
        self,
        id: &QuestionId,
        question: Question,
    ) -> Result<(), Box<dyn Error>> {
        let tx = Pool::begin(&self.0).await?;
        let tags = question
            .tags
            .map(|tags| tags.into_iter().collect::<Vec<String>>());
        sqlx::query(r#"UPDATE questions SET title = $1, content = $2, tags = $3 WHERE id = $4;"#)
            .bind(question.title)
            .bind(question.content)
            .bind(tags)
            .bind(id.0)
            .execute(&self.0)
            .await?;
        Ok(tx.commit().await?)
    }
}
