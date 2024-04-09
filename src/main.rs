use std::{
    fs,
    io::{self, Write},
    path::Path,
};

pub mod maker;

fn main() -> io::Result<()> {
    let mut maker_main = maker::LaSingleton::init();
    match maker_main.parse_args() {
        Ok(_) => {}
        Err(x) => {
            match x {
                maker::MakerError::OverrideHelp => {
                    println!("maker: build system for playgrounds\n");

                    println!("usage: maker [-flag...] test1.lang test2.lang ...");
                    println!("       -o | --output: Set output directory - default is 'bin'");
                    println!("       -c | --config: Set current config, else format set");
                    println!("                      without a preceding config is used");
                    println!("       -f | --flags:  Set additional flags for all formats");
                    println!("       -a | --async: all commands run via maker are spawned");
                    println!("                     as children, and ran at the same time,");
                    println!("                     thus being \"async\". depending on compilation");
                    println!("                     time and number of files being compiled,");
                    println!("                     this may be resource intensive and janky.");
                    println!("            --maker: create maker file template in current dir");
                    println!("            --help : show this help text");
                    return Ok(());
                }
                maker::MakerError::OverrideMakerCreate => {
                    if Path::new("maker").exists() {
                        println!("maker already exists!");
                        return Ok(());
                    }
                    if let Ok(mut x) = fs::File::create("maker") {
                        let _ =
                            writeln!(x, "extension .lang # .lang2 .lang3 # You can add multiple extensions here.");
                        let _ = writeln!(x, "\tformat langc %file% -o %output%\n");
                        let _ = writeln!(x, "\tconfig testConfig");
                        let _ = writeln!(x, "\t\tformat testConfigLangC %file% -o %output%");
                        let _ = writeln!(x, "\t\tcomment //MAKER: ");
                        let _ = writeln!(x, "\tend-config");
                        let _ = writeln!(x, "end-extension");
                        return Ok(());
                    }
                }
                _ => {}
            }
        }
    };
    if cfg!(debug_assertions) {
        maker_main.debug();
    }
    match maker_main.get_config() {
        Ok(_) => {}
        Err(x) => {
            print!("ERROR! ");
            match x {
                maker::MakerError::DotMakerNotFound => {
                    println!("maker not found!");
                }
                _ => {
                    println!("{x:?}")
                }
            }
            return Ok(());
        }
    }
    if cfg!(debug_assertions) {
        maker_main.debug();
    }
    match maker_main.execute() {
        Ok(_) => {
            for i in maker_main.async_processes {
                let id = i.0.id();
                println!("---waiting for {}|{}---", i.1, id);
                let result = i.0.wait_with_output();
                match result {
                    Ok(y) => {
                        if !y.status.success() {
                            println!("ERROR! {}|{} RETURNED {}", i.1, id, y.status);
                        } 
                    }
                    Err(y) => {
                        println!("ERROR: {:?} | {:?}", y.kind(), y.raw_os_error());
                    }
                }
            }
        }
        Err(err) => {
            print!("ERROR! ");
            match err {
                maker::MakerError::ParsingError(x) => {
                    println!("Parsing Error. Tried to parse {} but failed!", x);
                    println!("Is this a directory? Or does this file not have an extension?")
                }
                maker::MakerError::ConfigNotFound(x) => {
                    println!("Cannot find config '{}", x);
                }
                maker::MakerError::ExtensionNotCovered(x) => {
                    println!("Extension not covered for file '{}'", x);
                }
                _ => {}
            }
        }
    };
    if maker_main.async_commands {
        println!("--- Done. ---");
    }
    Ok(())
}
