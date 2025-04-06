use std::{
    fs::{self, DirEntry},
    path::Path,
};

use git2::Repository;

fn main() {
    remove_heroicons_directory();

    Repository::clone("https://github.com/tailwindlabs/heroicons.git", "heroicons")
        .expect("Failed to clone the repository");

    let mut file_content = String::from(
        "use maud::html;
use maud::Markup;
",
    );

    let twenty_four_dir =
        fs::read_dir("heroicons/optimized/24").expect("Failed to read 24 directory");

    for style_directories in twenty_four_dir.filter_map(|entry_res| {
        entry_res.ok().and_then(|entry| {
            entry.file_type().ok().and_then(|file_type| {
                if file_type.is_dir() {
                    Some(entry)
                } else {
                    None
                }
            })
        })
    }) {
        let outline_dir = fs::read_dir(style_directories.path())
            .unwrap_or_else(|_| panic!("Failed to read directory {:?}", style_directories.path()));

        for icon_file in outline_dir.filter_map(|entry_res| {
            entry_res.ok().and_then(|entry| {
                entry.file_name().to_str().and_then(|name| {
                    if name.ends_with(".svg") {
                        Some(entry)
                    } else {
                        None
                    }
                })
            })
        }) {
            file_content.push_str(
                &process_icon_file(
                    &icon_file,
                    style_directories.file_name().to_str().unwrap_or_default(),
                )
                .unwrap_or_default(),
            );
        }
    }

    let dest_path = Path::new("src").join("lib.rs");
    fs::write(&dest_path, file_content).expect("Failed to write to lib.rs");

    remove_heroicons_directory();
}

fn process_icon_file(icon_file: &DirEntry, style_name: &str) -> Option<String> {
    let icon_string = fs::read_to_string(icon_file.path()).ok()?;

    let icon_name = icon_file
        .file_name()
        .to_str()?
        .replace(".svg", "")
        .replace("-", "_");

    let mut icon_function = format!("\npub fn {}_{}() -> Markup {{\n", icon_name, style_name);
    icon_function.push_str(&html2maud::html2maud(&icon_string));
    icon_function.push_str("\n}\n");

    Some(icon_function)
}

fn remove_heroicons_directory() {
    if Path::new("heroicons").exists() {
        fs::remove_dir_all("heroicons").expect("Failed to remove directory");
    }
}
