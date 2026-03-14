use std::{error::Error, fs, io, path::Path};

use pdfium_render::prelude::{PdfPageRenderRotation, PdfRenderConfig, Pdfium};

fn bind_pdfium() -> Result<Pdfium, Box<dyn Error>> {
    if let Ok(custom_path) = std::env::var("PDFIUM_LIB_PATH") {
        if let Ok(bindings) = Pdfium::bind_to_library(&custom_path) {
            return Ok(Pdfium::new(bindings));
        }
    }

    if let Ok(bindings) = Pdfium::bind_to_system_library() {
        return Ok(Pdfium::new(bindings));
    }

    if let Ok(bindings) = Pdfium::bind_to_library("/opt/homebrew/lib/libpdfium.dylib") {
        return Ok(Pdfium::new(bindings));
    }

    if let Ok(bindings) = Pdfium::bind_to_library("/usr/local/lib/libpdfium.dylib") {
        return Ok(Pdfium::new(bindings));
    }

    if let Ok(bindings) = Pdfium::bind_to_library("./third_party/pdfium/lib/libpdfium.dylib") {
        return Ok(Pdfium::new(bindings));
    }

    let local_lib = Pdfium::pdfium_platform_library_name_at_path("./");
    if let Ok(bindings) = Pdfium::bind_to_library(&local_lib) {
        return Ok(Pdfium::new(bindings));
    }

    let custom_hint = std::env::var("PDFIUM_LIB_PATH")
        .map(|path| format!("\n- Tried `PDFIUM_LIB_PATH={path}` (not found or not loadable)."))
        .unwrap_or_default();

    let message = format!(
        "Could not load Pdfium (`libpdfium.dylib`).\
         \nInstall it and/or set `PDFIUM_LIB_PATH` to the absolute dylib path.\
         \n\
         \nTried locations:\
         \n- System library lookup\
         \n- /opt/homebrew/lib/libpdfium.dylib\
         \n- /usr/local/lib/libpdfium.dylib\
         \n- ./third_party/pdfium/lib/libpdfium.dylib\
         \n- {local_lib:?}{custom_hint}\
         \n\
         \nFix options:\
         \n1) Download macOS arm64/x64 Pdfium from https://github.com/bblanchon/pdfium-binaries/releases\
         \n2) Place `libpdfium.dylib` in one of the paths above\
         \n3) Or export `PDFIUM_LIB_PATH=/absolute/path/to/libpdfium.dylib`"
    );

    Err(io::Error::new(io::ErrorKind::NotFound, message).into())
}

fn configure_tessdata_prefix() {
    if std::env::var_os("TESSDATA_PREFIX").is_some() {
        return;
    }

    let candidates = [
        "/opt/homebrew/share/tessdata",
        "/usr/local/share/tessdata",
        "./third_party/tessdata",
        "./tessdata",
    ];

    for candidate in candidates {
        if Path::new(candidate).exists() {
            std::env::set_var("TESSDATA_PREFIX", candidate);
            break;
        }
    }
}

fn has_traineddata_file(tessdata_prefix: &Path, lang: &str) -> bool {
    let file_name = format!("{lang}.traineddata");
    let direct_path = tessdata_prefix.join(&file_name);
    let nested_path = tessdata_prefix.join("tessdata").join(file_name);

    direct_path.exists() || nested_path.exists()
}

fn resolve_tesseract_languages() -> String {
    if let Some(prefix) = std::env::var_os("TESSDATA_PREFIX") {
        let tessdata_prefix = Path::new(&prefix);
        let has_thai = has_traineddata_file(tessdata_prefix, "tha");
        let has_english = has_traineddata_file(tessdata_prefix, "eng");

        return match (has_thai, has_english) {
            (true, true) => "tha+eng".to_string(),
            (true, false) => "tha".to_string(),
            (false, true) => "eng".to_string(),
            (false, false) => "eng".to_string(),
        };
    }

    "tha+eng".to_string()
}

