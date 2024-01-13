use std::{
    collections::HashMap,
    env, fs,
    process::{Command, Child}, path
};
#[derive(Debug)]
pub enum MakerError {
    NotEnoughArgs,
    ParsingError(String),
    ExtensionNotCovered(String),
    ConfigNotFound(String),
    DotMakerNotFound,
    OverrideHelp,
    OverrideMakerCreate,
}
#[derive(Debug)]
pub struct LaSingleton {
    input_files: Vec<String>,
    output_dir: String,
    set_config: String,
    additional_flags: String,
    pub async_commands: bool,
    pub async_processes: Vec<(Child, String)>,
    configs: Vec<MakerConfig>,
}
impl LaSingleton {
    pub fn init() -> LaSingleton {
        LaSingleton {
            input_files: Vec::new(),
            output_dir: String::from("bin"),
            configs: Vec::new(),
            set_config: String::from("__DEFAULT__"),
            async_commands: false,
            async_processes: Vec::new(),
            additional_flags: String::new(),
        }
    }
    pub fn get_config(&mut self) -> Result<(), MakerError> {
        let mut path = path::Path::new("maker");
        let mut path_str = path.to_string_lossy().to_string();
        if !path.exists() {
            if let Some(home) = option_env!("HOME") {
                path_str.insert_str(0, format!("{}/", home).as_str());
                path = &path::Path::new(&path_str);
                if !path.exists() {
                    return Err(MakerError::DotMakerNotFound);
                }
            } else {
                return Err(MakerError::DotMakerNotFound);
            }
        }
        let file = fs::read_to_string(path_str).unwrap();
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
    pub fn parse_args(&mut self) -> Result<(), MakerError> {
        let mut state: ArgsParseState = ArgsParseState::Input;
        let mut args: Vec<String> = env::args().collect();
        if args.len() <= 1 {
            return Err(MakerError::OverrideHelp);
        }
        args.remove(0);
        for i in args {
            if i == "-o" || i == "--output" {
                state = ArgsParseState::Output;
                continue;
            }
            if i == "-c" || i == "--config" {
                state = ArgsParseState::Config;
                continue;
            }
            if i == "-a" || i == "--async" {
                self.async_commands = true;
                continue;
            }
            if i == "-f" || i == "--flags" {
                state = ArgsParseState::AdditionalFlags;
                continue;
            }
            if i == "--help" {
                return Err(MakerError::OverrideHelp);
            }
            if i == "--maker" {
                return Err(MakerError::OverrideMakerCreate);
            }
            match state {
                ArgsParseState::Input => {
                    if let Err(x) = fs::metadata(i.clone()) {
                        println!("ERROR! Failed to get metadata for '{}'!\n{}", i, x);
                        continue;
                    }
                    if !fs::metadata(i.clone()).unwrap().is_dir() {
                        self.input_files.push(i);
                    }
                }
                ArgsParseState::Output => {
                    self.output_dir = i;
                    state = ArgsParseState::Input;
                }
                ArgsParseState::Config => {
                    self.set_config = i;
                    state = ArgsParseState::Input;
                }
                ArgsParseState::AdditionalFlags => {
                    self.additional_flags = i;
                    state = ArgsParseState::Input;
                },
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
    pub fn execute(&mut self) -> Result<(), MakerError> {
        for i in self.input_files.clone() {
            let split_index = i.find('.');
            match split_index {
                Some(_) => {},
                None => {
                    println!("ERROR: tried to split \"{}\" at '.' but failed! Is it a directory?", i);
                    continue;
                },
            }
            let config = self
                .find_config(
                    i.split_at(split_index.unwrap())
                    .1,
                );
            if let None = config {
                println!("ERROR! Extension not covered for file '{}'", i);
                continue;
            } 
            let config = config.unwrap();

            let output_file = i.split_at(i.find('.').unwrap()).0;
            let format = config
                .configs
                .get(&self.set_config);
            if let None = format {
                return Err(MakerError::ConfigNotFound(self.set_config.clone()))
            }
            let mut format_real = format.unwrap()
                .clone()
                .replace("%file%", i.as_str())
                .replace(
                    "%output%",
                    format!("{}/{}", self.output_dir, output_file).as_str(),
                );
            format_real
                .push_str(self.additional_flags.as_str());
            match fs::create_dir(self.output_dir.clone()) {
                _ => {}
            }
            let mut format_split = format_real.split_whitespace();
            let mut com = Command::new(format_split.next().unwrap());
            for arg in format_split {
                com.arg(arg);
            }
            if self.async_commands {
                match com.spawn() {
                    Ok(x) => {self.async_processes.push((x, i))},
                    Err(x) => {
                        println!("COMMAND ERROR:\nERROR_INFO:{}\nFORMAT:{}", x, format_real);
                    }
                }
            } else {
                println!("---{}---", i);
                if let Err(x) = com.output() {
                    println!("COMMAND ERROR:\nERROR_INFO:{}\nFORMAT:{}", x, format_real);
                } else {
                }

            }
        }
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
    AdditionalFlags,
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
