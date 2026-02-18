use core::fmt;
use regex::Regex;
use std::{
    error::Error,
    fmt::{Display, Formatter},
    fs,
    ops::Index,
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
#[derive(Debug)]
struct Import {
    items: Vec<String>,
    from: String,
}

#[derive(Debug)]
struct CodeBase {
    indent: usize,
    path: String,
    imports: Vec<Import>,
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

fn get_import_path(content: &str, path: &str) -> Result<Vec<Import>, Box<dyn Error>> {
    let pattern = r#"import\s+(?:([\w*\s{},]+)\s+from\s+)?['"]([^'"]+)['"]"#;
    let re = Regex::new(pattern)?;

    let mut result: Vec<Import> = Vec::new();

    for cap in re.captures_iter(content) {

        if cap.get(0).is_none() || cap.get(0).unwrap().as_str().trim().is_empty() || cap.get(0).unwrap().as_str().trim().starts_with("//") {
            continue;
        }

        let imported_items = cap
            .get(1)
            .map_or("", |m| m.as_str().trim())
            .trim_matches(|c| {
                c == '{' || c == '}' || c == ' ' || c == '\n' || c == '\t' || c == '\r'
            })
            .split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect::<Vec<String>>();

        let from = cap.get(2).map(|m| m.as_str().trim().to_string()).unwrap();

        let file_name = Path::new(path)
            .file_name()
            .and_then(|x| x.to_str())
            .unwrap();
        let pwd = path.replace(&format!("/{}", file_name), "");

        let pattern_back = r#"^../"#;
        let pattern_current = r#"^./"#;
        let re_back = Regex::new(pattern_back)?;
        let re_current = Regex::new(pattern_current)?;

        let is_back = re_back.is_match(&from);
        let is_current = re_current.is_match(&from);

        let mut folders: Vec<&str> = Vec::new();
        if is_back {

            let count_back = from
                .split("/")
                .map(|x| x.to_string())
                .filter(|x| x == ".." || x == "./")
                .count();

            let pwd_list = pwd.split("/");

            folders = pwd_list
                .to_owned()
                .take(pwd_list.count() - count_back)
                .collect::<Vec<&str>>();


            from.split('/').skip(count_back).for_each(|x| {
                folders.push(x);
            });

            let d = folders
                .iter()
                .take(folders.len() - 1)
                .map(|x| x.to_string())
                .collect::<Vec<String>>()
                .join("/");
            let y = Path::new(d.as_str());
            let z = fs::read_dir(y)?
                .filter_map(|f| f.ok().map(|x| x.path()))
                .filter(|x| {
                    x.is_file() && x.extension().and_then(|x| x.to_str()) == Some("ts")
                        || x.extension().and_then(|x| x.to_str()) == Some("tsx")
                })
                .filter_map(|x| {
                    x.file_name()
                        .and_then(|x| x.to_str())
                        .map(|x| x.to_string())
                })
                .collect::<Vec<String>>();


            println!("z: {:?}", z);

        } else if is_current {
            // folders = pwd.split("/").collect::<Vec<&str>>();
            // for f in from.split("/") {

            // }
        }

        // println!("from: {}", from);
        // println!("folders: {:?}", folders);
        // println!("is_back: {:?}", is_back);
        // println!("is_current: {:?}", is_current);

        // println!("--> imported items: {:?}", imported_items);
        // println!("--> imported path: {:?}", from);

        result.push(Import {
            items: imported_items,
            from: from,
        });
        // let imported_items = cap.get(1).map_or("Default/None", |m| m.as_str().trim());
        // let path = &cap[2];

        // println!("Imported: {imported_items:?} from {path:?}");
        // println!("------------------------");
    }

    Ok(result)
}

fn deep_path(
    path_str: &str,
    indent: usize,
    ignore_dirs: &Vec<String>,
) -> Result<Vec<CodeBase>, Box<dyn Error>> {
    let mut result: Vec<CodeBase> = Vec::new();
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

            // println!(
            // //     "{}{}({}): {}",
            // //     " ".repeat(indent),
            // //     if path.is_dir() { "dir" } else { "file" },
            // //     indent,
            // //     path.file_name().unwrap().to_str().unwrap()
            // // );

            if path.is_dir() {
                let r = deep_path(path.to_str().unwrap(), indent + 3, ignore_dirs);
                result.extend(r?);
            } else if path.is_file() {
                let content = read_ts_file_content(path.to_str().unwrap())?;
                let imports = get_import_path(content.as_str(), path.to_str().unwrap())?;

                result.push(CodeBase {
                    path: path.to_str().unwrap().to_string(),
                    indent: indent,
                    imports: imports,
                });
            }
        }
    }

    Ok(result)
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
    let code_bases = deep_path(
        "/Users/chanwit_y/Desktop/Projects/banpu/fingw-ui/src",
        0,
        &ignore_dirs,
    )?;

    // for c in code_bases {
    //     println!("------------------------");
    //     println!("path: {}", c.path);
    //     println!("imports: {:#?}", c.imports);
    //     println!("indent: {}", c.indent);
    //     println!("------------------------");
    // }

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
