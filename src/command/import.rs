use regex::Regex;
use serde::Serialize;
use std::{
    error::Error,
    fs::{self, File},
    io::BufWriter,
    path::Path,
};
use uuid::Uuid;
#[derive(Debug, Serialize)]
struct Import {
    items: Vec<String>,
    from: String,
    from_path: String,
    is_external: bool,
    imports: Vec<Import>,
}
#[derive(Debug, Serialize)]
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

fn get_file_extension(folders: &Vec<&str>) -> Result<String, Box<dyn Error>> {
    let path = folders
        .iter()
        .take(folders.len() - 1)
        .map(|x| x.to_string())
        .collect::<Vec<String>>()
        .join("/");

    let path = Path::new(path.as_str());
    let file_names = fs::read_dir(path)?
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

    let name = folders.last().unwrap().to_string();
    let mut result: &str = "";
    // println!("list name: {:?}", file_names);
    file_names.iter().for_each(|f| {
        if f.starts_with(&name) {
            result = f;
        }
    });

    Ok(result.to_string())
}

fn get_import_path(content: &str, path: &str) -> Result<Vec<Import>, Box<dyn Error>> {
    let pattern = r#"import\s+(?:([\w*\s{},]+)\s+from\s+)?['"]([^'"]+)['"]"#;
    let re = Regex::new(pattern)?;

    let mut result: Vec<Import> = Vec::new();

    for cap in re.captures_iter(content) {
        if cap.get(0).is_none()
            || cap.get(0).unwrap().as_str().trim().is_empty()
            || cap.get(0).unwrap().as_str().trim().starts_with("//")
        {
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
        let mut from_path: String = String::from("");
        if is_back {
            let count_back = from
                .split("/")
                .map(|x| x.to_string())
                .filter(|x| x == ".." || x == "./")
                .count();

            folders = pwd
                .split("/")
                .to_owned()
                .take(pwd.split("/").to_owned().count() - count_back)
                .collect::<Vec<&str>>();

            from.split('/').skip(count_back).for_each(|x| {
                folders.push(x);
            });

            let import_file_name = get_file_extension(&folders)?;
            if import_file_name == "" {
                folders.push("index.ts");
            } else {
                folders.pop();
                folders.push(import_file_name.as_str());
            }

            from_path = folders.join("/");
        } else if is_current {
            let fp = format!("{}/{}", pwd, from.replace("./", "").as_str());
            let import_file_name = get_file_extension(&fp.split("/").collect::<Vec<&str>>())
                .unwrap_or("index.ts".to_string());
            folders = fp.split("/").collect::<Vec<&str>>();
            if import_file_name == "" {
                folders.push("index.ts");
            } else {
                folders.pop();
                folders.push(import_file_name.as_str());
            }

            from_path = folders.join("/");
        }

        let is_external = from_path == "";
        result.push(Import {
            items: imported_items,
            from: from,
            from_path: from_path,
            is_external: is_external,
            imports: Vec::new(),
        });
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

fn deep_import(
    code_bases: &Vec<CodeBase>,
    imports: &Vec<Import>,
    base_path: &str,
) -> Result<Vec<Import>, Box<dyn Error>> {
    let mut visited: Vec<String> = Vec::new();
    deep_import_recursive(code_bases, imports, &mut visited, base_path)
}

fn deep_import_recursive(
    code_bases: &Vec<CodeBase>,
    imports: &Vec<Import>,
    visited: &mut Vec<String>,
    base_path: &str,
) -> Result<Vec<Import>, Box<dyn Error>> {
    let mut result: Vec<Import> = Vec::new();

    for import in imports {
        if import.is_external || import.from_path.is_empty() {
            result.push(Import {
                items: import.items.clone(),
                from: import.from.clone(),
                from_path: import.from_path.clone(),
                is_external: import.is_external,
                imports: Vec::new(),
            });
            continue;
        }

        if !import.from_path.starts_with(base_path) {
            result.push(Import {
                items: import.items.clone(),
                from: import.from.clone(),
                from_path: import.from_path.clone(),
                is_external: import.is_external,
                imports: Vec::new(),
            });
            continue;
        }

        if visited.contains(&import.from_path) {
            result.push(Import {
                items: import.items.clone(),
                from: import.from.clone(),
                from_path: import.from_path.clone(),
                is_external: import.is_external,
                imports: Vec::new(),
            });
            continue;
        }

        visited.push(import.from_path.clone());

        let sub_imports = code_bases
            .iter()
            .find(|cb| cb.path == import.from_path)
            .map(|cb| deep_import_recursive(code_bases, &cb.imports, visited, base_path))
            .unwrap_or(Ok(Vec::new()))?;

        result.push(Import {
            items: import.items.clone(),
            from: import.from.clone(),
            from_path: import.from_path.clone(),
            is_external: import.is_external,
            imports: sub_imports,
        });
    }

    Ok(result)
}

fn generate_uuid() -> String {
    Uuid::new_v4().to_string()
}

fn write_json(code_bases: &Vec<CodeBase>) -> Result<String, Box<dyn Error>> {
    let name = generate_uuid();
    let path = format!("../store/{}.json", name);
    let path = Path::new(&path);
    if !path.parent().unwrap().exists() {
        fs::create_dir_all(path.parent().unwrap())?;
    }
    let file = File::create(&path);
    match file {
        Ok(file) => {
            let writer = BufWriter::new(file);

            serde_json::to_writer_pretty(writer, &code_bases)?;

            Ok(name)
        }
        Err(e) => {
            println!("Failed to create file: {}", e);
            Err(e.into())
        }
    }
}

pub fn run(src_path: &str) -> Result<String, Box<dyn Error>> {
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
    let code_bases = deep_path(src_path, 0, &ignore_dirs)?;

    let code_bases: Vec<CodeBase> = code_bases
        .iter()
        .map(|cb| {
            let imports = deep_import(&code_bases, &cb.imports, src_path).unwrap_or_default();
            CodeBase {
                indent: cb.indent,
                path: cb.path.clone(),
                imports,
            }
        })
        .collect();
    println!("create json file");
    let path = write_json(&code_bases)?;
    Ok(path)
}
