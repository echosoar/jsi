use std::{path::{Path, PathBuf}, env, fs::{self, File}, io::{Write, Read}, panic};
use std::fs::metadata;
use std::collections::HashMap;
use jsi::JSI;
use serde::{Serialize, Deserialize};
use yaml_rust::{YamlLoader, Yaml};
 
#[derive(Clone)]
struct Test262Dir {
    pub name: String,
    pub dir: String,
    pub cases: usize,
    pub passed: usize,
    pub result: Test262DirResult,
}

impl Test262Dir {
    pub fn new(name: String, dir: String) -> Test262Dir {
        return Test262Dir {
            name,
            dir,
            cases: 0,
            passed: 0,
            result: Test262DirResult::new(),
        }
    }
    pub fn run(&mut self, preload_code: &str, ignore_list: &Vec<PathBuf>, only_list: &Vec<PathBuf>) {
        let (dirs, files) = self.get_childs(ignore_list, only_list);
        self.cases += files.len();
        for file in files.iter() {
            let mut passed = false;
            let result = panic::catch_unwind(|| {
                let mut jsi = JSI::new();
                jsi.set_strict(!file.no_strict);
                // println!("run: {:?}", code);
                let result = jsi.run(format!("{}\n{}", preload_code, file.code));
                if only_list.len() > 0 {
                    println!("result: {:?}", result);
                }
                return result;
            });
            if result.is_err() {
                println!("panic: {:?} {:?}", file.name, file.code);
                passed = false;
            } else {
                if let Ok(inner_result) = result {
                    // println!("inner_result {:?}", inner_result);
                    if let Err(jsi_error) = inner_result {
                        if file.negative {
                            let error = jsi_error.error_type.to_string();
                            if file.negative_type.len() > 0 && error != file.negative_type {
                                println!("negative error type: {:?} {:?} {:?}", file.name, file.negative_type, error);
                                passed = false;
                            } else {
                                passed = true;
                            }
                        } else {
                            passed = false;
                        }
                    } else {
                        passed = !file.negative;
                    }
                }
            }
            if passed {
                self.passed += 1;
            }
            self.result.files.insert(file.name.clone(), passed);
        }
        for dirs in dirs.iter() {
            let mut dirs_info = dirs.clone();
            dirs_info.run(preload_code, ignore_list, only_list);
            self.cases += dirs_info.cases;
            self.passed += dirs_info.passed;
            self.result.dirs.insert(dirs_info.name.clone(), dirs_info.result);
        }
        self.result.cases = self.cases;
        self.result.passed = self.passed;
    }

    fn get_childs(&self,  ignore_list: &Vec<PathBuf>, only_list: &Vec<PathBuf>) -> (Vec<Test262Dir>, Vec<Test262File>) {
        let dir = make_dir(&self.dir);
        let paths = fs::read_dir(&dir).unwrap();
        let names = paths.filter_map(|entry| {
            entry.ok().and_then(|e|
                e.path().file_name()
                .and_then(|n| n.to_str().map(|s| String::from(s)))
            )
        }).collect::<Vec<String>>();
        let mut dirs: Vec<Test262Dir> = vec![];
        let mut files: Vec<Test262File> = vec![];
        for name in names.iter() {
            let abso_name = dir.join(&name);
            if ignore_list.contains(&abso_name) {
                continue;
            }
            let md = metadata(&abso_name).unwrap();
            if md.is_dir() {
                dirs.push(Test262Dir::new(name.clone(), String::from(abso_name.to_str().unwrap())))
            } else {
                if only_list.len() > 0 {
                    let mut is_ignore = true;
                    for only in only_list.iter() {
                        if abso_name.starts_with(&only) {
                            is_ignore = false
                        }
                    }
                    if is_ignore {
                        continue;
                    }
                }
                if name.ends_with(".js") {
                    files.push(Test262File::new(name.clone(), String::from(abso_name.to_str().unwrap())))
                }
            }
        }
        return (dirs, files)
    }
}



#[derive(Clone)]
struct Test262File {
    pub name: String,
    pub code: String,
    pub no_strict: bool,
    pub negative: bool,
    pub negative_type: String,
    
}
impl Test262File {
    pub fn new(name: String, path: String) -> Test262File {
        let mut file = File::open(&path).unwrap();
        let mut code = String::new();
        file.read_to_string(&mut code).unwrap();
        let config = Test262File::parse(&code);
        return Test262File {
            name,
            code,
            no_strict: config.0,
            negative: config.1,
            negative_type: config.2
        }
    }

    pub fn parse(code: &String) -> (bool, bool, String) {
        let mut no_strict = false;
        let mut negative = false;
        let mut negative_type = String::from("");
        let start = code.find("/*---");
        if let Some(start) = start {
            let end = code.find("---*/");
            if let Some(end) = end {
                let config = &code[start + 5..end];
                let docs = YamlLoader::load_from_str(config);
                if let Ok(docs) = docs {
                    
                    if let Yaml::Array(arr) = &docs[0]["flags"] {
                        for flag in arr.iter() {
                            if let Some(str) = flag.as_str() {
                                match str {
                                    "noStrict" => {
                                        no_strict = true;
                                    },
                                    _ => {}
                                }
                            }
                        }
                    }

                    if let Yaml::BadValue = docs[0]["negative"] {
                    
                    } else {
                        negative = true;
                        let negative_type_value = docs[0]["negative"]["type"].as_str();
                        if let Some(negative_type_item) = negative_type_value {
                            negative_type = String::from(negative_type_item);
                        }
                    }
                }
            }
        }
        return (no_strict, negative, negative_type);
    }
}

