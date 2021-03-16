mod logs;

use std::process;
use std::rc::Rc;

use clap::{App, load_yaml};
 
#[tokio::main]
async fn main() {
    let cli_yml = load_yaml!("cw_cli.yml");

    let matches = App::from_yaml(cli_yml).get_matches();

    if let Some(ls) = matches.subcommand_matches("ls") {
        // cwl ls ...
        if let Some(gn) = ls.value_of("group_name") {
            // cwl ls -g group_name ...
            if let Some(sn) = ls.value_of("stream_name") {
                let group = logs::LogGroup::new(
                    String::from(gn)
                );
                let stream = logs::LogStream::new(
                    String::from(sn),
                    Rc::new(group)
                );

                // cwl ls -g group_name -s stream_name
                match logs::ls_log_events_for(stream).await {
                    Ok(_) => process::exit(0),
                    Err(_) => process::exit(1)
                }
            } else {
                let group = logs::LogGroup::new(String::from(gn));

                match logs::ls_log_streams_for(Rc::new(group)).await {
                    Ok(_) => process::exit(0),
                    Err(_) => process::exit(1)
                }
            }
        } else if ls.value_of("stream_name").is_some() {
            // cwl ls -s stream_name
            println!("You must supply the log group to which this belongs with -g.")
        } else {
            // cwl ls
            match logs::ls_log_groups().await {
                Ok(_) => process::exit(0),
                Err(_) => process::exit(1)
            }
        }
    } else {
        println!("Try again with ls subcommand.");
    }
    // if let Some(g) = matches.subcommand_matches("group") {
    //     if g.subcommand_matches("ls").is_some() {
    //         match logs::ls_log_groups().await {
    //             Ok(_) => process::exit(0),
    //             Err(_) => process::exit(1)
    //         }
    //     }
    // } else if let Some(st) = matches.subcommand_matches("stream") {
    //     if let Some(ls) = st.subcommand_matches("ls") {
    //         let group_name = ls.value_of("group_name").unwrap();
    //     } 
    // }

}