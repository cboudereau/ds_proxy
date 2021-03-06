extern crate encrypt;
extern crate env_logger;
extern crate log;
extern crate sodiumoxide;

use docopt::Docopt;
use encrypt::args::{Args, USAGE};
use encrypt::config::Config;
use log::info;

fn main() {
    env_logger::init();
    sodiumoxide::init().unwrap();

    let docopt: Docopt = Docopt::new(USAGE)
        .unwrap_or_else(|e| e.exit())
        .version(Some(env!("GIT_HASH").to_string()));

    let args: Args = docopt.deserialize().unwrap_or_else(|e| e.exit());

    let config: Config = Config::create_config(&args);

    if args.cmd_proxy {
        if args.flag_noop {
            info!("proxy in dry mode")
        }

        let _ = encrypt::proxy::main(config);
    } else if args.cmd_encrypt {
        encrypt::file::encrypt(config);
    } else if args.cmd_decrypt {
        encrypt::file::decrypt(config);
    }
}
