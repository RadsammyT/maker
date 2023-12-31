use std::{
    collections::HashMap,
    env, fs,
    io::{self, ErrorKind},
    panic, path,
    process::Command,
};
#[derive(Debug)]
pub struct LaSingleton {
    input_files: Vec<String>,
    output_dir: String,
    set_config: String,
    configs: Vec<MakerConfig>,
}
impl LaSingleton {
    pub fn init() -> LaSingleton {
        LaSingleton {
            input_files: Vec::new(),
            output_dir: String::from("bin"),
            configs: Vec::new(),
            set_config: String::from("__DEFAULT__"),
        }
    }
    pub fn get_config(&mut self) -> io::Result<()> {
        let path = path::Path::new(".maker");
        if !path.exists() {
            panic!(".maker not found!");
        }
        let file = fs::read_to_string(".maker")?;
        let mut temp_config: MakerConfig = MakerConfig::default();
        let mut config_string = String::from("__DEFAULT__");
        if cfg!(debug_assertions) {
            dbg!(&file);
        }
        for mut line in file.lines() {
            line = line.trim_start();
            if let Some(c) = line.find('#') {
                line = line.split_at(c).0;
            }
            {
                if line.starts_with("extension") {
                    line = line.trim_start_matches("extension");
                    temp_config.extensions = split_string(line.to_string());
                }

                if line.starts_with("config") {
                    config_string = line
                        .trim_start_matches("config")
                        .trim_start()
                        .trim_end()
                        .to_string();
                }

                if line.starts_with("format") {
                    temp_config
                        .configs
                        .entry(config_string.clone())
                        .or_insert(line.trim_start_matches("format ").trim_end().to_string());
                    config_string = "__DEFAULT__".to_owned();
                }

                if line.starts_with("push") {
                    self.configs.push(temp_config.clone());
                    temp_config.clear();
                }
            }
        }

        Ok(())
    }
    pub fn parse_args(&mut self) -> Result<(), ErrorKind> {
        let mut state: ArgsParseState = ArgsParseState::Input;
        let mut args: Vec<String> = env::args().collect();
        if args.len() == 1 {
            return Err(ErrorKind::Other);
        }
        args.remove(0);
        for i in args {
            if i == "-o" {
                state = ArgsParseState::Output;
                continue;
            }
            if i == "-c" {
                state = ArgsParseState::Config;
                continue;
            }
            if i == "--help" {
                return Err(ErrorKind::Other);
            }
            match state {
                ArgsParseState::Input => {
                    self.input_files.push(i);
                }
                ArgsParseState::Output => {
                    self.output_dir = i;
                    state = ArgsParseState::Input;
                }
                ArgsParseState::Config => {
                    self.set_config = i;
                    state = ArgsParseState::Input;
                }
            }
        }
        Ok(())
    }
    fn find_config(&mut self, extension: &str) -> Option<MakerConfig> {
        let mut ret: Option<MakerConfig> = None;
        self.configs.iter().for_each(|i| {
            i.extensions.iter().for_each(|j| {
                if extension.ends_with(j) {
                    ret = Some(i.clone());
                }
            });
        });
        ret
    }
    pub fn execute(&mut self) -> Result<(), ErrorKind> {
        self.input_files.clone().into_iter().for_each(|i| {
            let config = self
                .find_config(
                    i.split_at(i.find('.').unwrap_or_else(|| {
                        println!("ERROR: tried to split \"{}\" via '.' but failed!", i);
                        std::process::exit(1);
                    }))
                    .1,
                )
                .unwrap_or_else(|| {
                    println!("ERROR: tried to find command for {} but failed!", i);
                    std::process::exit(1);
                });
            let output_file = i.split_at(i.find('.').unwrap()).0;
            let format = config
                .configs
                .get(&self.set_config)
                .unwrap_or_else(|| -> &String {
                    println!("ERROR! Config not found: {}", self.set_config);
                    std::process::exit(1);
                })
                .clone()
                .replace("%file%", i.as_str())
                .replace(
                    "%output%",
                    format!("{}/{}", self.output_dir, output_file).as_str(),
                );
            println!("---{}---", i);
            match fs::create_dir(self.output_dir.clone()) {
                _ => {}
            }
            let mut format_split = format.split_whitespace();
            let mut com = Command::new(format_split.next().unwrap());
            for arg in format_split {
                com.arg(arg);
            }
            com.spawn().expect(format!("ERROR!\n{}\n", format).as_str());
        });
        Ok(())
    }
    pub(crate) fn debug(&self) {
        dbg!(&self);
    }
}

enum ArgsParseState {
    Input,
    Output,
    Config,
}

#[derive(Debug, Clone)]
struct MakerConfig {
    extensions: Vec<String>,
    configs: HashMap<String, String>, // or, config to format.
}
impl Default for MakerConfig {
    fn default() -> Self {
        Self {
            extensions: Default::default(),
            configs: Default::default(),
        }
    }
}
impl MakerConfig {
    fn clear(&mut self) {
        self.extensions.clear();
        self.configs.clear();
    }
}

fn split_string(mut inp: String) -> Vec<String> {
    let mut buf = String::new();
    let mut ret: Vec<String> = Vec::new();
    inp.push(' ');
    for i in inp.chars() {
        if i.is_whitespace() && !buf.is_empty() {
            ret.push(buf.clone());
            buf.clear();
        }
        if i.is_alphanumeric() {
            buf.push(i);
        }
    }
    ret
}
