use std::io::{self,BufRead, Read,Write, SeekFrom,Seek,BufReader};
use std::fs::{self,File,OpenOptions};
use std::time::{SystemTime,UNIX_EPOCH};
use crypto::digest::Digest;
use crypto::sha2::Sha256;
use sha1::{Sha1,Digest as OtherDigest};
use std::env::{self, var};

enum TreeEntry {
    Blob {mode: String,hash:String,name:String},
    Tree {mode: String,hash:String,name:String,entries:Vec<TreeEntry>},
}



enum IndexCheckResult {
    InIndex(String),
    NotInIndex,
    Error(io::Error),
}

pub fn status() {
    // gives iformation about current directory status
}


pub fn init() {
    let _ = fs::create_dir_all("../../.gitrs");
    env::set_var(".gitrs", "../../.gitrs");
    // pointers to specific commit
    let _ = fs::create_dir_all("../../.gitrs/refs");
    env::set_var("refs", "../../.gitrs/refs");
    // git stores commits here
    let _ = fs::create_dir_all("../../.gitrs/objects");
    env::set_var("objects", "../../.gitrs/objects");
    //contains references to the heads of branches.
    let _ = File::create("../../.gitrs/refs/heads");
    env::set_var("heads_ref", "../../.gitrs/refs/heads");

    //points to the last commit
    let _ = File::create("../../.gitrs/HEAD");
    env::set_var("heads", "../../.gitrs/HEAD");

    create_branch("main", "");
    update_head("main");

    //stored permanents add files
    let _ = File::create("../../.gitrs/index");
    env::set_var("index", "../../.gitrs/index");
    
    //config file for git
    let _ = File::create("../../.gitrs/config");
    env::set_var("config","../../.gitrs/config");
    fill_config();

    println!("created .gitrs file");
}

pub fn create_branch(branch_name:&str,content:&str) {
    create_branch_in_refs(branch_name,content);
}



fn update_head(branch_name:&str) -> io::Result<()> {

    let mut head_file = File::create(env::var("heads").unwrap())?;

    let path_to_refs_heads = env::var("heads_ref").unwrap();

    let content = format!("refs: {}/{}\n",path_to_refs_heads,branch_name);

    head_file.write_all(content.as_bytes());

    Ok(())
}

pub fn delete_brench(name:&str) {
    let result = find_branch(name);

    match  result {
        Ok(res) => {
            if res {
                let current_brench = read_head();

                match current_brench {
                    Ok(path) => {
                        let path_for_this = format!("{}/{}",env::var("heads_ref").unwrap(),name);
                        let is_equal = path == path_for_this;

                        if is_equal {
                            println!("cannot delete current branch");
                        } else {
                            delete(&path_for_this);
                        }
                    },
                    Err(err) => println!("this err occured: {}",err),
                }

            } else {
                println!("this brench does not exist");
            }
        },
        Err(err) => println!("this err occured: {}",err),
    }
}

fn delete(path:&str) {
    fs::remove_file(path);
}

pub fn list_branches() -> String {
    let dir = fs::read_dir(env::var("heads_ref").unwrap()).unwrap();

    let mut res = String::new();

    for entries in dir {
        let file = entries.unwrap();
        let name = file.file_name();

        res.push_str(name.to_str().unwrap());
        res.push('\n');
    }
    res
} 

pub fn checkout(name:&str) {
    let result = find_branch(name);

    match  result {
        Ok(res) => {
            if res {
                update_head(name);
            } else {
                println!("this branch does not exist\ntry using checkout -b <branch-name>");
            }
        },
        Err(err) =>  println!("this err occured: {}",err),
    }
}

pub fn checkout_b(branch_name:&str) {
    let path_to_read = read_head().unwrap();
    let hash = read_from_refs(path_to_read).unwrap();
    create_branch(branch_name, &hash);
}



fn find_branch(name:&str) -> io::Result<bool> {
    let refs = fs::read_dir(env::var("heads_ref").unwrap()).unwrap();

    let mut is_branch = false;

    for entry in refs {
        let entry_file = entry.unwrap();
        if entry_file.file_name() == name {
            is_branch = true;
            break;
        }
    }
    Ok(is_branch)
}

fn create_branch_in_refs(branch_name:&str,content:&str) -> io::Result<()> {
    
    let file_path = format!("{}/{}",env::var("heads_ref").unwrap(),branch_name);

    let mut refs  = File::create(file_path)?;

    refs.write_all(content.as_bytes());

    Ok(())
}

fn fill_config()  -> io::Result<()> {
    let mut config_file = File::create(env::var("config").unwrap())?;

    let default_name = "Your name";

    let default_email = "your.email@example.com";

    let config_content = format!(
        "user.email {}\n\
        user.name {}\n\
        global user.email {}\n
        global user.name {}\n",
    default_email,default_name,default_email,default_name);

    config_file.write_all(config_content.as_bytes());

    Ok(())
}