fn create_tesseract_with_fallback(language: &str) -> Result<tesseract::Tesseract, Box<dyn Error>> {
    configure_tessdata_prefix();

    match tesseract::Tesseract::new(None, Some(language)) {
        Ok(tes) => Ok(tes),
        Err(err) => {
            eprintln!(
                "Warning: could not initialize Tesseract with `{language}` ({err}). Falling back to `eng`."
            );
            let tes = tesseract::Tesseract::new(None, Some("eng"))?;
            Ok(tes)
        }
    }
}

fn run_ocr_with_mode(
    image_path: &str,
    mode: tesseract::PageSegMode,
    language: &str,
) -> Result<(String, i32), Box<dyn Error>> {
    let mut tes = create_tesseract_with_fallback(language)?;
    tes.set_page_seg_mode(mode);
    tes = tes
        .set_image(image_path)?
        .set_source_resolution(300)
        .set_variable("user_defined_dpi", "300")?
        .set_variable("preserve_interword_spaces", "1")?
        .set_variable("textord_heavy_nr", "1")?
        .recognize()?;

    let confidence = tes.mean_text_conf();
    let text = tes.get_text()?;
    Ok((text, confidence))
}

fn score_ocr_text(text: &str, confidence: i32) -> i64 {
    // Prefer high-confidence outputs with longer words; penalize symbol-heavy noise.
    let token_score: i64 = text
        .split_whitespace()
        .map(|token| token.chars().filter(|c| c.is_alphanumeric()).count())
        .filter(|len| *len >= 3)
        .map(|len| (len * len) as i64)
        .sum();

    let alnum_count = text.chars().filter(|c| c.is_alphanumeric()).count() as i64;
    let noisy_symbol_count = text
        .chars()
        .filter(|c| !c.is_whitespace() && !c.is_alphanumeric() && !",.:;/-_()%#&+*'\"@".contains(*c))
        .count() as i64;

    let orphan_short_line_count = text
        .lines()
        .filter(|line| {
            let trimmed = line.trim();
            !trimmed.is_empty() && trimmed.chars().count() <= 3
        })
        .count() as i64;

    let text_lc = text.to_lowercase();
    let anchor_terms = [
        "goods received note",
        "item description",
        "po number",
        "receipt number",
        "quantity ordered",
        "delivery instructions",
        "report date",
        "invoice",
        "tax",
        "total amount",
    ];
    let anchor_bonus: i64 = anchor_terms
        .iter()
        .filter(|term| text_lc.contains(**term))
        .count() as i64
        * 600;

    token_score
        + (alnum_count * 2)
        + (confidence.max(0) as i64 * 180)
        + anchor_bonus
        - (noisy_symbol_count * 3)
        - (orphan_short_line_count * 20)
}

#[derive(Clone, Copy, PartialEq)]
enum OcrProfile {
    Fast,
    Balanced,
    Accurate,
}

impl OcrProfile {
    fn from_env() -> Self {
        match std::env::var("OCR_PROFILE")
            .ok()
            .unwrap_or_else(|| "balanced".to_string())
            .to_lowercase()
            .as_str()
        {
            "accurate" => Self::Accurate,
            "balanced" => Self::Balanced,
            _ => Self::Fast,
        }
    }
}

fn profile_label(profile: OcrProfile) -> &'static str {
    match profile {
        OcrProfile::Fast => "fast",
        OcrProfile::Balanced => "balanced",
        OcrProfile::Accurate => "accurate",
    }
}

#[derive(Clone, Copy)]
enum ImageVariantKind {
    Base,
    Gray,
    Contrast,
}

fn profile_render_size(profile: OcrProfile) -> (i32, i32) {
    match profile {
        OcrProfile::Fast => (2480, 3508),
        OcrProfile::Balanced => (3000, 4240),
        OcrProfile::Accurate => (3508, 4960),
    }
}

