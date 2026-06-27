use crate::error::CLIResult;
use std::{
    fs,
    path::{Path, PathBuf},
};

#[cfg(unix)]
use std::os::unix::fs::MetadataExt;

// TODO See FileMeta::new, below
// #[cfg(windows)]
// use std::os::windows::fs::MetadataExt;

enum FileType {
    /// Regular file
    File,

    /// Directory
    Directory,

    /// Something else, which we don't care about
    /// (e.g., FIFOs, sockets, etc.)
    SomethingElse,
}

#[allow(dead_code)]
enum Hardlink {
    // We know there's at least one link, but cannot say more than that
    AtLeastOne,

    // We know exactly how many links there are
    // NOTE While technically possible, I would worry about a file that had 2^64 links
    Exactly(u64),
}

struct FileMeta {
    filetype: FileType,
    symlink: bool,
    hardlink: Hardlink,
}

impl FileMeta {
    #[allow(clippy::result_large_err)]
    fn new<P: AsRef<Path>>(path: &P) -> CLIResult<Self> {
        // Stat a potential symlink
        let lmeta = fs::symlink_metadata(path)?;
        let symlink = lmeta.is_symlink();

        // Follow the symlink, if necessary
        let meta = if symlink { fs::metadata(path)? } else { lmeta };

        let filetype = {
            if meta.is_file() {
                FileType::File
            } else if meta.is_dir() {
                FileType::Directory
            } else {
                FileType::SomethingElse
            }
        };

        #[cfg(unix)]
        let hardlink = Hardlink::Exactly(meta.nlink());

        // TODO Windows has fs::MetadataExt::number_of_links, but this is experimental as of
        // writing (see https://github.com/rust-lang/rust/issues/63010)
        #[cfg(windows)]
        let hardlink = Hardlink::AtLeastOne;

        // Everything else
        #[cfg(not(any(unix, windows)))]
        let hardlink = Hardlink::AtLeastOne;

        Ok(Self {
            filetype,
            symlink,
            hardlink,
        })
    }

    fn ignore(&self) -> bool {
        matches!(self.filetype, FileType::SomethingElse)
    }

    fn is_dir(&self) -> bool {
        matches!(self.filetype, FileType::Directory)
    }

    fn is_file(&self) -> bool {
        matches!(self.filetype, FileType::File)
    }

    fn is_symlink(&self) -> bool {
        self.symlink
    }

    fn has_multiple_links(&self) -> bool {
        matches!(self.hardlink, Hardlink::Exactly(n) if n > 1)
    }
}

/// Given a vector of paths, recursively expand those that identify as directories, in place.
/// Follow symlinks, if specified, and skip over files with multiple links. Ultimately, we'll
/// finish with a vector of canonical paths to real files with a single link.
#[allow(clippy::result_large_err)]
pub fn traverse(files: &mut Vec<PathBuf>, follow_symlinks: bool) -> CLIResult<()> {
    let mut expanded = vec![];

    for file in &mut *files {
        // Using FileMeta means we, at most, stat each file twice
        let meta = match FileMeta::new(file) {
            Ok(meta) => meta,
            Err(_) => {
                log::error!("Skipping {}: Cannot access", file.display());
                continue;
            }
        };

        // Skip over everything we don't care about
        if meta.ignore() {
            // This isn't such an important message, so only warn
            log::warn!("Skipping {}: Not a regular file", file.display());
            continue;
        }

        let is_dir = if follow_symlinks {
            meta.is_dir()
        } else {
            meta.is_dir() && !meta.is_symlink()
        };

        if is_dir {
            // Descend into directory, symlink-aware as required
            let mut subfiles = file.read_dir()?.flatten().map(|f| f.path()).collect();
            traverse(&mut subfiles, follow_symlinks)?;
            expanded.append(&mut subfiles);
        } else if meta.is_file() {
            if meta.is_symlink() && !follow_symlinks {
                // This isn't such an important message, so only warn
                log::warn!(
                    "Skipping {}: File is a symlink; use --follow-symlinks to follow",
                    file.display()
                );
                continue;
            }

            if meta.has_multiple_links() {
                log::error!(
                    "Skipping {}: File has multiple links, which Topiary would break",
                    file.display()
                );
                continue;
            }

            // Only push the file if the canonicalisation succeeds (i.e., skip broken symlinks)
            if let Ok(candidate) = file.canonicalize() {
                expanded.push(candidate);
            } else {
                log::error!(
                    "Skipping {}: File does not exist (e.g., broken symlink)",
                    file.display()
                );
            }
        }
    }

    *files = expanded;
    Ok(())
}
