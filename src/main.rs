use std::{io::{self, Write}, fs, path::Path};

pub mod maker;

fn main() -> io::Result<()> {
    let mut maker_main = maker::LaSingleton::init();
    match maker_main.parse_args() {
        Ok(_) => {},
        Err(x) => {
            match x {
                maker::MakerError::OverrideHelp => {
                    
                    println!("maker: build system for playgrounds\n");

                    println!("usage: maker [-o -c] test1.lang test2.lang ...");
                    println!("       -o | --output: Set output directory - default is 'bin'");
                    println!("       -c | --config: Set current config, else format set \n"); 
                    println!("                      without a preceding config is used");
                    println!("       -a | --async: all commands run via maker are spawned");
                    println!("                     as children. depending on compilation");
                    println!("                     time and number of files being compiled,"); 
                    println!("                     this may be resource intensive and janky.");
                    println!("       --maker: create .maker file template in current dir");
                    println!("       --help : show this help text");
                    return Ok(())
                },
                maker::MakerError::OverrideMakerCreate => {
                    if Path::new(".maker").exists() {
                        println!(".maker already exists!");
                        return Ok(())
                    }
                    if let Ok(mut x) = fs::File::create(".maker") {
                        let _ =
                            writeln!(x, "extension .lang # You can add multiple extensions per config");
                        let _ = writeln!(x, "\tformat langc %file% -o %output%\n");
                        let _ = writeln!(x, "\tconfig testConfig");
                        let _ = writeln!(x, "\tformat testConfigLangC %file% -o %output%");
                        let _ = 
                            writeln!(x, "push");
                        return Ok(())
                    }
                },
                _ => {}
            }
        },
    };
    if cfg!(debug_assertions) {
        maker_main.debug();
    }
    maker_main.get_config()?;
    if cfg!(debug_assertions) {
        maker_main.debug();
    }
    match maker_main.execute() {
        Ok(_) => {},
        Err(err) => {
            print!("ERROR! ");
            match err {
                maker::MakerError::ParsingError(x) => {
                    println!("Parsing Error. Tried to parse {} but failed!", x);
                    println!("Is this a directory? Or is this file not have an extension?")
                },
                maker::MakerError::ConfigNotFound(x) => {
                    println!("Cannot find config '{}", x);
                },
                maker::MakerError::ExtensionNotCovered(x) => {
                    println!("Extension not covered for file '{}'", x);
                }
                _ => {},
            }
        },
    };
    println!("--- Done. ---");
    Ok(())
}
