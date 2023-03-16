use std::{path::{Path}, env, fs::{self, File}, io::{Write, Read}};
use std::fs::metadata;
use std::collections::HashMap;
use jsi::JSI;
use serde::{Serialize, Deserialize};
 
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
    pub fn run(&mut self) {
        let (dirs, files) = self.get_childs();
        self.cases += files.len();
        for file in files.iter() {
            let mut jsi = JSI::new();
            let result = jsi.run(file.code.clone());
            println!("result: {:?}", result);
            self.result.files.insert(file.name.clone(), false);
        }
        for dirs in dirs.iter() {
            let mut dirs_info = dirs.clone();
            dirs_info.run();
            self.cases += dirs_info.cases;
            self.passed += dirs_info.passed;
            self.result.dirs.insert(dirs_info.name.clone(), dirs_info.result);
        }
        self.result.cases = self.cases;
        self.result.passed = self.passed;
    }

    fn get_childs(&self) -> (Vec<Test262Dir>, Vec<Test262File>) {
        let dir = Path::new(
            &env::current_dir().unwrap()
        ).join(&self.dir);
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
            let md = metadata(&abso_name).unwrap();
            if md.is_dir() {
                dirs.push(Test262Dir::new(name.clone(), String::from(abso_name.to_str().unwrap())))
            } else {
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
    
}
impl Test262File {
    pub fn new(name: String, path: String) -> Test262File {
        let mut file = File::open(&path).unwrap();
        let mut code = String::new();
        file.read_to_string(&mut code).unwrap();
        return Test262File {
            name,
            code,
        }
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

#[test]
fn test_all_262() {
       let mut test262 = Test262Dir::new(String::from("base"), String::from("tests/test262/test"));
       test262.run();
       let serialized_result = serde_json::to_string_pretty(&test262.result).unwrap();
       let file_name = ".262/result.json";
        let mut file = File::create(file_name).unwrap();
        file.write_all(serialized_result.as_bytes()).unwrap();
       println!("result: {:?}/{:?}", test262.passed, test262.cases)
}
/*

装载一些测试工具 harness
+ harness/assert.js
+ harness/sta.js
 */
