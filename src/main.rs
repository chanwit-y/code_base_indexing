use core::fmt;
use std::{
    error::Error,
    fmt::{Display, Formatter},
    fs,
    path::Path,
};

use async_openai::{
    config::OpenAIConfig,
    types::chat::{ChatCompletionRequestUserMessage, CreateChatCompletionRequestArgs},
    Client,
};

// enum FileType {
//     File,
//     Directory,
// }

// impl Display for FileType {
//     fn fmt(&self, f: &mut Formatter) -> fmt::Result {
//         match self {
//             FileType::File => write!(f, "file"),
//             FileType::Directory => write!(f, "directory"),
//         }
//     }
// }

fn deep_path(
    path_str: &str,
    indent: usize,
    ignore_dirs: &Vec<String>,
) -> Result<(), Box<dyn Error>> {
    let path = Path::new(path_str);

    if !path.exists() {
        return Err(format!("Path {} does not exist", path_str).into());
    }

    if path.is_dir() {
        let files = fs::read_dir(path)?;
        for file in files {
            let file = file?;
            let path = file.path();

            if ignore_dirs.contains(&path.file_name().unwrap().to_str().unwrap().to_string()) {
                continue;
            }
            println!(
                "{}{}: {}",
                " ".repeat(indent),
                if path.is_dir() { "dir" } else { "file" },
                path.file_name().unwrap().to_str().unwrap()
            );

            if path.is_dir() {
                let _ = deep_path(path.to_str().unwrap(), indent + 3, ignore_dirs);
            }
        }
    }

    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let ignore_dirs = vec![
        "node_modules".to_string(),
        ".git".to_string(),
        ".next".to_string(),
        "dist".to_string(),
        "build".to_string(),
        "target".to_string(),
        "venv".to_string(),
        "env".to_string(),
        "env.local".to_string(),
        "env.development.local".to_string(),
        "env.test.local".to_string(),
        "env.production.local".to_string(),
        "env.development".to_string(),
        "env.test".to_string(),
        "env.production".to_string(),
    ];
    deep_path(
        "/Users/chanwit_y/Desktop/Projects/synapse-o",
        0,
        &ignore_dirs,
    )?;

    // dotenvy::dotenv().ok();

    // let http_client = reqwest::Client::builder()
    //     .danger_accept_invalid_certs(true)
    //     .build()?;

    // let client = Client::with_config(
    //     OpenAIConfig::new().with_api_base("https://api.openai.com/v1"),
    // )
    // .with_http_client(http_client);

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
