use structopt::StructOpt;

#[derive(Debug,StructOpt)]
pub enum Command {
    #[structopt(name = "init")]
     Init {
    },
    #[structopt(name = "add")]
    Add {
        #[structopt(default_value=".")]
        path:String,
        #[structopt(flatten)]
        opts:AddOptions,

        //add implementation
    },
    #[structopt(name = "reset")]
    Reset {
        //  //add implementation
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
        #[structopt(flatten)]
        opts: BranchOpts,
    },
    #[structopt(name = "checkout")]
    Checkout {
        branch:String,
        #[structopt(flatten)]
        opts:CheckoutOptions,
    },
    #[structopt(name = "switch")]
    Switch {
        branch:String,
    },
    #[structopt(name = "stash")]
    Stash {
        
    }
    
}

#[derive(Debug,StructOpt)]
pub struct  BranchOpts {
    #[structopt(short = "a", long = "all", help = "Show all branches")]
    pub all: bool,
    #[structopt(short="d", long = "delete", help = "Delete brahch")]
    pub delete: bool
}

#[derive(Debug,StructOpt)]
pub struct  CommitOpts {
    #[structopt(short="a",long = "all",help="Commits all files")]
    pub all:bool,
    #[structopt(short="m",long = "message",help="user types message")]
    pub message:bool,
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
                // init();
            }
        Command::Add { path ,opts} =>  {
            not_implemented();
        },
        Command::Reset {  } => {
            not_implemented();
        },
        Command::Commit { opts } => {
            not_implemented();
        },
        Command::Status {  } => {
            not_implemented();
        }
        Command::Diff { first_hash } => todo!(),
        Command::Branch { opts } => todo!(),
        Command::Checkout { branch, opts } => todo!(),
        Command::Switch { branch  } => todo!(),
        Command::Stash {  } => todo!(),
     }
}


 fn not_implemented() {
    println!("not implemented");
}

