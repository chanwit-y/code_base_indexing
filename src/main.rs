use std::error::Error;

use async_openai::{
    Client, config::OpenAIConfig, types::embeddings::CreateEmbeddingRequestArgs
};

mod command;

fn run_import_code_bases() -> Result<(), Box<dyn Error>> {
    let base_path = "/Users/chanwit_y/Desktop/Projects/banpu/fingw-ui/src";
    command::import::run(base_path)?;

    Ok(())
}

async fn embed_texts(openai: &Client<OpenAIConfig>) -> Result<Vec<Vec<f32>>, Box<dyn Error>> {
    let request = CreateEmbeddingRequestArgs::default()
        .model("text-embedding-3-small")
        .input("Hello, world!")
        .build()?;

    println!("request: {:#?}", request);

    // let response = openai.embeddings().create(request).await?;

    Ok(vec![vec![0.0, 0.0, 0.0]])
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    dotenvy::dotenv().ok();


    let http_client = reqwest::Client::builder()
        .danger_accept_invalid_certs(true)
        .build()?;

    let openai = Client::with_config(
        OpenAIConfig::new().with_api_base("https://api.openai.com/v1"),
    )
    .with_http_client(http_client);

    embed_texts(&openai).await?;

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
