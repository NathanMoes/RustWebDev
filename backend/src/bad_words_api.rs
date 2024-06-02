use crate::*;
use api::ApiError;
use reqwest_middleware::ClientBuilder;
use reqwest_retry::{policies::ExponentialBackoff, RetryTransientMiddleware};
use serde::{Deserialize, Serialize};
use std::env::var;

/// A struct representing an API response
#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct APIResponse {
    message: String,
}

/// A struct representing a bad word
#[derive(Deserialize, Serialize, Debug, Clone)]
struct BadWord {
    original: String,
    word: String,
    deviations: i64,
    info: i64,
    #[serde(rename = "replacedLen")]
    replaced_len: i64,
}

/// A response from the bad_words API
#[derive(Deserialize, Serialize, Debug, Clone)]
struct BadWordsResponse {
    content: String,
    bad_words_total: i64,
    bad_words_list: Vec<BadWord>,
    censored_content: String,
}

/// Runs a check for profanity on the given content using the bad_words API
/// # Arguments
/// * `content` - The content to check for profanity
/// # Returns
/// * `Ok(String)` - The censored content
/// * `Err(ApiError)` - The error that occurred
pub async fn check_profanity(content: String) -> Result<String, ApiError> {
    let bad_word_api_key = match var("API_LAYER_KEY") {
        Ok(key) => key,
        Err(_) => return Err(ApiError::MissingParameters),
    };
    let retry_policy = ExponentialBackoff::builder().build_with_max_retries(3);
    let client = ClientBuilder::new(reqwest::Client::new())
        // Trace HTTP requests. See the tracing crate to make use of these traces.
        // Retry failed requests.
        .with(RetryTransientMiddleware::new_with_policy(retry_policy))
        .build();

    let res = client
        .post("https://api.apilayer.com/bad_words?censor_character=*")
        .header("apikey", bad_word_api_key)
        .header("Content-Length", content.len().to_string())
        .body(content)
        .send()
        .await
        .map_err(ApiError::MiddlewareReqwestAPIError)?;

    if !res.status().is_success() {
        if res.status().is_client_error() {
            return Err(ApiError::ClientError(res.error_for_status().unwrap_err()));
        } else {
            return Err(ApiError::ReqwestAPIError(
                res.error_for_status().unwrap_err(),
            ));
        }
    }

    match res.json::<BadWordsResponse>().await {
        Ok(res) => Ok(res.censored_content),
        Err(e) => Err(ApiError::ReqwestAPIError(e)),
    }
}
