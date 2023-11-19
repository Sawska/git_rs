use std::path::PathBuf;
use std::{ops::Deref, env};
use std::io;

use structopt::StructOpt;
use git_worker::{init,add_all,create_commit,set_user_input,checkout,checkout_b,list_branches,delete_brench};

#[derive(Debug,StructOpt)]
pub enum Command {
    #[structopt(name = "init")]
     Init {
    },
    #[structopt(name = "add")]
    Add {
        #[structopt(parse(from_os_str), help = "Specify the path")]
        path: Option<PathBuf>,
        #[structopt(flatten)]
        opts: AddOptions,
    },
    #[structopt(name = "reset")]
    Reset {
        
    },
    #[structopt(name = "commit")]
    Commit {
        #[structopt(flatten)]
        opts:CommitOpts,
    },
    #[structopt(name = "status")]
    Status {
        
    },
    #[structopt(name = "diff")]
    Diff {
        first_hash: String,
    },
    #[structopt(name = "branch")]
    Branch {
        name:String,
        #[structopt(flatten)]
        opts: BranchOpts,
    },
    #[structopt(name = "checkout")]
    Checkout {
        branch:String,
        #[structopt(flatten)]
        opts:CheckoutOptions,
    },
    #[structopt(name = "stash")]
    Stash {
        
    },
    #[structopt(name = "config")]
    Config {
        #[structopt(flatten)]
        opts: ConfigOptions
    }
    
}

#[derive(Debug,StructOpt)]
pub struct  BranchOpts {
    #[structopt( long = "--list", help = "Show all branches")]
    pub list: bool,
    #[structopt(short="d", long = "delete", help = "Delete brahch",takes_value = true)]
    pub delete: Option<String>,
}
#[derive(Debug,StructOpt)]
pub struct ConfigOptions {
    #[structopt(long = " user.email",help ="sets email", takes_value = true)]
    pub email:Option<String>,
    #[structopt(long = "--global user.email",help ="sets email", takes_value = true)]
    pub global_email:Option<String>,
    #[structopt(long = "--global user.name",help ="sets name", takes_value = true)]
    pub global_name:Option<String>,
    #[structopt(long = "user.name",help ="sets name", takes_value = true)]
    pub name:Option<String>,
}

#[derive(Debug,StructOpt)]
pub struct  CommitOpts {
    #[structopt(short="m",long = "message",help="user types message")]
    pub message:Option<String>
}

#[derive(Debug,StructOpt)]
pub struct  StashOpts {
    #[structopt(long = "apply",help="aplly without deliting")]
    pub apply:bool,
    #[structopt(long = "pop",help="aplly saved")]
    pub pop:bool,
}

#[derive(Debug,StructOpt)]
pub struct AddOptions {
    #[structopt(short="a",long="all",help ="adds all files")]
    pub all:bool,
}

#[derive(Debug,StructOpt)]
pub struct  CheckoutOptions {
    #[structopt(short="b",help ="adds new branch", takes_value = true)]
    pub branch_name:Option<String>
}

#[derive(Debug,StructOpt)]
#[structopt(name = "git_rs",about = "git written rust")]
pub struct  Opt {
    #[structopt(subcommand)]
    pub command:Command,
}



pub fn use_command() {
    let opt = Opt::from_args();

    match opt.command {
            Command::Init { } => {
                init();
            }
        Command::Add { path ,opts} =>  {
            
            let target_path = if opts.all {
                None // Add all files in the current directory
            } else {
                path
            };

            
            match target_path {
                Some(p) => add_all(p.to_str().unwrap()), 
                None => add_all("."),
            };
            
        },
        Command::Reset {  } => {
            // reset();
        },
        Command::Commit { opts } => {

            let message = opts.message.unwrap();
            if message != "".deref() {
                    create_commit( &message);
            } else {
                let message_user = user_not_specified_message();
                create_commit( &message_user);

            }

        },
        Command::Status {  } => {
            not_implemented();
        }
        Command::Diff { first_hash } => todo!(),
        Command::Branch { opts, name } => {
            let name_br = opts.delete.unwrap();
            if name_br != "" {
                delete_brench(&name_br);
            } else if opts.list {
                let res = list_branches();
                println!("{}",res);
            } else if name != "" {
                checkout_b(&name);
            }
        },
        Command::Checkout { branch, opts } => {
            let branch_name = opts.branch_name.unwrap();

            if branch_name != "" {
                checkout_b(&branch_name);
            } else {
                checkout(&branch);
            }
        },
        Command::Stash {  } => todo!(),
        Command::Config { opts } => {
            let result = match opts {
                ConfigOptions {
                    email: Some(email),
                    global_email: _,
                    name: _,
                    global_name: _,
                } => set_user_input("user.email", &email),
                ConfigOptions {
                    email: _,
                    global_email: Some(global_email),
                    name: _,
                    global_name: _,
                } => set_user_input("global user.email", &global_email),
                ConfigOptions {
                    email: _,
                    global_email: _,
                    name: Some(name),
                    global_name: _,
                } => set_user_input("user.name", &name),
                ConfigOptions {
                    email: _,
                    global_email: _,
                    name: _,
                    global_name: Some(global_name),
                } => set_user_input("global user.name", &global_name),
                _ => Ok(()), 
            };
            if let Err(err) = result {
                eprintln!("Error setting user input: {:?}", err);
            }
        }
        
     }
}

fn user_not_specified_message() -> String {
    println!("Enter commit message");

    
    let mut user_input = String::new();

    
    io::stdin().read_line(&mut user_input).expect("Failed to read line");

    user_input

}

 fn not_implemented() {
    println!("not implemented");
}

