use std::{collections::HashSet, fs::{self, File}, io::{self, Read, Write}, path::{Path, PathBuf}};

use snapsafe::config::Config;
use tempfile::{tempdir, NamedTempFile};


pub fn get_password() -> String {
    String::from("password")
}

pub fn write_test_file<P: AsRef<Path>>(path: P, content: &str) {
    let mut file = File::create(path).expect("Failed to create test file");
    writeln!(file, "{}", content).expect("Failed to write to file");
}

pub fn setup_dir() -> PathBuf {
    let dir = tempdir().expect("Failed to create temp dir");

    dir.path().to_path_buf()
}

pub fn setup_file_dirs() -> (PathBuf, PathBuf) {
    let source_dir = setup_dir();
    let output_dir = setup_dir();

    let file1_path = source_dir.join("file1.txt");
    let file2_path = source_dir.join("logs").join("file2.log");

    fs::create_dir_all(file2_path.parent().unwrap()).unwrap();
    write_test_file(file1_path, "This is the content of file1");
    write_test_file(file2_path, "This is a log in file2. What?");

    (source_dir, output_dir)
}

fn collect_files(dir: &Path) -> io::Result<HashSet<PathBuf>> {
    let mut files = HashSet::new();
    for entry in walkdir::WalkDir::new(dir).into_iter().filter_map(Result::ok) {
        if entry.file_type().is_file() {
            let rel_path = entry.path().strip_prefix(dir).unwrap().to_path_buf();
            files.insert(rel_path);
        }
    }
    Ok(files)
}

pub fn get_test_registry() -> String {
    let unique_path = NamedTempFile::new().unwrap();
    let unique_path = unique_path.path();

    unique_path.to_string_lossy().to_string()
}

pub fn get_test_configs() -> Config {
    todo!("Include a config definition for temp files to used in integration tests");
}

pub fn clear_test_registry(path: &str) {
    let temp_path = Path::new(path);
    if temp_path.exists() {
        let _ = fs::remove_dir_all(temp_path);
    }
}

/// Compare two directories for identical file structure and content
pub fn compare_dirs(dir1: PathBuf, dir2: PathBuf) -> io::Result<bool> {
    let files1 = collect_files(&dir1)?;
    let files2 = collect_files(&dir2)?;

    if files1 != files2 {
        return Ok(false); // file presence mismatch
    }

    for rel_path in files1 {
        let path1 = dir1.join(&rel_path);
        let path2 = dir2.join(&rel_path);

        let mut file1 = fs::File::open(path1)?;
        let mut file2 = fs::File::open(path2)?;

        let mut buf1 = Vec::new();
        let mut buf2 = Vec::new();

        file1.read_to_end(&mut buf1)?;
        file2.read_to_end(&mut buf2)?;

        if buf1 != buf2 {
            return Ok(false); // file content mismatch
        }
    }

    Ok(true)
}

pub fn copy_dir_contents(from: &Path, to: &Path) -> io::Result<()> {
    if !to.exists() {
        fs::create_dir_all(to)?;
    }

    for entry in fs::read_dir(from)? {
        let entry = entry?;
        let file_type = entry.file_type()?;
        let from_path = entry.path();
        let to_path = to.join(entry.file_name());

        if file_type.is_dir() {
            copy_dir_contents(&from_path, &to_path)?;
        } else {
            fs::copy(&from_path, &to_path)?;
        }
    }

    Ok(())
}
