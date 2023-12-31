use std::io;

pub mod maker;

fn main() -> io::Result<()> {
    let mut maker_main = maker::LaSingleton::init();
    maker_main.parse_args().unwrap_or_else(|_| {
        println!("maker: build system for playgrounds\n");

        println!("usage: maker [-o -c] test1.rs test2.rs ...");
        println!("       -o: Set output directory - default is 'bin'");
        println!(
            "       -c: Set current config, else format set \n{}", // CURSED
            "           without a preceding config is used"
        );
        std::process::exit(0);
    });
    maker_main.get_config()?;
    if cfg!(debug_assertions) {
        maker_main.debug();
    }
    maker_main.execute()?;

    Ok(())
}
