use std::{
    collections::HashMap,
    env, fs, path,
    process::{Child, Command}, str,
};
#[derive(Debug)]
pub enum MakerError {
    NotEnoughArgs,
    ParsingError(String),
    ExtensionNotCovered(String),
    ConfigNotFound(String),
    DotMakerNotFound,
    MiscError(String),
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
        // rename any instance of ".maker" to maker?
        if let Ok(_ok) = fs::metadata(".maker") {
            if let Err(_err) = fs::metadata("maker") {
                println!(".maker exists but maker doesn't! renaming '.maker' to 'maker'...");
                if let Err(uhoh) = fs::rename(".maker", "maker") {
                    return Err(MakerError::MiscError(format!(
                                    "Unable to rename '.maker' to 'maker'! {}",
                                    uhoh
                                )))
                }
            } else {
                println!(".maker exists, but so does maker! you should prolly delete '.maker'");
            }
        }

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
        let mut is_pushed = false;
        let mut config_string = String::from("__DEFAULT__");
        if cfg!(debug_assertions) {
            dbg!(&file);
        }
        let mut line_iter = file.lines();
        while let Some(line_str) = line_iter.next() {
            let mut line = String::from(line_str);
            line = line.trim().to_string(); 
            if let Some(c) = line.find('#') {
                line = line.split_at(c).0.to_string();
            }
            {
                if line.ends_with('\\') {
                    line = line.trim_end_matches('\\').to_string();
                    loop {
                        let next_line = line_iter.next();
                        let mut break_loop = false;
                        if let Some(mut str) = next_line {
                            if !str.ends_with('\\') {
                                break_loop = true;
                            }
                            str = str.trim();
                            str = str.trim_end_matches('\\');
                            line.push_str(str);

                        }
                        if break_loop {
                            break;
                        }
                    }
                }
                if line.starts_with("extension") {
                    line = line.trim_start_matches("extension").to_string();
                    temp_config.extensions = split_string(line.to_string());
                    is_pushed = false;
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
                    is_pushed = true;
                }
            }
        }
        if !is_pushed {
            return Err(MakerError::ParsingError("Extension config not pushed.".to_string()))
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
    pub fn execute(&mut self) -> Result<(), MakerError> {
        for i in self.input_files.clone() {
            let split_index = i.find('.');
            match split_index {
                Some(_) => {}
                None => {
                    println!(
                        "ERROR: tried to split \"{}\" at '.' but failed! Is it a directory?",
                        i
                    );
                    continue;
                }
            }
            let config = self.find_config(i.split_at(split_index.unwrap()).1);
            if config.is_none() {
                println!("ERROR! Extension not covered for file '{}'", i);
                continue;
            }
            let config = config.unwrap();

            let output_file = i.split_at(i.find('.').unwrap()).0;
            let format = config.configs.get(&self.set_config);
            if format.is_none() {
                return Err(MakerError::ConfigNotFound(self.set_config.clone()));
            }
            let mut format_real = format
                .unwrap()
                .clone()
                .replace("%file%", i.as_str())
                .replace(
                    "%output%",
                    format!("{}/{}", self.output_dir, output_file).as_str(),
                );
            format_real.push_str(self.additional_flags.as_str());
            let _ = fs::create_dir(self.output_dir.clone()); // dir creation result doesnt matter
            let mut format_split = format_real.split_whitespace();
            let mut com = Command::new(format_split.next().unwrap());
            for arg in format_split {
                com.arg(arg);
            }
            if self.async_commands {
                match com.spawn() {
                    Ok(x) => self.async_processes.push((x, i)),
                    Err(x) => {
                        println!("COMMAND ERROR:\nERROR_INFO:{}\nFORMAT:{}", x, format_real);
                    }
                }
            } else {
                println!("---{}---", i);
                match com.output() {
                    Ok(x) => {
                        if let Ok(s) = std::str::from_utf8(x.stdout.as_slice()) {
                            print!("{}", s);
                        }
                        if let Ok(s) = std::str::from_utf8(x.stderr.as_slice()) {
                            print!("{}", s);
                        }
                    },
                    Err(x) => {
                        println!("COMMAND ERROR:\nERROR_INFO:{}\nFORMAT:{}", x, format_real);
                    },
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

#[derive(Debug, Clone, Default)]
struct MakerConfig {
    extensions: Vec<String>,
    configs: HashMap<String, String>, // or, config to format.
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
