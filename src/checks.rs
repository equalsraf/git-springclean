///!
///! Checks for git repositories, each function returns a slice with
///! chars that indicate different error conditions
///!

use std::process::Command;
use std::path::Path;

#[derive(Debug, RustcDecodable)]
pub struct Args {
    pub flag_version: bool,
    pub flag_all: bool,
    pub flag_no_untracked: bool,
    pub flag_no_modified: bool,
    pub flag_no_unpushed: bool,
    pub arg_path: String,
}

const GIT_TRIM: &'static [char] = &[' ','\r', '\t','*'];

/// Call git branch command to get a list of branches
fn git_branch_list(p: &Path, params: &[&str]) -> Result<Vec<String>,String> {
    let mut cmd = Command::new("git");
    cmd.arg("branch").current_dir(p);
    for param in params {
        cmd.arg(param);
    }
    let output = cmd.output()
                .unwrap_or_else(|e| {panic!("Error running git: {}", e)});
    if !output.status.success() {
        return Err(String::from_utf8_lossy(&output.stderr).to_string());
    }

    let mut res = Vec::new();
    let status = String::from_utf8_lossy(&output.stdout);
    for line in status.lines()
        .map(|l| l.trim_matches(GIT_TRIM))
        .filter(|l| !l.is_empty() && !l.starts_with('(')){
        if let Some(branch) = line.split_whitespace().next() {
            res.push(branch.to_string());
        }
    }

    Ok(res)
}

/// Check if there are untracked (U) or modified (M) files inside
/// the repository - i.e. git status.
pub fn check_untracked_modified<'a>(p: &Path, args: &Args)
        -> Result<&'a str,String> {

    let output = Command::new("git")
                .arg("status")
                .arg("--porcelain")
                .current_dir(p)
                .output()
                .unwrap_or_else(|e| {panic!("Error running git: {}", e)});
    if !output.status.success() {
        return Err(String::from_utf8_lossy(&output.stderr).to_string());
    }

    let mut untracked = false;
    let mut modified = false;
    let status = String::from_utf8_lossy(&output.stdout);
    for line in status.lines().map(|l| l.trim()) {
        if line.starts_with("??") {
            untracked = !args.flag_no_untracked;
        } else {
            modified = !args.flag_no_modified;
        }
    }

    Ok(match (untracked,modified) {
        (false,false) => "",
        (true,false) => "U",
        (false,true) => "M",
        (true,true) => "UM",
    })
}

/// Go through all branches in the repository and make sure all are
/// merged into at least one remote branch (P).
pub fn check_unpushed_branches<'a>(p: &Path, args: &Args)
        -> Result<&'a str,String> {

    if args.flag_no_unpushed {
        return Ok("");
    }

    let mut local_branches = try!(git_branch_list(p, &[]));
    let remote_branches = try!(git_branch_list(p, &["-r"]));

    for remote in remote_branches {
        if local_branches.is_empty() {
            break;
        }
        let merged = try!(git_branch_list(p, &["--merged", &remote]));
        for branch in merged {
            if let Some(pos) = local_branches.iter().position(|e| e == &branch) {
                local_branches.remove(pos);
            }
        }
    }

    if !local_branches.is_empty() {
        Ok("P")
    } else {
        Ok("")
    }
}

pub const ALL_CHECKS: &'static [&'static Fn(&Path, &Args) -> Result<&'static str,String>] = &[
    &check_unpushed_branches,
    &check_untracked_modified,
];

