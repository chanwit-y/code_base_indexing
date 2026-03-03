use std::{collections::HashMap, error::Error};

use async_openai::{
    config::OpenAIConfig,
    types::embeddings::{CreateEmbeddingRequestArgs, Embedding},
    Client,
};
use qdrant_client::{
    Qdrant, qdrant::{PointStruct, UpsertPointsBuilder, Value}
};
use serde::Serialize;
use uuid::Uuid;

// mod command;
// fn run_import_code_bases() -> Result<(), Box<dyn Error>> {
//     let base_path = "/Users/chanwit_y/Desktop/Projects/banpu/fingw-ui/src";
//     command::import::run(base_path)?;

//     Ok(())
// }

async fn embed_texts(
    openai: &Client<OpenAIConfig>,
    input: &Vec<String>,
) -> Result<Vec<Embedding>, Box<dyn Error>> {
    let request = CreateEmbeddingRequestArgs::default()
        .model("text-embedding-3-small")
        .input(input)
        .build()?;

    let response = openai.embeddings().create(request).await?;

    Ok(response.data)
}

#[derive(Serialize)]
struct StoredPoint {
    id: String,
    name: String,
    embedding: Vec<f32>,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    dotenvy::dotenv().ok();

    let http_client = reqwest::Client::builder()
        .danger_accept_invalid_certs(true)
        .build()?;

    let openai =
        Client::with_config(OpenAIConfig::new().with_api_base("https://api.openai.com/v1"))
            .with_http_client(http_client);

    let items = vec!["Test".to_string(), "A".to_string()];
    let embed = embed_texts(&openai, &items).await?;

    let points: Vec<PointStruct> = items
        .iter()
        .zip(embed.iter())
        .map(|(item, vectors)| {
            PointStruct::new(
                Uuid::new_v4().to_string(),
                vectors.embedding.clone(),
                HashMap::from([("name".to_string(), Value::from(item.clone()))]),
            )
        })
        .collect();

    // qdrant-client uses gRPC; connect to the gRPC port (6334), not REST (6333).
    let qdrant = Qdrant::from_url("http://localhost:6334").build()?;
    qdrant
        .upsert_points(UpsertPointsBuilder::new("synapse", points))
        .await?;

    // let client = Qdrant::from_url("http://localhost:6334").build();

    // let request = CreateChatCompletionRequestArgs::default()
    //     .model("gpt-3.5-turbo")
    //     .messages(vec![ChatCompletionRequestUserMessage::from("Hello, how are you?").into()])
    //     .build()?;

    // let response = client.chat().create(request).await?;

    // if let Some(choice) = response.choices.first() {
    //     if let Some(ref msg) = choice.message.content {
    //         println!("message: {msg}");
    //     }
    // }

    Ok(())
}

// fn chat() -> Result<Completion, Error> {
//     let auth = std::env::var("OPENAI_API_KEY").expect("OPENAI_API_KEY must be set");
//     let openai = OpenAI::new(Auth::new(auth.as_str()), "https://api.openai.com/v1");
//     let body = ChatBody {
//         model: "gpt-3.5-turbo".to_string(),
//         messages: vec![Message {
//             role: Role::User,
//             content: "Hello, how are you?".to_string(),
//         }],
//         temperature: Some(0.5),
//         top_p: Some(1.0),
//         n: Some(1),
//         stream: Some(false),
//         stop: None,
//         max_tokens: None,
//         presence_penalty: None,
//         frequency_penalty: None,
//         logit_bias: None,
//         user: None,
//     };
//     // let rs = openai.chat_completion_create(&body);
//     // rs
//     let rs = openai.chat_completion_create(&body);
//     rs
// }
