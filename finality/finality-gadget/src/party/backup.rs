// بِسْمِ اللَّهِ الرَّحْمَنِ الرَّحِيم

// This file is part of DAGESTAN.

// Copyright (C) 2019-Present Setheum Labs.
// SPDX-License-Identifier: GPL-3.0-or-later WITH Classpath-exception-2.0

// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with this program. If not, see <https://www.gnu.org/licenses/>.

use std::{
    fmt, fs,
    fs::File,
    io,
    io::{Cursor, Read, Write},
    path::{Path, PathBuf},
    str::FromStr,
};

use log::{debug, warn};

const BACKUP_FILE_EXTENSION: &str = ".abfts";

#[derive(Debug)]
pub enum BackupLoadError {
    BackupIncomplete(Vec<usize>),
    IOError(io::Error),
}

impl fmt::Display for BackupLoadError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            BackupLoadError::BackupIncomplete(backups) => {
                write!(
                    f,
                    "Backup is not complete. Got backup for runs numbered: {:?}",
                    backups
                )
            }
            BackupLoadError::IOError(err) => {
                write!(f, "Backup could not be loaded because of IO error: {}", err)
            }
        }
    }
}

impl From<io::Error> for BackupLoadError {
    fn from(err: io::Error) -> Self {
        Self::IOError(err)
    }
}

impl std::error::Error for BackupLoadError {}

pub type Saver = Box<dyn Write + Send>;
pub type Loader = Box<dyn Read + Send>;
pub type ABFTBackup = (Saver, Loader);

/// Find all `*.abfts` files at `session_path` and return their indexes sorted, if all are present.
fn get_session_backup_idxs(session_path: &Path) -> Result<Vec<usize>, BackupLoadError> {
    fs::create_dir_all(&session_path)?;
    let mut session_backups: Vec<_> = fs::read_dir(&session_path)?
        .filter_map(|r| r.ok())
        .filter_map(|x| x.file_name().into_string().ok())
        .filter_map(|s| usize::from_str(s.strip_suffix(BACKUP_FILE_EXTENSION)?).ok())
        .collect();
    session_backups.sort_unstable();
    if !session_backups.iter().cloned().eq(0..session_backups.len()) {
        return Err(BackupLoadError::BackupIncomplete(session_backups));
    }
    Ok(session_backups)
}

/// Load session backup at path `session_path` from all `session_idxs`.
fn load_backup(session_path: &Path, session_idxs: &[usize]) -> Result<Loader, BackupLoadError> {
    let mut buffer = Vec::new();
    for index in session_idxs.iter() {
        let load_path = session_path.join(format!("{}{}", index, BACKUP_FILE_EXTENSION));
        File::open(load_path)?.read_to_end(&mut buffer)?;
    }
    Ok(Box::new(Cursor::new(buffer)))
}

/// Get path of next backup file in session.
fn get_next_path(session_path: &Path, session_idxs: &[usize]) -> PathBuf {
    session_path.join(format!(
        "{}{}",
        session_idxs.last().map_or(0, |i| i + 1),
        BACKUP_FILE_EXTENSION,
    ))
}

/// Loads the existing backups, and opens a new backup file to write to.
///
/// `backup_path` is the path to the backup directory (i.e. the argument to `--backup-saving-path`).
///
/// Returns the newly-created file (opened for writing), and the concatenation of the contents of
/// all existing files.
///
/// Current directory structure (this is an implementation detail, not part of the public API):
///   backup-stash/      - the main directory, backup_path/--backup-saving-path
///   `-- 18723/         - subdirectory for the current session
///       |-- 0.abfts    - files containing data
///       |-- 1.abfts    - each restart after a crash will cause another one to be created
///       |-- 2.abfts    - these numbers count up sequentially
///       `-- 3.abfts
pub fn rotate(
    backup_path: Option<PathBuf>,
    session_id: u32,
) -> Result<ABFTBackup, BackupLoadError> {
    debug!(target: "stance-party", "Loading Stance backup for session {:?}", session_id);
    let session_path = if let Some(path) = backup_path {
        path.join(format!("{}", session_id))
    } else {
        debug!(target: "stance-party", "Passing empty backup for session {:?} as no backup argument was provided", session_id);
        return Ok((Box::new(io::sink()), Box::new(io::empty())));
    };
    debug!(target: "stance-party", "Loading backup for session {:?} at path {:?}", session_id, session_path);

    let session_backup_idxs = get_session_backup_idxs(&session_path)?;

    let backup_loader = load_backup(&session_path, &session_backup_idxs)?;

    let next_backup_path = get_next_path(&session_path, &session_backup_idxs);
    debug!(target: "stance-party", "Loaded backup for session {:?}. Creating new backup file at {:?}", session_id, next_backup_path);
    let backup_saver = Box::new(File::create(next_backup_path)?);

    debug!(target: "stance-party", "Backup rotation done for session {:?}", session_id);
    Ok((backup_saver, backup_loader))
}

/// Removes the backup directory for a session.
///
/// `backup_path` is the path to the backup directory (i.e. the argument to `--backup-saving-path`).
/// If it is `None`, nothing is done.
///
/// Any filesystem errors are logged and dropped.
///
/// This should be done after the end of the session.
pub fn remove(path: Option<PathBuf>, session_id: u32) {
    let path = match path {
        Some(path) => path.join(session_id.to_string()),
        None => return,
    };
    match fs::remove_dir_all(&path) {
        Ok(()) => {
            debug!(target: "stance-party", "Removed backup for session {}", session_id);
        }
        Err(err) => {
            if err.kind() != io::ErrorKind::NotFound {
                warn!(target: "stance-party", "Error cleaning up backup for session {}: {}", session_id, err);
            }
        }
    }
}
