# OCR Module Overview

This project now keeps OCR logic in `src/ocr.rs` to make `src/main.rs` smaller and easier to read.

## Entry Point

- `extract_pdf_ocr_text(path: &Path) -> Result<String, Box<dyn Error>>`
  - Loads a PDF via Pdfium
  - Renders each page to image variants
  - Runs Tesseract OCR with multiple settings
  - Picks the best candidate by score
  - Returns all pages as one text block

## OCR Pipeline

1. Bind Pdfium (`bind_pdfium`) from known dynamic library locations or `PDFIUM_LIB_PATH`.
2. Resolve OCR profile (`OcrProfile`) from `OCR_PROFILE`.
3. Render each page with different rotations and image variants.
4. Run Tesseract with selected language(s) and page segmentation mode(s).
5. Score each candidate text (`score_ocr_text`) and keep the best result.
6. Normalize common OCR mistakes (`normalize_ocr_text`).
7. Join page results into one output string.

## Install Third-Party Dependencies

The OCR flow needs external native libraries. On macOS, use this setup:

1. Install required tools:

```bash
brew install tesseract leptonica
```

2. Install language data (English + Thai):

```bash
mkdir -p third_party/tessdata
curl -L -o third_party/tessdata/eng.traineddata https://github.com/tesseract-ocr/tessdata/raw/main/eng.traineddata
curl -L -o third_party/tessdata/tha.traineddata https://github.com/tesseract-ocr/tessdata/raw/main/tha.traineddata
```

3. Install Pdfium (`libpdfium.dylib`):
   - Download macOS binary from [pdfium-binaries releases](https://github.com/bblanchon/pdfium-binaries/releases)
   - Put `libpdfium.dylib` in one of these paths:
     - `/opt/homebrew/lib/libpdfium.dylib`
     - `/usr/local/lib/libpdfium.dylib`
     - `./third_party/pdfium/lib/libpdfium.dylib`
   - Or set `PDFIUM_LIB_PATH` to the absolute path.

4. Export runtime env vars (recommended):

```bash
export TESSDATA_PREFIX="$PWD/third_party/tessdata"
export PDFIUM_LIB_PATH="$PWD/third_party/pdfium/lib/libpdfium.dylib"
```

5. Verify install:

```bash
tesseract --version
ls "$TESSDATA_PREFIX"/eng.traineddata "$TESSDATA_PREFIX"/tha.traineddata
ls "$PDFIUM_LIB_PATH"
```

## Environment Variables

- `PDFIUM_LIB_PATH`: absolute path to `libpdfium.dylib`.
- `TESSDATA_PREFIX`: location of Tesseract `*.traineddata` files.
- `OCR_PROFILE`: one of `fast`, `balanced`, `accurate`.
- `OCR_MAX_PAGES`: optional page limit.
- `OCR_LANGS`: comma-separated language codes (example: `tha+eng,eng`).

## Main Integration

`src/main.rs` now only calls:

```rust
let ocr_output = ocr::extract_pdf_ocr_text(path)?;
write_file("tmp/ocr_output.txt", &ocr_output)?;
```

This keeps OCR implementation details inside one module and makes future OCR tuning easier.
