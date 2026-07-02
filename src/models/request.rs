use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct CodeRequest {
    pub access_code: String,
}