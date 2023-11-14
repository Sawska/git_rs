use std::io::{self,BufRead, Read,Write};
use std::fs::{self,File,OpenOptions};
use std::time::{SystemTime,UNIX_EPOCH};
use std::path::{Path,PathBuf};
use crypto::digest::Digest;
use crypto::sha2::Sha256;
use sha1::{Sha1,Digest as OtherDigest};

enum TreeEntry {
    Blob {mode: &'static str,hash:String,name:String},
    Tree {mode: &'static str,hash:String,name:String,entries:Vec<TreeEntry>},
}

enum IndexCheckResult {
    InIndex(String),
    NotInIndex,
    Error(io::Error),
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

pub fn add_all(base_directory:&str) -> io::Result<()> {
    if let Ok(entries) = fs::read_dir(base_directory) {
        for entry in entries {
            if let Ok(entry) = entry {
                let file_name = entry.file_name();
                let path = entry.path();

                if path.is_dir() && file_name != ".gitrs" && file_name != ".git" {
                    add_all(&path.to_str().unwrap())?;
                } else if path.is_file() {
                    match check_if_in_index(path.to_str().unwrap()) {
                        IndexCheckResult::InIndex(hashed_line) => {
                            remove_file_from_index("../../.gitrs/index",&hashed_line)?;
                        }
                        IndexCheckResult::NotInIndex => {
                            add_to_objects(&path.to_str().unwrap())?;
                        }
                        IndexCheckResult::Error(err) => {
                            eprintln!("Error checking the index: {:?}",err);
                        }
                    }
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

fn hash_line(line:&str) -> String {
    let mut hasher = Sha256::new();
    hasher.input_str(line);
    hasher.result_str()
}

fn add_to_objects(path:&str) -> io::Result<()> {
    let file = File::open(&path)?;

    let reader = io::BufReader::new(file);

    let mut arr = Vec::<String>::new();

    for line_result in reader.lines() {
        let line = line_result?;
        let hashed_line = hash_line(&line);

        arr.push(hashed_line);
    }

    let name_dir = &arr[0][0..1];

    fs::create_dir(format!("../../.gitrs/objects/{}",name_dir))?;

    let another_name = &arr.join("");

    let file_path = format!("../../.gitrs/objects/{}.bin",another_name);

    let file_path_clone = file_path.clone();

    let mut file = File::create(file_path_clone)?;

    for line in &arr {
        file.write_all(line.as_bytes())?;
    }

    add_to_index(&arr,&file_path);

    Ok(())
}

fn add_to_index(arr: &Vec<String>,file_path:&str) -> io::Result<()> {

    let mut index_file = OpenOptions::new().create(true).append(true).open("../../.girts/index")?;

    for line in arr {
        let index_line = format!("{} 100644 {}",line,file_path);

        index_file.write_all(index_line.as_bytes())?;
    }

    Ok(())
}

fn check_if_in_index(file_path: &str) -> IndexCheckResult {
    let index_file = match File::open("../../.gitrs/index") {
        Ok(file) => file,
        Err(err) => return IndexCheckResult::Error(err),
    };

    let reader = io::BufReader::new(index_file);

    let hashed_line = hash_line(file_path);

    for line_result in reader.lines() {
        if let Ok(line) = line_result {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() == 3 && parts[2] == hashed_line {
                return IndexCheckResult::InIndex(hashed_line);
            }
        }
    }

    IndexCheckResult::NotInIndex
}

fn remove_file_from_index(file_path: &str, hashed_line: &str) -> io::Result<()> {
    let index_file = match File::open(file_path) {
        Ok(file) => file,
        Err(err) => return Err(err),
    };

    let reader = io::BufReader::new(index_file);

    let lines: Vec<String> = reader
        .lines()
        .filter(|line| line.as_ref().map_or(false, |l| {
            let parts: Vec<&str> = l.split_whitespace().collect();
            if parts.len() == 3 && parts[2] == hashed_line {
                return false;
            }
            true
        }))
        .map(|line| line.unwrap())
        .collect();

    let mut file = OpenOptions::new().write(true).truncate(true).open(file_path)?;

    for line in lines {
        writeln!(file, "{}", line)?;
    }

    Ok(())
}

pub fn create_commit(base_directory:&str,author:&str,commiter:&str,commit_message:&str) {
        let tree_hash = match create_tree_from_index(base_directory) {
            Ok(tree) => hash_tree(tree),
            Err(_) => {
                eprintln!("Error:Unable to create the tree object.");
                return;
            },
        };

        match create_commit_object(author,commiter,&tree_hash,commit_message) {
            Ok(()) => println!("Commit create succesfully."),
            Err(err) => eprintln!("Error: {:?}",err),
        }
}



fn create_commit_object(author:&str,commiter:&str,tree_hash:&str,commit_message:&str) -> io::Result<()> {
    let timestamp = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
    let commit_content = format!(
        "tree {}\n\
        author {} {}\n\
        commiter {} {}\n\n\
        {}",
        tree_hash,author,timestamp,commiter,timestamp,commit_message
    );

    let commit_hash = hash_line(&commit_content);

    let commit_hash = hash_line(&commit_content);

    let commit_file_path = format!(".gitrs/objects/{}.bin",commit_hash);
    let mut commit_file = File::create(&commit_file_path)?;
    commit_file.write_all(commit_content.as_bytes())?;

    Ok(())
}

fn create_tree_from_index(base_directory: &str) -> io::Result<TreeEntry> {
    let mut tree_entries = Vec::new();

    
    let index_file = File::open("../../.gitrs/index")?;
    let reader = io::BufReader::new(index_file);

    for line_result in reader.lines() {
        let line = line_result?;
        let parts: Vec<&str> = line.split_whitespace().collect();

        if parts.len() == 3 {
            let mode = "100644";  
            let hash = parts[0];
            let name = parts[2];

            let entry = if name.contains('/') {
                
                let subdirectory = name.split('/').next().unwrap();
                let subdirectory_entry = tree_entries
                    .iter_mut()
                    .find(|entry| matches!(entry, TreeEntry::Tree { name, .. } if name == subdirectory));

                if let Some(TreeEntry::Tree { entries, .. }) = subdirectory_entry {
                    let remaining_path = name.split('/').skip(1).collect::<Vec<&str>>().join("/");
                    create_tree_entry(&remaining_path, mode, hash, name)?
                } else {
                    let tree_entry = create_tree_entry(name, mode, hash, name)?;
                    TreeEntry::Tree { mode, hash: hash.to_string(), name: subdirectory.to_string(), entries: vec![tree_entry] }
                }
            } else {
                
                create_tree_entry(name, mode, hash, name)?
            };

            tree_entries.push(entry);
        }
    }

    Ok(TreeEntry::Tree { mode: "040000", hash: "".to_string(), name: base_directory.to_string(), entries: tree_entries })
}

fn create_tree_entry(name: &str, mode: &'static str, hash: &str, full_path: &str) -> io::Result<TreeEntry> {
    let path_parts: Vec<&str> = full_path.split('/').collect();
    if path_parts.len() == 1 {
        
        Ok(TreeEntry::Blob { mode, hash: hash.to_string(), name: name.to_string() })
    } else {
        
        let subdirectory_name = path_parts[0];
        let remaining_path = path_parts[1..].join("/");
        let subdirectory_entry = create_tree_entry(name, mode, hash, &remaining_path)?;
        Ok(TreeEntry::Tree { mode: "040000", hash: "".to_string(), name: subdirectory_name.to_string(), entries: vec![subdirectory_entry] })
    }
}

fn hash_tree_entry(entry:&TreeEntry) -> String {
    match entry {
        TreeEntry::Blob {mode,hash,name} => format!("{} {} {}\t{}",mode,"blob",hash,name),
        TreeEntry::Tree {mode,hash,name,entries} => {
            let entries_str: Vec<String> = entries.iter().map(hash_tree_entry).collect();
            format!("{} {} {}\t{}\n{}",mode,"tree",hash,name,entries_str.join("\n"))
        }
    }
}

fn hash_tree(tree:TreeEntry) -> String {
    match tree {
        TreeEntry::Tree {mode,hash,name,entries} => {
            let tree_str = hash_tree_entry(&TreeEntry::Tree {mode,hash,name,entries});
            hash_string(&tree_str)
        }
        TreeEntry::Blob { mode, hash, name } => todo!(),
    }
}

fn hash_string(input: &str) -> String {
    let mut hasher = Sha1::new();
    hasher.update(input);
    let result = hasher.finalize();
    format!("{:x}", result)
}

fn read_head() -> io::Result<String> {
    let path = format!(".gitrs/HEAD");
    let mut file = File::open(path)?;
    let mut content = String::new();  
    file.read_to_string(&mut content)?;
    Ok(content.trim().to_string())
}

fn rebuild_tree() -> io::Result<TreeEntry> {
    let head_hash = read_head()?;
    let root_tree = read_object(&head_hash)?;
    Ok(root_tree)
}

fn read_object(hash: &str) -> io::Result<TreeEntry> {
    let path = format!(".gitrs/objects/{}.bin", hash);
    let mut file = File::open(path)?;
    let mut content = String::new();
    file.read_to_string(&mut content)?;

    if content.starts_with("tree") {
        let mut tree_entries = Vec::new();
        for line in content.lines().skip(1) {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() == 4 {
                let mode = parts[0];
                let hash = parts[2];
                let name = parts[3];
                let entry = if name.contains('/') {
                    let subdirectory = name.split('/').next().unwrap();
                    let subdirectory_entry = tree_entries
                        .iter_mut()
                        .find(|entry| matches!(entry, TreeEntry::Tree { name, .. } if name == subdirectory));

                    if let Some(TreeEntry::Tree { entries, .. }) = subdirectory_entry {
                        let remaining_path = name.split('/').skip(1).collect::<Vec<&str>>().join("/");
                        create_tree_entry(&remaining_path, mode, hash, name)?
                    } else {
                        let tree_entry = create_tree_entry(name, mode, hash, name)?;
                        TreeEntry::Tree { mode, hash: hash.to_string(), name: subdirectory.to_string(), entries: vec![tree_entry] }
                    }
                } else {
                    create_tree_entry(name, mode, hash, name)?
                };
                tree_entries.push(entry);
            }
        }
        Ok(TreeEntry::Tree { mode: "040000", hash: "".to_string(), name: "".to_string(), entries: tree_entries })
    } else {
        // Handle blob case
        Ok(TreeEntry::Blob { mode: "100644", hash: "".to_string(), name: "".to_string() })
    }
}
