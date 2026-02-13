use core::fmt;
use regex::Regex;
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

// fn get_import_path(path_str: &str) -> String {}

enum ImportType {
    internal,
    external
}

struct ImportPath {
    source: String,
    imports: Vec<String>,
    import_type: ImportType
}

fn read_ts_file_content(path: &str) -> Result<String, Box<dyn Error>> {
    let p = Path::new(path);
    let ex = p
        .extension()
        .and_then(|s| s.to_str())
        .ok_or_else(|| format!("Path {} is not a file", path))?;

    if !p.is_file() || (ex != "ts" && ex != "tsx") {
        return Ok("".to_string());
    }

    Ok(fs::read_to_string(path)
        .expect("Failed to read file")
        .into())
}

fn get_import_path(content: &str) -> Result<Vec<String>, Box<dyn Error>> {
    let pattern = r#"import\s+(?:([\w*\s{},]+)\s+from\s+)?['"]([^'"]+)['"]"#;
    let re = Regex::new(pattern)?;

    println!("Searching for imports...");
    println!("------------------------");

    for cap in re.captures_iter(content) {
        println!("cap: {:?}", &cap);
        let imported_items = cap.get(1).map(|m| m.as_str().trim());
        println!("imported items: {:?}", imported_items);
        // let imported_items = cap.get(1).map_or("Default/None", |m| m.as_str().trim());
        // let path = &cap[2];

        // println!("Imported: {imported_items:?} from {path:?}");
        println!("------------------------");
    }

    Ok(vec![])
}

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
                "{}{}({}): {}",
                " ".repeat(indent),
                if path.is_dir() { "dir" } else { "file" },
                indent,
                path.file_name().unwrap().to_str().unwrap()
            );

            if path.is_dir() {
                let _ = deep_path(path.to_str().unwrap(), indent + 3, ignore_dirs);
            } else if path.is_file() {
                let content = read_ts_file_content(path.to_str().unwrap())?;
                get_import_path(content.as_str())?;
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
        "public".to_string(),
        "example".to_string(),
        "icons".to_string(),
        "env".to_string(),
        "assets".to_string(),
        "drizzle".to_string(),
        "env.local".to_string(),
        "env.development.local".to_string(),
        "env.test.local".to_string(),
        "env.production.local".to_string(),
        "env.development".to_string(),
        "env.test".to_string(),
        "env.production".to_string(),
    ];
    deep_path(
        "/Users/chanwit_y/Desktop/Projects/banpu/fingw-ui/src",
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