pub fn set_user_input(part_to_match: &str, user_value: &str) -> io::Result<()> {
    
    let config_path = env::var("config").unwrap();
    let mut config_file = OpenOptions::new().read(true).write(true).open(&config_path)?;

    
    let temp_path = format!("{}.temp", &config_path);
    let mut temp_file = File::create(&temp_path)?;

    
    config_file.seek(SeekFrom::Start(0))?;

    for line in BufReader::new(&config_file).lines() {
        let original_line = line?;

        
        let parts: Vec<&str> = original_line.split_whitespace().collect();

        
        if let Some(property) = parts.get(0) {
            if property == &part_to_match {
                
                writeln!(&mut temp_file, "{} {}", part_to_match, user_value)?;
            } else {
                
                writeln!(&mut temp_file, "{}", original_line)?;
            }
        }
    }

    
    temp_file.seek(SeekFrom::Start(0))?;
    // Copy the content from the temporary file to the original file
    io::copy(&mut temp_file, &mut config_file)?;

    
    config_file.set_len(temp_file.seek(SeekFrom::Current(0))?)?;

    // Remove the temporary file
    std::fs::remove_file(&temp_path)?;

    Ok(())
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
                            remove_file_from_index(env::var("index").unwrap().as_str(),&hashed_line)?;
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

// fn read_info_from_commit() -> Option<PathBuf> {
//     let commit = match fs::read_to_string(env::var("heads").unwrap()) {
//         Ok(commit) => commit,
//         Err(_) => return None,
//     };

//     let commit_path = Path::new(env::var("objects").unwrap().as_str()).join(&commit[0..2]).join(&commit[2..]);

//     if commit_path.is_dir() {
//         Some(commit_path)
//     } else {
//         None
//     }
// }

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

    let objects_dir = env::var("objects").unwrap_or_else(|_| String::from(".gitrs/objects"));
    fs::create_dir(format!("{}/{}",objects_dir,name_dir))?;

    let another_name = &arr.join("");

    let file_path = format!("{}/{}.bin", objects_dir, another_name);

    let file_path_clone = file_path.clone();

    let mut file = File::create(file_path_clone)?;

    for line in &arr {
        file.write_all(line.as_bytes())?;
    }

    let _ = add_to_index(&arr,&file_path);

    Ok(())
}

fn add_to_index(arr: &Vec<String>,file_path:&str) -> io::Result<()> {

    let mut index_file = OpenOptions::new().create(true).append(true).open(env::var("index").unwrap())?;

    for line in arr {
        let index_line = format!("{} 100644 {}",line,file_path);

        index_file.write_all(index_line.as_bytes())?;
    }

    Ok(())
}

fn check_if_in_index(file_path: &str) -> IndexCheckResult {
    let index_file = match File::open(env::var("index").unwrap()) {
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

#[derive(Debug)]
enum Config {
    Author(String),
    Email(String),
}

fn read_info_from_config() -> io::Result<Vec<Config>> {
    let config_file_path = env::var("config").unwrap();
    let config_file = File::open(&config_file_path)?;
    let reader = BufReader::new(config_file);

    let mut configs = Vec::new();

    for line in reader.lines() {
        let line_content = line?;

        // Using regex to match "user.email", "user.name", "global user.email", "global user.name"
        if let Some(captures) = regex::Regex::new(r#"^(user\.email|user\.name|global user\.email|global user\.name)\s+(.+)$"#)
            .unwrap()
            .captures(&line_content)
        {
            let property = captures.get(1).unwrap().as_str();
            let value = captures.get(2).unwrap().as_str();

            match property {
                "user.email" => {
                    let config = Config::Email(value.to_string());
                    configs.push(config);
                }
                "user.name" => {
                    let config = Config::Author(value.to_string());
                    configs.push(config);
                }
                "global user.email" => {
                    let config = Config::Email(value.to_string());
                    configs.push(config);
                }
                "global user.name" => {
                    let config = Config::Author(value.to_string());
                    configs.push(config);
                }
                _ => {
                    // Handle other cases if needed...
                }
            }
        }
    }

    Ok(configs)
}

pub fn create_commit( commit_message: &str) {
    let config = match read_info_from_config() {
        Ok(config) => config,
        Err(err) => {
            eprintln!("Error reading config: {:?}", err);
            return;
        }
    };

    
    let author = match config.iter().find(|c| matches!(c, Config::Author(_))) {
        Some(Config::Author(author)) => author.clone(),
        _ => {
            eprintln!("Error: Author information not found in the configuration.");
            return;
        }
    };

    let email = match config.iter().find(|c| matches!(c, Config::Email(_))) {
        Some(Config::Email(email)) => email.clone(),
        _ => {
            eprintln!("Error: Email information not found in the configuration.");
            return;
        }
    };

    let base_directory = match env::current_dir() {
        Ok(current_dir) => current_dir.to_string_lossy().into_owned(),
        Err(err) => {
            eprintln!("Error getting current directory: {:?}", err);
            return;
        }
    };

    
    let tree_hash = match create_tree_from_index(&base_directory) {
        Ok(tree) => hash_tree(tree),
        Err(err) => {
            eprintln!("Error creating tree: {:?}", err);
            return;
        }
    };

    
    match create_commit_object(&author, email.as_str(), &tree_hash, commit_message) {
        Ok(()) => println!("Commit created successfully."),
        Err(err) => {
            eprintln!("Error creating commit: {:?}", err);
            return;
        }
    }
}





fn read_from_refs(path:String) -> io::Result<String> {
    
    let contents = fs::read_to_string(path);
    contents
    
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

    let objects_path = env::var("objects").unwrap();
    let commit_file_path = format!("{}/{}.bin",objects_path,commit_hash);
    let mut commit_file = File::create(&commit_file_path)?;
    commit_file.write_all(commit_content.as_bytes())?;
    write_to_current_branch(commit_hash);
    Ok(())
}

fn write_to_current_branch(commit_hash:String) {
    let path = read_head();

    let mut branch = File::create(path.unwrap()).unwrap();

    branch.write_all(commit_hash.as_bytes());

}

fn create_tree_from_index(base_directory: &str) -> io::Result<TreeEntry> {
    let mut tree_entries = Vec::new();

    
    let index_file = File::open(env::var("index").unwrap().as_str())?;
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

                if let Some(TreeEntry::Tree { entries: _, .. }) = subdirectory_entry {
                    let remaining_path = name.split('/').skip(1).collect::<Vec<&str>>().join("/");
                    create_tree_entry(&remaining_path, mode.to_owned(), hash, name)?
                } else {
                    let tree_entry = create_tree_entry(name, mode.to_owned(), hash, name)?;
                    TreeEntry::Tree { mode: mode.to_string(), hash: hash.to_string(), name: subdirectory.to_string(), entries: vec![tree_entry] }
                }
            } else {
                
                create_tree_entry(name, mode.to_owned(), hash, name)?
            };

            tree_entries.push(entry);
        }
    }

    Ok(TreeEntry::Tree { mode: "040000".to_owned(), hash: "".to_string(), name: base_directory.to_string(), entries: tree_entries })
}