#[derive(Clone,Serialize, Deserialize, Debug)]
struct Test262DirResult {
    pub cases: usize,
    pub passed: usize, 
    pub dirs: HashMap<String,Test262DirResult>,
    pub files: HashMap<String,bool>,
}
impl Test262DirResult {
    pub fn new() -> Test262DirResult {
        return Test262DirResult {
            cases: 0,
            passed: 0,
            dirs: HashMap::new(),
            files: HashMap::new(),
        }
    }
}


fn load_harness(path: &str) -> String {
    let mut file = File::open(format!("test262/{}", path)).unwrap();
    let mut code = String::new();
    file.read_to_string(&mut code).unwrap();
    return code;
}

fn make_dir(dir: &String) -> PathBuf {
    Path::new(
        &env::current_dir().unwrap()
    ).join(dir)
}

#[test]
fn test_all_262() {
    let preload_list = vec![
        load_harness("harness/assert.js"),
        load_harness("harness/sta.js"),
        load_harness("harness/compareArray.js"),
    ];
    let prelaod = preload_list.join("\n");
    let ignore_list: Vec<PathBuf> =vec![
        make_dir(&String::from("test262/test/annexB")),
        make_dir(&String::from("test262/test/intl402")),
    ];
    let only_list: Vec<PathBuf> =vec![
        // make_dir(&String::from("test262/test/built-ins/String/prototype/charAt/pos-coerce-err.js")),
    ];
    let mut test262 = Test262Dir::new(String::from("base"), String::from("test262/test"));
    test262.run(prelaod.as_str(), &ignore_list, &only_list);
    let serialized_result = serde_json::to_string(&test262.result).unwrap();
    let file_name = "./tests/262_result.json";
    let mut file = File::create(file_name).unwrap();
    file.write_all(serialized_result.as_bytes()).unwrap();
    println!("result: passed {:?} / total {:?} at {}", test262.passed, test262.cases, file_name);
    if only_list.len() > 0 {
        return;
    }
    let old_file_name = "./tests/262_result_data.json";
    let old_exists = Path::new(old_file_name).exists();
    if old_exists {
        let mut diff = ResultDiff {
            passed: vec![],
            failed: vec![],
            add: vec![],
            remove: vec![],
        };
        let json_old = read_json(old_file_name);
        let json_new = read_json(file_name);
        check_diff(&mut diff, String::from("test262/test"), &json_old, &json_new);
        let serialized_result = serde_json::to_string_pretty(&diff).unwrap();
        let diff_result_file_name = "./tests/262_result_diff.json";
        let mut file = File::create(diff_result_file_name).unwrap();
        file.write_all(serialized_result.as_bytes()).unwrap();
        println!("result diff: passed {:?} / failed {:?} / add {:?} / remove {:?} at {}", diff.passed.len(), diff.failed.len(), diff.add.len(), diff.remove.len(), diff_result_file_name);
    }
    
}

fn read_json(file_path: &str) -> serde_json::Value {
    let file = File::open(file_path).unwrap();
    serde_json::from_reader(file).unwrap()
}

#[derive(Debug, Serialize)]
struct ResultDiff {
    passed: Vec<String>,
    failed: Vec<String>,
    add: Vec<String>,
    remove: Vec<String>,
}

fn check_diff(diff: &mut ResultDiff, path: String, old: &serde_json::Value, new: &serde_json::Value) {
    let old_dirs = old.get("dirs").unwrap();
    let new_dirs = new.get("dirs").unwrap();
    let old_dir_list = old_dirs.as_object().unwrap();
    let new_dir_list = new_dirs.as_object().unwrap();
    for old_dir in old_dir_list.iter() {
        let dir_name = old_dir.0;
        let new_dir = new_dir_list.get(dir_name);
        let dir_path = format!("{}/{}", path, dir_name);
        if let Some(new_dir_value) = new_dir {
            check_diff(diff, dir_path, old_dir.1, new_dir_value);
        } else {
            // old exists but new not exists
            diff.remove.push(dir_path);
        }
    }

    for new_dir in new_dir_list.iter() {
        let dir_name = new_dir.0;
        let old_dir = old_dir_list.get(dir_name);
        let dir_path = format!("{}/{}", path, dir_name);
        if let None = old_dir {
            diff.add.push(dir_path);
        }
    }

    let old_files = old.get("files").unwrap();
    let new_files = new.get("files").unwrap();
    let old_files_list = old_files.as_object().unwrap();
    let new_files_list = new_files.as_object().unwrap();

    for old_file in old_files_list.iter() {
        let file_name = old_file.0;
        let new_file = new_files_list.get(file_name);
        let file_path = format!("{}/{}", path, file_name);
        if let Some(new_file_value) = new_file {
            let old_value_bool = old_file.1.as_bool().unwrap();
            let new_value_bool = new_file_value.as_bool().unwrap();
            if old_value_bool && !new_value_bool {
                diff.failed.push(file_path);
            } else if !old_value_bool && new_value_bool {
                diff.passed.push(file_path);
            }
        } else {
            // old exists but new not exists
            diff.remove.push(file_path);
        }
    }

    for new_file in new_files_list.iter() {
        let file_name = new_file.0;
        let old_file = old_files_list.get(file_name);
        let file_path = format!("{}/{}.js", path, file_name);
        if let None = old_file {
            diff.add.push(file_path);
        }
    }

}