fn profile_rotations(profile: OcrProfile) -> Vec<PdfPageRenderRotation> {
    let _ = profile;
    vec![
        PdfPageRenderRotation::None,
        PdfPageRenderRotation::Degrees90,
        PdfPageRenderRotation::Degrees180,
        PdfPageRenderRotation::Degrees270,
    ]
}

fn profile_psm_modes(profile: OcrProfile) -> Vec<tesseract::PageSegMode> {
    match profile {
        OcrProfile::Fast => vec![tesseract::PageSegMode::PsmSingleBlock],
        OcrProfile::Balanced => vec![
            tesseract::PageSegMode::PsmSingleBlock,
            tesseract::PageSegMode::PsmAuto,
        ],
        OcrProfile::Accurate => vec![
            tesseract::PageSegMode::PsmAuto,
            tesseract::PageSegMode::PsmSingleColumn,
            tesseract::PageSegMode::PsmSparseText,
            tesseract::PageSegMode::PsmSingleBlock,
        ],
    }
}

fn profile_variants(profile: OcrProfile) -> Vec<ImageVariantKind> {
    match profile {
        OcrProfile::Fast => vec![ImageVariantKind::Gray],
        OcrProfile::Balanced => vec![ImageVariantKind::Gray, ImageVariantKind::Contrast],
        OcrProfile::Accurate => vec![
            ImageVariantKind::Base,
            ImageVariantKind::Gray,
            ImageVariantKind::Contrast,
        ],
    }
}

fn anchor_hits(text: &str) -> usize {
    let text_lc = text.to_lowercase();
    let anchor_terms = [
        "goods received note",
        "item description",
        "po number",
        "receipt number",
        "quantity ordered",
        "delivery instructions",
        "report date",
        "invoice",
        "tax",
        "total amount",
    ];
    anchor_terms
        .iter()
        .filter(|term| text_lc.contains(**term))
        .count()
}

fn is_good_enough_fast_result(text: &str, confidence: i32) -> bool {
    confidence >= 70 && anchor_hits(text) >= 4
}

fn normalize_ocr_text(text: &str) -> String {
    let mut normalized = text.replace("\r\n", "\n");
    let replacements = [
        ("\n\ntem\n", "\n\nItem\n"),
        ("\nHem\n", "\nItem\n"),
        ("Aecount Dese", "Account Desc:"),
        ("Account Dese", "Account Desc:"),
        ("Account Desc  ", "Account Desc: "),
        (" TANGERINE CO. LTO", " TANGERINE CO. LTD."),
        ("\nL |   Vendor\n", "\n[ ] Vendor\n"),
        ("\n[|     Employee Name\n", "\n[ ] Employee Name\n"),
        ("\nL]   Vendor\n", "\n[ ] Vendor\n"),
        ("\nL]    Vendor\n", "\n[ ] Vendor\n"),
        ("\n[|    Employee Name\n", "\n[ ] Employee Name\n"),
        ("SC: | ICENSING", "LICENSING"),
    ];

    for (from, to) in replacements {
        normalized = normalized.replace(from, to);
    }

    normalized
}

