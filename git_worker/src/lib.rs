use std::io::BufRead;
use std::{fs, io};
use std::fs::File;
// use std::io;
use std::path::{Path,PathBuf};
pub struct  Commit_tree {
    pub parents:Vec<Commit_tree>,
    pub children: Vec<Commit_tree>,
    pub merged_branch_name: Option<String>,
    pub commit_id: String,
    pub branch_name:String,
}


pub fn init() {
    let _ = fs::create_dir_all("../../.gitrs");
    // pointers to specific commit
    let _ = fs::create_dir_all("../../.gitrs/refs");
    // git stores commits here
    let _ = fs::create_dir_all("../../.gitrs/objects");

    //contains references to the heads of branches.
    let _ = File::create("../../.gitrs/refs/heads");

    //points to the last commit
    let _ = File::create("../../gitrs/HEAD");

    //stored permanents add files
    let _ = File::create("../../gitrs/index");

    println!("create .gitrs file");
}

pub fn add_all(base_directory: &str) -> io::Result<()> {
    let commit = read_info_from_commit();

    match commit {
        Some(commit) => {
            // Implement logic for when commit is present
            todo!();
        }
        None => {
            let entries = fs::read_dir(base_directory)?;

            for entry in entries {
                let entry = entry?;
                let file_name = entry.file_name();
                let path = entry.path();

                if path.is_dir() && file_name != ".gitrs" && file_name != ".git" {
                    add_all(&path.to_str().unwrap())?;
                } else {
                    add_to_index(entry.path().to_str().unwrap())?;
                }
            }
        }
    }

    Ok(())
}

fn read_info_from_commit() -> Option<PathBuf> {
    let commit = match fs::read_to_string("../../gitrs/HEAD") {
        Ok(commit) => commit,
        Err(_) => return None,
    };

    let commit_path = Path::new("../../.gitrs/objects").join(&commit[0..2]).join(&commit[2..]);

    if commit_path.is_dir() {
        Some(commit_path)
    } else {
        None
    }
}

fn add_to_index(file_path: &str) -> io::Result<()> {
    let file = File::open(file_path)?;
    let file_name = Path::new(file_path).file_name().and_then(|name| name.to_str()).unwrap_or_default();

    let mut lines = Vec::new();

    let reader = io::BufReader::new(file);

    for line in reader.lines() {
        match line {
            Ok(line_content) => {
                lines.push(line_content);
            }
            Err(e) => {
                eprintln!("Error reading line: {}", e);
            }
        }
    }

    lines.push(format!("filename: {}", file_name));
    lines.push(format!("path: {}", file_path));

    let content = lines.join("\n");

    let index_path = "../../gitrs/index";
    fs::create_dir_all(Path::new(index_path).parent().unwrap())?;
    fs::write(index_path, content)?;

    Ok(())
}
