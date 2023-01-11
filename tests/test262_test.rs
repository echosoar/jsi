use std::{path::{Path, PathBuf}, env, fs};

#[test]
fn test_all_262() {
    let dir = get_262_path();
    let paths = fs::read_dir(dir).unwrap();
    let names = paths.filter_map(|entry| {
        entry.ok().and_then(|e|
            e.path().file_name()
            .and_then(|n| n.to_str().map(|s| String::from(s)))
        )
    }).collect::<Vec<String>>();
    println!("paths {:?}", names);
}

fn get_262_path() -> PathBuf {
    Path::new(
        &env::current_dir().unwrap()
    ).join("tests/test262/src")
}