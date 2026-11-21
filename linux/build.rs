use std::{fs, io, path::Path};

// The shared WebUI is used unmodified. Each tag is placed before (true) or after (false) its anchor
// line: the Linux transport precedes every shared script, the overview follows them.
const INJECTIONS: [(&str, &str, bool); 3] = [
    ("<head>", r#"<script src="linux/webview2-shim.js"></script>"#, false),
    ("</head>", r#"<link rel="stylesheet" href="linux/overview.css" />"#, true),
    ("</body>", r#"<script src="linux/overview.js"></script>"#, true),
];

fn assemble_html(path: &Path) -> io::Result<String> {
    let html = fs::read_to_string(path)?;
    let mut out = String::with_capacity(html.len());
    for line in html.lines() {
        if let Some(name) = line.trim().strip_prefix("<!-- @include ").and_then(|s| s.strip_suffix(" -->")) {
            out.push_str(&assemble_html(&path.with_file_name(name))?);
        } else {
            out.push_str(line);
        }
        out.push('\n');
    }
    Ok(out)
}

fn inject(html: &str) -> Result<String, String> {
    let mut out = String::with_capacity(html.len() + 256);
    let mut found = [0; INJECTIONS.len()];
    for line in html.lines() {
        let injection = INJECTIONS.iter().position(|(anchor, ..)| line.trim() == *anchor);
        let parts = match injection {
            Some(index) => {
                found[index] += 1;
                let (_, tag, before) = INJECTIONS[index];
                if before { [tag, line] } else { [line, tag] }
            }
            None => [line, ""],
        };
        for part in &parts[..if injection.is_some() { 2 } else { 1 }] {
            out.push_str(part);
            out.push('\n');
        }
    }
    for (count, (anchor, ..)) in found.iter().zip(INJECTIONS) {
        if *count != 1 {
            return Err(format!("WebUI index.html must contain exactly one `{anchor}` line, found {count}"));
        }
    }
    Ok(out)
}

fn copy_tree(src: &Path, dst: &Path) -> io::Result<()> {
    fs::create_dir_all(dst)?;
    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let target = dst.join(entry.file_name());
        if entry.file_type()?.is_dir() {
            copy_tree(&entry.path(), &target)?;
        } else {
            fs::copy(entry.path(), target)?;
        }
    }
    Ok(())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let source = Path::new("../src/AstralParty.Toys/WebUI");
    let native = Path::new("webui");
    println!("cargo:rerun-if-changed={}", source.display());
    println!("cargo:rerun-if-changed={}", native.display());
    let dest = Path::new("generated/webui");
    if dest.exists() { fs::remove_dir_all(dest)?; }
    copy_tree(source, dest)?;
    copy_tree(native, &dest.join("linux"))?;
    fs::write(dest.join("index.html"), inject(&assemble_html(&source.join("index.html"))?)?)?;
    tauri_build::try_build(tauri_build::Attributes::new().app_manifest(
        tauri_build::AppManifest::new().commands(&["host_message"]),
    ))?;
    Ok(())
}
