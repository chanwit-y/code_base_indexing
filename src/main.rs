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
mod md;

fn run_import_code_bases() -> Result<(), Box<dyn Error>> {
    let base_path = "/Users/chanwit_y/Desktop/Projects/banpu/mybp-ui-v2/src";
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

async fn code_base_indexing(content: String) -> Result<String, Box<dyn Error>> {
    dotenvy::dotenv().ok();
    let auth = std::env::var("OPENAI_API_KEY").expect("OPENAI_API_KEY must be set");
    let openai = Client::with_config(OpenAIConfig::new().with_api_key(auth.as_str()));

    let prompt = format!(
        r#"# 
You are a frontend codebase analyzer. Your job is to read all provided React/TypeScript source files and produce a **structured UI component index** in markdown format. This index will be consumed by a second AI prompt to generate Playwright E2E tests.
## Instructions
Analyze every source file provided below and extract the following information. Be exhaustive — do not skip any interactive element.
## Required Output Format
Produce a single markdown document with these exact sections:
### 1. APP OVERVIEW
```
- Framework: (e.g., React + TypeScript)
- UI Library: (e.g., MUI / Material UI)
- State Management: (e.g., Zustand store, Redux, Context)
- Responsive Strategy: (e.g., useMediaQuery breakpoint at 900px)
- API Layer: (e.g., custom hooks like api.useGetApplications)
```
### 2. PAGE / VIEW STRUCTURE
For each top-level page or view, list:
```
#### [ComponentName]
- File: [relative path]
- Route/URL: [if identifiable, otherwise "N/A"]
- Viewport: [desktop | mobile | both]
- Children: [list of child components]
- Description: [1-2 sentence summary of what this view does]
```
### 3. INTERACTIVE ELEMENTS REGISTRY
This is the most critical section. For EVERY interactive element, create an entry:
```
#### Element: [human-readable name]
- Component: [React component name]
- File: [relative path]
- Type: [input | button | chip/tab | link | icon-button | select | checkbox | modal-trigger]
- HTML Element: [e.g., MUI TextField, MUI Chip, MUI Button, MUI IconButton]
- Selector Strategy (best to worst):
  1. id: [if present, e.g., #web-keyword]
  2. data-testid: [if present]
  3. role + name: [e.g., role="textbox" name="keyword"]
  4. CSS: [e.g., .MuiChip-root]
  5. text: [e.g., text="All"]
- Default Value: [if any]
- User Action: [type text | click | select | toggle]
- Triggers: [what happens on interaction — state change, API call, navigation, modal open, filter apply]
- Viewport: [desktop | mobile | both]
```
### 4. STATE & DATA FLOW
List every piece of state that affects the UI:
```
#### [stateName]
- Source: [store name or local state]
- Type: [string | string[] | array of objects | boolean]
- Set By: [which component/action sets it]
- Used By: [which components read it]
- UI Effect: [what changes visually when this state changes]
```
### 5. API ENDPOINTS
```
#### [hookName or endpoint]
- Method: [GET | POST | etc.]
- Parameters: [list params]
- Response Shape: [describe the data structure as inferred from usage]
- Used In: [component name]
- UI Effect: [what renders once data arrives]
```
### 6. RESPONSIVE BREAKPOINTS
```
| Breakpoint | Threshold | Desktop Behavior | Mobile Behavior |
|------------|-----------|-------------------|-----------------|
| md         | 900px     | ...               | ...             |
```
### 7. USER FLOWS (Critical Paths)
Identify the main user journeys through the UI:
```
#### Flow: [flow name]
- Viewport: [desktop | mobile]
- Steps:
  1. [action] → [expected result]
  2. [action] → [expected result]
  ...
- Preconditions: [e.g., API must return data]
- Components Involved: [list]
```
Focus on these common flows:
- Search/filter by keyword
- Filter by category (desktop chip click vs. mobile modal filter)
- Responsive layout switching
- App icon click/navigation (both desktop AND mobile)
- Empty state / no results (both desktop AND mobile)
- VPN-specific app indicators or badges
### 8. TESTABILITY NOTES
Flag any issues that affect E2E testing:
```
- Missing test IDs: [list elements without id or data-testid]
- Dynamic selectors: [elements with generated/index-based keys]
- Async dependencies: [API calls that must resolve before assertions]
- Modal/overlay patterns: [how modals are triggered and dismissed]
- Third-party components: [components from external libraries that may need special handling]
```
---
## Source Code Files
    {}
Format each file as:
```
### File: [relative/path/to/file.tsx]
\`\`\`tsx
[file contents]
\`\`\`
```"#,
        content
    );

    let request = CreateChatCompletionRequestArgs::default()
        .model("gpt-4o")
        .messages(vec![ChatCompletionRequestMessage::User(
            ChatCompletionRequestUserMessageArgs::default()
                .content(prompt)
                .build()?,
        )])
        .build()?;

    let response = openai.chat().create(request).await?;
    let content = response
        .choices
        .into_iter()
        .next()
        .and_then(|c| c.message.content)
        .unwrap_or_default();


    write_file(format!("store/application.md").as_str(), &content)?;

    Ok(content)
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // let paths = vec![
    //     "/Users/chanwit_y/Desktop/Projects/banpu/mybp-ui-v2/src/components/container/application/mobile/SearchMobile.tsx".to_string(),
    //     "/Users/chanwit_y/Desktop/Projects/banpu/mybp-ui-v2/src/components/container/application/mobile/Filter.tsx".to_string(),
    //     "/Users/chanwit_y/Desktop/Projects/banpu/mybp-ui-v2/src/components/container/application/mobile/index.tsx".to_string(),
    //     "/Users/chanwit_y/Desktop/Projects/banpu/mybp-ui-v2/src/components/container/application/desktop/Search.tsx".to_string(),
    //     "/Users/chanwit_y/Desktop/Projects/banpu/mybp-ui-v2/src/components/container/application/desktop/Category.tsx".to_string(),
    //     "/Users/chanwit_y/Desktop/Projects/banpu/mybp-ui-v2/src/components/container/application/desktop/index.tsx".to_string(),
    //     "/Users/chanwit_y/Desktop/Projects/banpu/mybp-ui-v2/src/components/container/application/List.tsx".to_string(),
    //     "/Users/chanwit_y/Desktop/Projects/banpu/mybp-ui-v2/src/components/container/application/index.tsx".to_string(),
    // ];

    // let mut content = String::new();
    // for path in paths {
    //     let c = fs::read_to_string(&path)?;
    //     content.push_str(c.as_str());
    // }
    // let res = code_base_indexing(content).await?;
    // println!("res: {}", res);
    // let res = fs::read_to_string("store/e2e-application.md")?;
    // let files = md::extract_files_map(&res);

    // for (path, code) in &files {
    //     let dest = Path::new("store/e2e").join(path);
    //     if let Some(parent) = dest.parent() {
    //         fs::create_dir_all(parent)?;
    //     }
    //     fs::write(&dest, code)?;
    //     println!("wrote: {}", dest.display());
    // }

    
    // code_base_indexing("/Users/chanwit_y/Desktop/Projects/banpu/mybp-ui-v2/src/components/container/application/index.tsx".to_string()).await?;

    // let started_at = SystemTime::now();
    // let started_timer = Instant::now();
    // println!("start_time_unix: {}", format_unix_time(started_at));

    // dotenvy::dotenv().ok();
    // let result  = command::import::load_and_sort_by_indent("/Users/chanwit_y/Desktop/Projects/poc/code_base_indexing/store/216c2f47-3961-4c58-b4aa-8111b6eb8fd0.json")?;

    // let auth = std::env::var("OPENAI_API_KEY").expect("OPENAI_API_KEY must be set");
    // let openai = Client::with_config(OpenAIConfig::new().with_api_key(auth.as_str()));

    // let mut count = 0;
    // for code_base in result {
    //     let is_not_internal = code_base.imports.iter().all(|i| i.is_external);
    //     // println!("imports: {:?}", code_base.imports);
    //     println!("is not internal: {}", is_not_internal);
    //     if is_not_internal {
    //         let path = Path::new(&code_base.path);
    //         let content = fs::read_to_string(path)?;

    //         let prompt = format!(
    //             r#"You are an expert React TypeScript architect. I am providing you with a main React component file, along with the code (or summaries) of its imported dependencies.
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
    //             code_base.path, content
    //         );

    //         let request = CreateChatCompletionRequestArgs::default()
    //             .model("gpt-5.1")
    //             .messages(vec![ChatCompletionRequestMessage::User(
    //                 ChatCompletionRequestUserMessageArgs::default()
    //                     .content(content)
    //                     .build()?,
    //             )])
    //             .build()?;

    //         let response = openai.chat().create(request).await?;
    //         let content = response
    //             .choices
    //             .into_iter()
    //             .next()
    //             .and_then(|c| c.message.content)
    //             .unwrap_or_default();

    //         println!("path: {}", code_base.path);
    //         // println!("{}", content);

    //         let name = code_base.path.split("/").last().unwrap().to_string();

    //         write_file(format!("store/{}.md", name).as_str(), &content)?;

    //         if count == 2 {
    //             break;
    //         }

    //         count += 1;
    //     }
    // }

    // ----------------------- DEMO OCR -----------------------
    let path = Path::new(
        "/Users/chanwit_y/Desktop/Projects/poc/code_base_indexing/pdf/REC-108708-010000000019860.pdf",
    );
    // let ocr_output = ocr::extract_pdf_ocr_text(path)?;
    ocr::call_pdf_page_qr_count(path)?;

    // let output_path = "tmp/ocr_output.txt";
    // write_file(output_path, &ocr_output)?;
    // println!("Saved OCR text to {output_path}");
    // let ended_at = SystemTime::now();
    // println!("end_time_unix: {}", format_unix_time(ended_at));
    // println!("elapsed_ms: {}", started_timer.elapsed().as_millis());

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
