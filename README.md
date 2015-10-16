
Spring cleaning tool for git repos.

## Usage

With no arguments `git springclean` will recurse starting at the current
directory and look for all git repositories. For each repository it will run a
series of checks and print a one line summary in the format

    XXXX /path/to/repo

Where X is one of

- (M) changes in the tree
- (U) untracked files in the tree
- (P) a branch was not pushed to any remote
- (E) means there was an unrecoverable error (check stderr)

To point to a specific repository

    git springclean path/to/repo

## Build

With cargo, simply

    $ cargo build


