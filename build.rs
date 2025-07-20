use std::{
    fs::{self, DirEntry},
    path::Path,
};

const HEROICONS_REPO_URL: &str = "https://github.com/tailwindlabs/heroicons.git";
const HEROICONS_REPO_DIR: &str = "heroicons";

fn main() {
    remove_heroicons_directory();

    clone(HEROICONS_REPO_URL, HEROICONS_REPO_DIR);

    let mut file_content = String::from(
        "use maud::html;
use maud::Markup;
",
    );

    let twenty_four_dir = fs::read_dir(format!("{HEROICONS_REPO_DIR}/optimized/24"))
        .expect("Failed to read 24 directory");

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

    let mut icon_function = format!("\npub fn {icon_name}_{style_name}() -> Markup {{\n");
    icon_function.push_str(&html2maud::html2maud(&icon_string));
    icon_function.push_str("\n}\n");

    Some(icon_function)
}

fn remove_heroicons_directory() {
    if Path::new(HEROICONS_REPO_DIR).exists() {
        fs::remove_dir_all(HEROICONS_REPO_DIR).expect("Failed to remove directory");
    }
}

/// Clones a remote repository to the specified destination.
///
/// This is the more low level version of performing a git clone of a
/// remote repository.
///
/// The git2 crate would be easier and more idiomatic, but it depends on openssl
/// and is not available on all platforms. This is also also causing compilation errors when
/// when you want to compile static linked binaries with musl.
///
/// Gix allows to opt-into rustls TLS implementation and therefore support more platforms.
///
/// See https://github.com/rust-lang/git2-rs/issues/623
fn clone(repo_url: &str, dest: &str) -> gix::Repository {
    // Setup the remote repository URL and destination path
    let mut prepared_clone = gix::clone::PrepareFetch::new(
        repo_url,
        dest,
        gix::create::Kind::WithWorktree,
        gix::create::Options::default(),
        gix::open::Options::isolated(),
    )
    .expect("Failed to prepare clone");

    // Fetch the remote and prepare for the checkout
    let (mut prepare_checkout, _) = prepared_clone
        .fetch_then_checkout(gix::progress::Discard, &gix::interrupt::IS_INTERRUPTED)
        .expect("Failed to fetch and checkout");

    // Do the checkout...
    //This will be the latest commit on the default branch (in this case main)
    let (repo, _) = prepare_checkout
        .main_worktree(gix::progress::Discard, &gix::interrupt::IS_INTERRUPTED)
        .expect("Failed to get main worktree");
    repo
}
