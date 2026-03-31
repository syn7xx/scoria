use super::*;
use crate::engine::clipboard::Content;
use std::fs;
use std::path::Path;
use tempfile::TempDir;

fn test_config(vault_path: &Path) -> Config {
    Config {
        vault_path: vault_path.to_path_buf(),
        target: SaveTarget::NewFileInFolder,
        folder: "scoria".into(),
        append_file: "Scoria.md".into(),
        filename_template: "clip-%Y-%m-%d-%H%M%S.md".into(),
        prepend_timestamp_header: true,
        hotkey: None,
        autostart: false,
        auto_update: false,
        language: "en".into(),
    }
}

fn test_config_append(vault_path: &Path) -> Config {
    Config {
        vault_path: vault_path.to_path_buf(),
        target: SaveTarget::AppendToFile,
        folder: "scoria".into(),
        append_file: "Notes.md".into(),
        filename_template: "clip-%Y-%m-%d-%H%M%S.md".into(),
        prepend_timestamp_header: true,
        hotkey: None,
        autostart: false,
        auto_update: false,
        language: "en".into(),
    }
}

fn make_vault() -> (TempDir, std::path::PathBuf) {
    let tmp = TempDir::new().expect("create temp dir");
    let vault = tmp.path().join("vault");
    fs::create_dir_all(&vault).expect("create vault dir");
    (tmp, vault)
}

#[test]
fn test_save_text_new_file() {
    let (_tmp, vault) = make_vault();

    let cfg = test_config(&vault);
    let content = Content::Text("Hello, World!".into());

    let result = save(&cfg, &content);
    assert!(result.is_ok());

    let path = result.expect("save text to new file");
    assert!(path.exists());

    let contents = fs::read_to_string(&path).expect("read saved markdown");
    assert!(contents.contains("Hello, World!"));
    assert!(contents.contains("## ")); // timestamp header
}

#[test]
fn test_save_text_new_file_no_timestamp() {
    let (_tmp, vault) = make_vault();

    let mut cfg = test_config(&vault);
    cfg.prepend_timestamp_header = false;
    let content = Content::Text("No timestamp".into());

    let result = save(&cfg, &content);
    assert!(result.is_ok());

    let path = result.expect("save text without timestamp");
    let contents = fs::read_to_string(&path).expect("read saved markdown");
    assert_eq!(contents, "No timestamp");
}

#[test]
fn test_save_text_empty() {
    let (_tmp, vault) = make_vault();

    let cfg = test_config(&vault);
    let content = Content::Text("".into());

    let result = save(&cfg, &content);
    assert!(result.is_err());
    let err = result.unwrap_err().to_string();
    assert!(err.contains("empty"));
}

#[test]
fn test_save_text_append() {
    let (_tmp, vault) = make_vault();

    let cfg = test_config_append(&vault);

    // First save
    let content1 = Content::Text("First entry".into());
    let result1 = save(&cfg, &content1);
    assert!(result1.is_ok());

    // Second save - should append
    let content2 = Content::Text("Second entry".into());
    let result2 = save(&cfg, &content2);
    assert!(result2.is_ok());

    // Check both are in the file
    let append_file = vault.join("Notes.md");
    let contents = fs::read_to_string(&append_file).expect("read appended file");
    assert!(contents.contains("First entry"));
    assert!(contents.contains("Second entry"));
}

#[test]
fn test_save_text_append_separator() {
    let (_tmp, vault) = make_vault();

    let cfg = test_config_append(&vault);

    // Create existing file
    let append_file = vault.join("Notes.md");
    fs::write(&append_file, "Existing content").expect("seed append file");

    let content = Content::Text("New content".into());
    let result = save(&cfg, &content);
    assert!(result.is_ok());

    let contents = fs::read_to_string(&append_file).expect("read appended file");
    assert!(contents.contains("Existing content"));
    assert!(contents.contains("New content"));
}

