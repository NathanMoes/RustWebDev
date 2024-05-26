use chrono::{DateTime, Utc};

use crate::{
    auth::{make_jwt_keys, JwtKeys},
    *,
};
use std::collections::HashSet;

/// An account struct to represent an account in the database
#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, ToSchema, Clone)]
pub struct Account {
    #[schema(example = "1")]
    pub id: AccountId,
    #[schema(example = "moes@pdx.edu")]
    pub email: String,
    #[schema(example = "someHashOfAPassword")]
    pub password: String,
}

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq, Hash, sqlx::Type)]
pub struct AccountId(pub i32);

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Session {
    pub exp: DateTime<Utc>,
    pub account_id: AccountId,
    pub nbf: DateTime<Utc>,
}

/// An answer struct to represent an answer in the database
#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, ToSchema)]
pub struct Answer {
    #[schema(example = "This is an answer to the question")]
    pub content: String,
    #[schema(example = "1")]
    pub question_id: QuestionId,
}

/// Application state struct
/// This struct is used to hold the state of the application, which is currently only the questions for the API
#[derive(Clone, Debug)]
pub struct AppState(pub PgPool, pub JwtKeys);

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

        let port = var("PG_PORT")
            .map(|val| val.parse().expect("PG_PORT should be a valid u16"))
            .unwrap_or(6565);
        let password = var("PG_PASSWORD")?;
        let url = format!(
            "postgres://{}:{}@{}:{}",
            var("PG_USER")?,
            password.trim(),
            var("PG_HOST")?,
            port
        );
        let pool = PgPool::connect(&url).await?;
        sqlx::migrate!().run(&pool).await?;
        let keys = make_jwt_keys().await?;
        Ok(AppState(pool, keys))
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

    pub async fn add_answer(self, answer: Answer) -> Result<(), Box<dyn Error>> {
        let tx = Pool::begin(&self.0).await?;
        sqlx::query(r#"INSERT INTO answers (corresponding_question, content) VALUES ($1, $2);"#)
            .bind(answer.question_id.0)
            .bind(answer.content)
            .execute(&self.0)
            .await?;
        Ok(tx.commit().await?)
    }

    pub async fn get_answers(
        &self,
        question_id: &QuestionId,
    ) -> Result<Vec<Answer>, Box<dyn Error>> {
        let mut answers = Vec::new();
        let rows = sqlx::query(r#"SELECT * FROM answers WHERE corresponding_question = $1;"#)
            .bind(question_id.0)
            .fetch_all(&self.0)
            .await?;
        for row in rows {
            answers.push(Answer {
                content: row.get("content"),
                question_id: QuestionId(row.get("corresponding_question")),
            });
        }
        Ok(answers)
    }

    pub async fn delete_answer(self, question_id: &QuestionId) -> Result<(), Box<dyn Error>> {
        let tx = Pool::begin(&self.0).await?;
        sqlx::query(r#"DELETE FROM answers WHERE corresponding_question = $1;"#)
            .bind(question_id.0)
            .execute(&self.0)
            .await?;
        Ok(tx.commit().await?)
    }

    pub async fn update_answer(
        self,
        question_id: &QuestionId,
        answer: Answer,
    ) -> Result<(), Box<dyn Error>> {
        let tx = Pool::begin(&self.0).await?;
        sqlx::query(r#"UPDATE answers SET content = $1 WHERE corresponding_question = $2;"#)
            .bind(answer.content)
            .bind(question_id.0)
            .execute(&self.0)
            .await?;
        Ok(tx.commit().await?)
    }

    pub async fn add_account(self, acc: Account) -> Result<(), Box<dyn Error>> {
        let tx = Pool::begin(&self.0).await?;
        sqlx::query(r#"INSERT INTO accounts (email, password) VALUES ($1, $2);"#)
            .bind(acc.email)
            .bind(acc.password)
            .execute(&self.0)
            .await?;
        Ok(tx.commit().await?)
    }

    pub async fn get_account(&self, email: &str) -> Result<Option<Account>, Box<dyn Error>> {
        let row = sqlx::query(r#"SELECT * from accounts WHERE email = $1;"#)
            .bind(email)
            .fetch_one(&self.0)
            .await?;

        let email = match row.try_get("username")? {
            Some(email) => email,
            None => return Ok(None),
        };
        let password = row.try_get("password")?;
        let account_id = row.try_get("id")?;
        if let Some(id) = account_id {
            Ok(Some(Account {
                id,
                email,
                password,
            }))
        } else {
            Ok(None)
        }
    }

    pub async fn delete_account(self, email: &str) -> Result<(), Box<dyn Error>> {
        let tx = Pool::begin(&self.0).await?;
        sqlx::query(r#"DELETE FROM accounts WHERE email = $1;"#)
            .bind(email)
            .execute(&self.0)
            .await?;
        Ok(tx.commit().await?)
    }

    pub async fn update_account(self, email: &str, acc: Account) -> Result<(), Box<dyn Error>> {
        let tx = Pool::begin(&self.0).await?;
        sqlx::query(r#"UPDATE accounts SET email = $1, password = $2 WHERE email = $3;"#)
            .bind(acc.email)
            .bind(acc.password)
            .bind(email)
            .execute(&self.0)
            .await?;
        Ok(tx.commit().await?)
    }
}
