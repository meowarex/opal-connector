use anyhow::Result;
use colored::Colorize;

use crate::{
  auth_manager::AuthManager, config_manager::ConfigManager, data_collector::DataCollector,
};

/// The structure of the launch parameters.
#[derive(Debug, Clone)]
pub struct ArgParser {
  pub interval: f64,
  pub offline: bool,
}

impl ArgParser {
  pub async fn new() -> Result<ArgParser> {
    let mut arg_parser = ArgParser {
      interval: 1.0,
      offline: false,
    };
    let args: Vec<String> = std::env::args().collect();
    let mut index: usize = 0;
    while args.len() > index {
      let arg: &str = &args[index];
      match arg {
        "-h" | "--help" => {
          println!("\n{} Usage:", "●".green());
          println!("    {} {}", "xornet".yellow(), "[options]".bright_black());
          println!("\n{} Options:", "●".blue());
          println!(
            "    -h,  --help                         : {}",
            "Show this help".white()
          );
          println!(
            "    -v,  --version                      : {}",
            "Show version and exit".white()
          );
          println!(
            "    -su, --signup <key>                 : {}",
            "Sign up the machine with an authentication key to Xornet for online features".white()
          );
          println!(
            "    -i,  --interval   {}      : {}",
            "(default: 1)".bright_black(),
            "Data collection interval in seconds".white()
          );
          println!(
            "    -off,  --offline  {}  : {}",
            "(default: false)".bright_black(),
            "Disables sending data to Xornet's backend".white()
          );
          println!("\n{} Examples:", "●".magenta());
          println!("    {} {}", "$".bright_black(), "xornet".yellow());
          println!("    {} {} -i 0.25", "$".bright_black(), "xornet".yellow());
          println!(
            "    {} {} -su 86F8575ADBA746BA8008C9AA48317EF6",
            "$".bright_black(),
            "xornet".yellow(),
          );
          println!(
            "\n    {}",
            "More info at https://github.com/xornet-cloud/Reporter".bright_black()
          );

          println!();
          std::process::exit(0);
        }
        "-v" | "--version" => {
          println!("xornet v{}", env!("CARGO_PKG_VERSION"));
          std::process::exit(0);
        }
        "-off" | "--offline" => {
          arg_parser.offline = true;
        }
        "-su" | "--signup" => {
          if args.len() > index + 1 {
            index += 1;
            let two_factor_key = &args[index];
            let config_manager: ConfigManager = ConfigManager::new()?;

            if config_manager.config.backend_hostname.is_empty() {
              println!(
                "{}",
                "Backend Hostname is not set in the config.json, please set it and retry:".red(),
              );
              std::process::exit(1)
            }

            match AuthManager::signup(
              two_factor_key,
              &DataCollector::get_hostname()?,
              &config_manager.config.backend_hostname,
              &config_manager.config.uuid,
            )
            .await
            {
              Err(error) => {
                println!("{} {}", "Signup failed:".red(), error.to_string().red());
                std::process::exit(1)
              }
              Ok(response) => {
                ConfigManager::save_access_token(&response.access_token)?;
                println!(
                  "{} {}\n",
                  "Signup successful:".green(),
                  response.access_token
                );
                println!(
                                    "You can now start the reporter with the following command: \n    $ xornet --silent",
                                );
                std::process::exit(0)
              }
            }
          } else {
            println!(
              "{}",
              "Missing argument for option -su <key>, use -h for help".red()
            );
            std::process::exit(1);
          }
        }
        "-i" | "--interval" => {
          if args.len() > index + 1 {
            index += 1;
            arg_parser.interval = args[index]
              .parse::<f64>()
              .expect("Could not parse interval as integer");
          } else {
            println!(
              "{}",
              "Missing argument for option -i <interval>, use -h for help".red()
            );
            std::process::exit(1);
          }
        }
        _ => {}
      }
      index += 1;
    }
    Ok(arg_parser)
  }
}
