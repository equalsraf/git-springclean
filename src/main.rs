extern crate docopt;
extern crate rustc_serialize;

use std::io::Write;
use std::process::exit;
use std::env::current_dir;
use std::path::Path;
use std::fs::read_dir;
use std::fs::metadata;
use docopt::Docopt;

mod checks;
use checks::Args;

const USAGE: &'static str = "
Usage:
    git-springclean [options] [<path>]

Options:
    -A, --all           List all repos, even if they look ok
    -U, --no-untracked  Don't report untracked files
    -M, --no-modified   Don't report modified files
    -P, --no-unpushed   Don't report unpushed branches
    -h, --help          Show this help message
    -V, --version       Display version and exit
    -v, --verbose       Be more verbose
";

macro_rules! unwrap_or_return {
    ($e:expr, $r:expr) => {
        if let Ok(val) = $e { val } else { return $r }
    };
    ($e:expr) => {
        if let Ok(val) = $e { val } else { return }
    };
}

macro_rules! unwrap_or_continue {
    ($e:expr) => {
        if let Ok(val) = $e { val } else { continue }
    };
}

macro_rules! println_stderr(
    ($($arg:tt)*) => (
        match writeln!(&mut ::std::io::stderr(), $($arg)* ) {
            Ok(_) => {},
            Err(x) => panic!("Unable to write to stderr: {}", x),
        }
    )
);

/// Call **f** on all git repositories in path **p**
/// Returns the number of repos where f returned true
fn for_all_git_repos(p: &Path, 
        f: &Fn(&Path, &Args) -> bool, args: &Args) 
        -> i32 {

    let mut count = 0;
    for entry in unwrap_or_return!(read_dir(p), 0) {
        let entry = unwrap_or_continue!(entry);
        if !unwrap_or_continue!(entry.file_type()).is_dir() {
            continue
        }

        if ".git" == &entry.file_name() {
            if !f(p, args) {
                count += 1;
            }
            break;
        } else {
            count += for_all_git_repos(&entry.path(), f, args);
        }
    }
    count
}

/// Check git repository for issues
fn git_repo_ok(p: &Path, args: &Args) -> bool {

    let mut summary = String::new();
    let mut errors = Vec::new();
    let mut verbose = Vec::new();

    for check in checks::ALL_CHECKS {
        match check(p, args) {
            Err(err) => errors.push(err),
            Ok((sum, msg)) => {
                summary.push_str(&sum);
                verbose.push(msg);
            },
        }
    }
    
    if !errors.is_empty() {
        summary.push('E');
    }

    // All done, print summary
    if !summary.is_empty() || args.flag_all {
        println!("{:<4} {}", summary, p.to_string_lossy());

        if args.flag_verbose {
            for msg in verbose.iter().filter(|m| !m.is_empty()) {
                println!("{}", msg);
            }
        }

        let mut errn = 0;
        for err in errors {
            println_stderr!("[Error{}]:{}", errn, err);
            errn += 1;
        }
    }

    summary.is_empty()
}

fn main() {
    let args: Args = Docopt::new(USAGE)
                            .and_then(|d| d.decode())
                            .unwrap_or_else(|e| e.exit());
    if args.flag_version {
        println!("git-springclean {}", env!("CARGO_PKG_VERSION"));
        return;
    }

    let mut path = current_dir()
            .unwrap_or_else(|e| panic!("Cannot determine current directory {}", e));
    path.push(&args.arg_path);

    if let Err(err) = metadata(&path) {
        println_stderr!("Unable to read {}: {}", path.to_string_lossy(), err);
        exit(-1);
    } else {
        // Return the number of repos that failed
        exit(for_all_git_repos(&path, &git_repo_ok, &args));
    }
}