fn create_tree_entry(name: &str, mode:  String, hash: &str, full_path: &str) -> io::Result<TreeEntry> {
    let path_parts: Vec<&str> = full_path.split('/').collect();
    if path_parts.len() == 1 {
        
        Ok(TreeEntry::Blob { mode: mode.to_string(), hash: hash.to_string(), name: name.to_string() })
    } else {
        
        let subdirectory_name = path_parts[0];
        let remaining_path = path_parts[1..].join("/");
        let subdirectory_entry = create_tree_entry(name, mode, hash, &remaining_path)?;
        Ok(TreeEntry::Tree { mode: "040000".to_owned(), hash: "".to_string(), name: subdirectory_name.to_string(), entries: vec![subdirectory_entry] })
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
        TreeEntry::Blob { mode: _, hash: _, name: _ } => todo!(),
    }
}

fn hash_string(input: &str) -> String {
    let mut hasher = Sha1::new();
    hasher.update(input);
    let result = hasher.finalize();
    format!("{:x}", result)
}

fn read_head() -> io::Result<String> {
    
    let path = format!("{}", env::var("heads").unwrap());
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    let mut branch_path = String::new();

    for line in reader.lines() {
        let line = line?;
        
        
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() >= 2 {
            branch_path = parts[1].to_string();
            break; 
        }
    }

    Ok(branch_path)
}


fn rebuild_tree(hash: String) -> io::Result<Option<TreeEntry>> {
    match read_object(&hash) {
        Ok(tree) => Ok(Some(tree)),
        Err(e) => {
            if e.kind() == io::ErrorKind::NotFound {
                Ok(None)
            } else {
                Err(e)
            }
        }
    }
}

fn read_object(hash: &str) -> io::Result<TreeEntry> {

    let objects_path = env::var("objects").unwrap();
    let path = format!("{}/{}.bin",objects_path, hash);
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
                        create_tree_entry(name, mode.to_owned(), hash, &remaining_path.to_owned())?
                    } else {
                        let tree_entry = create_tree_entry(name, mode.to_owned(), hash, name)?;
                        TreeEntry::Tree { mode: mode.to_string(), hash: hash.to_string(), name: subdirectory.to_string(), entries: vec![tree_entry] }
                    }
                } else {
                    create_tree_entry(name, mode.to_owned(), hash, name)?
                };
                tree_entries.push(entry);
            }
        }
        Ok(TreeEntry::Tree { mode: "040000".to_owned(), hash: "".to_string(), name: "".to_string(), entries: tree_entries })
    } else {
        Ok(TreeEntry::Blob { mode: "100644".to_owned(), hash: "".to_string(), name: "".to_string() })
    }
}
