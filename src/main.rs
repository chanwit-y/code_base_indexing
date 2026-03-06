use std::{
    collections::HashMap,
    error::Error,
    fs,
    path::Path,
    time::{Instant, SystemTime, UNIX_EPOCH},
};

use async_openai::{
    config::OpenAIConfig,
    types::{
        chat::{
            ChatCompletionRequestMessage, ChatCompletionRequestUserMessageArgs,
            CreateChatCompletionRequestArgs,
        },
        embeddings::{CreateEmbeddingRequestArgs, Embedding},
    },
    Client,
};
use qdrant_client::{
    qdrant::{PointStruct, UpsertPointsBuilder, Value},
    Qdrant,
};
use serde::Serialize;
use uuid::Uuid;

mod command;
mod ocr;
fn run_import_code_bases() -> Result<(), Box<dyn Error>> {
    let base_path = "/Users/chanwit_y/Desktop/Projects/banpu/fingw-ui/src";
    command::import::run(base_path)?;

    Ok(())
}

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

async fn upsert_points(qdrant: &Qdrant, points: Vec<PointStruct>) -> Result<(), Box<dyn Error>> {
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

    Ok(())
}

// async fn chat(prompt: &str) -> Result<String, Box<dyn Error>> {

// }

fn write_file(path: &str, content: &str) -> Result<(), Box<dyn Error>> {
    let path = Path::new(path);
    fs::write(&path, content)?;

    Ok(())
}

fn format_unix_time(time: SystemTime) -> String {
    match time.duration_since(UNIX_EPOCH) {
        Ok(duration) => format!("{}.{}", duration.as_secs(), duration.subsec_millis()),
        Err(_) => "before_unix_epoch".to_string(),
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let started_at = SystemTime::now();
    let started_timer = Instant::now();
    println!("start_time_unix: {}", format_unix_time(started_at));

    //     dotenvy::dotenv().ok();
    //     let result  = command::import::load_and_sort_by_indent("/Users/chanwit_y/Desktop/Projects/poc/code_base_indexing/store/216c2f47-3961-4c58-b4aa-8111b6eb8fd0.json")?;

    //     let auth = std::env::var("OPENAI_API_KEY").expect("OPENAI_API_KEY must be set");
    //     let openai = Client::with_config(OpenAIConfig::new().with_api_key(auth.as_str()));

    //     let mut count = 0;
    //     for code_base in result {
    //         let is_not_internal = code_base.imports.iter().all(|i| i.is_external);
    //         // println!("imports: {:?}", code_base.imports);
    //         println!("is not internal: {}", is_not_internal);
    //         if is_not_internal {
    //             let path = Path::new(&code_base.path);
    //             let content = fs::read_to_string(path)?;

    //             let prompt = format!(
    //                 r#"You are an expert React TypeScript architect. I am providing you with a main React component file, along with the code (or summaries) of its imported dependencies.
    // Your task is to generate a highly compressed, information-dense summary of the MAIN component, integrating necessary context from its dependencies.

    // Please analyze the provided files and output a summary using the following structure:

    // 1. **Component Purpose:** 1-2 sentences explaining what this UI component does in the broader application.
    // 2. **Props & Types (`Interface`):** A concise list of the required/optional Props. Omit standard React.FC boilerplate.
    // 3. **State & Custom Hooks:** What local state it manages (`useState`, `useReducer`), and which crucial custom hooks it consumes (especially those from the provided dependencies).
    // 4. **Side Effects & Data Fetching:** Key `useEffect` triggers, API calls, or global state mutations (e.g., Redux dispatches, Context API updates).
    // 5. **Component Composition (JSX Skeleton):** A high-level bulleted list of the major child components it renders. Ignore styling (Tailwind/CSS) and basic HTML tags (div, span).

    // **CRITICAL CONSTRAINTS:**
    // - DO NOT output raw UI code, styling classes, or standard React boilerplate.
    // - Assume the reader is a Senior Frontend Engineer. Focus purely on data flow, component hierarchy, and business logic.
    // - Output the response in concise Markdown format. Optimize for the absolute minimum token count.

    // <dependencies>
    // </dependencies>

    // <main_file name="{}">
    // {}
    // </main_file>"#,
    //                 code_base.path, content
    //             );

    //             let request = CreateChatCompletionRequestArgs::default()
    //                 .model("gpt-5.1")
    //                 .messages(vec![ChatCompletionRequestMessage::User(
    //                     ChatCompletionRequestUserMessageArgs::default()
    //                         .content(content)
    //                         .build()?,
    //                 )])
    //                 .build()?;

    //             let response = openai.chat().create(request).await?;
    //             let content = response
    //                 .choices
    //                 .into_iter()
    //                 .next()
    //                 .and_then(|c| c.message.content)
    //                 .unwrap_or_default();

    //             println!("path: {}", code_base.path);
    //             // println!("{}", content);

    //             let name = code_base.path.split("/").last().unwrap().to_string();

    //             write_file(format!("store/{}.md", name).as_str(), &content)?;

    //             if count == 2 {
    //                 break;
    //             }

    //             count += 1;
    //         }
    //     }

    let path = Path::new(
        "/Users/chanwit_y/Desktop/Projects/poc/code_base_indexing/pdf/REC-168223-010000000019863.pdf",
    );
    let ocr_output = ocr::extract_pdf_ocr_text(path)?;

    let output_path = "tmp/ocr_output.txt";
    write_file(output_path, &ocr_output)?;
    println!("Saved OCR text to {output_path}");
    let ended_at = SystemTime::now();
    println!("end_time_unix: {}", format_unix_time(ended_at));
    println!("elapsed_ms: {}", started_timer.elapsed().as_millis());

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
