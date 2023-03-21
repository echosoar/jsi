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
                // println!("run: {:?}", code);
                let result = jsi.run(format!("{}\n{}", preload_code, file.code));
                // println!("result: {:?}", result);
                return result;
            });
            if result.is_err() {
                println!("panic: {:?} {:?}", file.name, file.code);
                passed = false;
            } else {
                if let Ok(inner_result) = result {
                    println!("inner_result {:?}", inner_result);
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
            negative: config.0,
            negative_type: config.1
        }
    }

    pub fn parse(code: &String) -> (bool, String) {
        let mut negative = false;
        let mut negative_type = String::from("");
        let start = code.find("/*---");
        if let Some(start) = start {
            let end = code.find("---*/");
            if let Some(end) = end {
                let config = &code[start + 5..end];
                let docs = YamlLoader::load_from_str(config);
                if let Ok(docs) = docs {
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
        return (negative, negative_type);
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
        make_dir(&String::from("test262/test/language/computed-property-names/to-name-side-effects/numbers-object.js")),
    ];
    let mut test262 = Test262Dir::new(String::from("base"), String::from("test262/test"));
    test262.run(prelaod.as_str(), &ignore_list, &only_list);
    let serialized_result = serde_json::to_string_pretty(&test262.result).unwrap();
    let file_name = "./262_result.json";
    let mut file = File::create(file_name).unwrap();
    file.write_all(serialized_result.as_bytes()).unwrap();
    println!("result: {:?}/{:?}", test262.passed, test262.cases)
}