#[test]
fn test_save_image() {
    let (_tmp, vault) = make_vault();

    let cfg = test_config(&vault);
    let image_data = vec![0x89, 0x50, 0x4E, 0x47]; // PNG header
    let content = Content::Image {
        data: image_data,
        ext: "png",
    };

    let result = save(&cfg, &content);
    assert!(result.is_ok());

    let path = result.expect("save image");
    assert!(path.exists());

    // Check attachments folder was created
    let attachments = vault.join("scoria").join("attachments");
    assert!(attachments.is_dir());

    // Check md file references the image
    let md_content = fs::read_to_string(&path).expect("read image markdown");
    assert!(md_content.contains("![[attachments/"));
}

#[test]
fn test_save_image_empty() {
    let tmp = TempDir::new().unwrap();
    let vault = tmp.path().join("vault");
    fs::create_dir_all(&vault).unwrap();

    let cfg = test_config(&vault);
    let content = Content::Image {
        data: vec![],
        ext: "png",
    };

    let result = save(&cfg, &content);
    assert!(result.is_err());
    let err = result.unwrap_err().to_string();
    assert!(err.contains("empty"));
}

#[test]
fn test_save_creates_folder_if_missing() {
    let tmp = TempDir::new().unwrap();
    let vault = tmp.path().join("vault");
    // Create vault dir but not the subfolder
    std::fs::create_dir_all(&vault).unwrap();

    let cfg = test_config(&vault);
    let content = Content::Text("Test".into());

    let result = save(&cfg, &content);
    assert!(result.is_ok(), "save failed: {:?}", result);

    // Folder should be created automatically
    assert!(vault.join("scoria").is_dir());
}

#[test]
fn test_save_to_nonexistent_vault() {
    let tmp = TempDir::new().unwrap();
    let vault = tmp.path().join("nonexistent_vault");
    // Don't create the vault directory

    let cfg = test_config(&vault);
    let content = Content::Text("Test".into());

    let result = save(&cfg, &content);
    assert!(result.is_err());
}

#[test]
fn test_filename_template_format() {
    let tmp = TempDir::new().unwrap();
    let vault = tmp.path().join("vault");
    fs::create_dir_all(&vault).unwrap();

    let mut cfg = test_config(&vault);
    cfg.filename_template = "test-%Y%m%d.md".into();

    let content = Content::Text("Test".into());
    let result = save(&cfg, &content);
    assert!(result.is_ok());

    let path = result.unwrap();
    let filename = path.file_name().unwrap().to_string_lossy();
    // Should contain date in YYYYMMDD format
    assert!(filename.starts_with("test-"));
    assert!(filename.ends_with(".md"));
}

#[test]
fn test_append_file_creates_subdirs() {
    let tmp = TempDir::new().unwrap();
    let vault = tmp.path().join("vault");
    fs::create_dir_all(&vault).unwrap();

    let mut cfg = test_config_append(&vault);
    cfg.append_file = "subdir/Notes.md".into();

    let content = Content::Text("Test".into());
    let result = save(&cfg, &content);
    assert!(result.is_ok());

    let append_file = vault.join("subdir").join("Notes.md");
    assert!(append_file.exists());
}

#[test]
fn test_append_file_rejects_windows_absolute_path() {
    let tmp = TempDir::new().unwrap();
    let vault = tmp.path().join("vault");
    fs::create_dir_all(&vault).unwrap();

    let mut cfg = test_config_append(&vault);
    cfg.append_file = r"C:\Temp\pwn.md".into();

    let content = Content::Text("Test".into());
    let result = save(&cfg, &content);
    assert!(result.is_err());
}

#[test]
fn test_append_file_rejects_unc_path() {
    let tmp = TempDir::new().unwrap();
    let vault = tmp.path().join("vault");
    fs::create_dir_all(&vault).unwrap();

    let mut cfg = test_config_append(&vault);
    cfg.append_file = r"\\server\share\pwn.md".into();

    let content = Content::Text("Test".into());
    let result = save(&cfg, &content);
    assert!(result.is_err());
}

#[test]
fn test_folder_rejects_traversal() {
    let tmp = TempDir::new().unwrap();
    let vault = tmp.path().join("vault");
    fs::create_dir_all(&vault).unwrap();

    let mut cfg = test_config(&vault);
    cfg.folder = "../outside".into();

    let content = Content::Text("Test".into());
    let result = save(&cfg, &content);
    assert!(result.is_err());
}