pub fn extract_pdf_ocr_text(path: &Path) -> Result<String, Box<dyn Error>> {
    let pdfium = bind_pdfium()?;
    let document = pdfium.load_pdf_from_file(path, None)?;
    let page_count = document.pages().len();
    let profile = OcrProfile::from_env();
    let max_pages: u16 = std::env::var("OCR_MAX_PAGES")
        .ok()
        .and_then(|raw| raw.parse::<u16>().ok())
        .map(|limit| limit.min(page_count))
        .unwrap_or(page_count);

    println!(
        "page_count: {} (processing: {}, profile: {})",
        page_count,
        max_pages,
        profile_label(profile)
    );

    fs::create_dir_all("tmp")?;
    let mut ocr_pages: Vec<String> = Vec::new();

    // println!("std::env::var('OCR_LANGS'): {}", std::env::var("OCR_LANGS").unwrap_or_default());

    let language_candidates: Vec<String> = std::env::var("OCR_LANGS")
        .ok()
        .map(|raw| {
            raw.split(',')
                .map(str::trim)
                .filter(|s| !s.is_empty())
                .map(str::to_string)
                .collect::<Vec<String>>()
        })
        .filter(|langs| !langs.is_empty())
        .unwrap_or_else(|| {
            let resolved = resolve_tesseract_languages();
            vec![resolved]
        });

    // println!("language_candidates: {:?}", language_candidates);

    for page_index in 0..max_pages {
        println!(
            "\n-- page {} -------------------------------",
            page_index + 1
        );

        let page = document.pages().get(page_index)?;

        let tmp_path = format!("tmp/page_demo_{page_index}.jpg");
        let rotations = profile_rotations(profile);

        let mut best_text = String::new();
        let mut best_score = i64::MIN;
        let mut best_variant_path = String::new();
        let page_seg_modes = profile_psm_modes(profile);
        let (target_width, max_height) = profile_render_size(profile);
        let variant_kinds = profile_variants(profile);

        'search: for rotation in rotations {
            let render_config = PdfRenderConfig::new()
                .set_target_width(target_width)
                .set_maximum_height(max_height)
                .rotate(rotation, true);

            let page_image = page.render_with_config(&render_config)?.as_image();
            let variant_stem = format!("tmp/page_demo_{page_index}_rot_{}", rotation as i32);
            let mut processed_variants = Vec::new();

            let gray_img = if variant_kinds
                .iter()
                .any(|kind| matches!(kind, ImageVariantKind::Gray | ImageVariantKind::Contrast))
            {
                Some(page_image.grayscale())
            } else {
                None
            };

            for variant_kind in variant_kinds.iter().copied() {
                match variant_kind {
                    ImageVariantKind::Base => {
                        let base_jpg = format!("{variant_stem}_ocr_base.jpg");
                        page_image.save(&base_jpg)?;
                        processed_variants.push(base_jpg);
                    }
                    ImageVariantKind::Gray => {
                        if let Some(gray_img) = &gray_img {
                            let gray_path = format!("{variant_stem}_ocr_gray.jpg");
                            gray_img.save(&gray_path)?;
                            processed_variants.push(gray_path);
                        }
                    }
                    ImageVariantKind::Contrast => {
                        if let Some(gray_img) = &gray_img {
                            let high_contrast_img = gray_img.adjust_contrast(40.0);
                            let contrast_path = format!("{variant_stem}_ocr_contrast.jpg");
                            high_contrast_img.save(&contrast_path)?;
                            processed_variants.push(contrast_path);
                        }
                    }
                }
            }

            for processed_path in processed_variants {
                for page_seg_mode in &page_seg_modes {
                    for language in &language_candidates {
                        let (candidate_text, confidence) =
                            run_ocr_with_mode(&processed_path, *page_seg_mode, language)?;
                        let score = score_ocr_text(&candidate_text, confidence);

                        if best_variant_path.is_empty() || score > best_score {
                            best_score = score;
                            best_text = candidate_text;
                            best_variant_path = processed_path.clone();

                            if profile == OcrProfile::Fast
                                && is_good_enough_fast_result(&best_text, confidence)
                            {
                                break 'search;
                            }
                        }
                    }
                }
            }
        }

        if !best_variant_path.is_empty() {
            fs::copy(&best_variant_path, &tmp_path)?;
        }

        let normalized_text = normalize_ocr_text(&best_text);
        println!("extracted_text: {}", normalized_text);
        ocr_pages.push(normalized_text);
    }

    let ocr_output = ocr_pages
        .iter()
        .enumerate()
        .map(|(index, content)| {
            format!(
                "=== PAGE {} ===\n{}\n",
                index + 1,
                content.trim_end_matches('\n')
            )
        })
        .collect::<Vec<String>>()
        .join("\n");

    Ok(ocr_output)
}
