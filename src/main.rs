mod logs;

use std::process;

use clap::{App, load_yaml};
 
#[tokio::main]
async fn main() {
    let cli_yml = load_yaml!("cw_cli.yml");

    let matches = App::from_yaml(cli_yml).get_matches();

    if let Some(g) = matches.subcommand_matches("group") {
        if g.subcommand_matches("ls").is_some() {
            match logs::ls_log_groups().await {
                Ok(_) => process::exit(0),
                Err(_) => process::exit(1)
            }
        }
    } else if let Some(st) = matches.subcommand_matches("stream") {
        if let Some(ls) = st.subcommand_matches("ls") {
            let group_name = ls.value_of("group_name").unwrap();
            let group = LogGroup::new(String::from(group));

            match logs::ls_log_streams_for(&group).await {
                Ok(_) => process::exit(0),
                Err(_) => process::exit(1)
            }
        } else {
            println!("Try again with ls subcommand.");
        }
    }

}