use std::fmt;

use colored::*;

use chrono::Utc;

use rusoto_core::Region;
use rusoto_logs::{
    CloudWatchLogs, CloudWatchLogsClient, DescribeLogGroupsRequest, DescribeLogGroupsResponse,
    DescribeLogStreamsRequest, LogGroup as rusoto_LogGroup, LogStream as rusoto_LogStream,
};


/* =======================================================

    Structs, Enums, Traits for Log Groups.

======================================================= */ 
pub enum AmazonService {
    Lambda,
}

impl fmt::Display for AmazonService {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AmazonService::Lambda => write!(f, "{}", "AWS Lambda".green())
        }
    }
}

pub struct LogGroup {
    name: String,
    service: AmazonService,
    stored_bytes: Option<i64>,
}

impl From<rusoto_LogGroup> for LogGroup {
    fn from(lg: rusoto_LogGroup) -> Self {
        let log_name = lg.log_group_name.unwrap();

        LogGroup { 
            name: log_name,
            service: AmazonService::Lambda,
            stored_bytes: lg.stored_bytes,
        }
    }
}

impl fmt::Display for LogGroup {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Name: {}\nService: {}\nStored bytes: {}\n================\n", 
            self.name.blue().bold(), self.service, self.stored_bytes.unwrap_or(0).to_string().red())
    }
}

/* =======================================================

    Structs, Enums, Traits for Log Streams

======================================================= */ 
// pub struct LogStream {
//     name: String,
//     log_group: LogGroup,
// }

// impl From<rusoto_LogStream> for LogStream {
//     fn from(lg: rusoto_LogStream) -> Self {

//         LogStream { 
//             name: log_name,
//             log_group: 
//         }
//     }
// }

// impl fmt::Display for LogStream {
//     fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
//         write!(f, "Name: {}\n================\n", 
//             self.name.blue())
//     }
// }


/* =======================================================

    Functions

======================================================= */ 
pub async fn get_log_groups() -> Option<Vec<LogGroup>> {
    let client = CloudWatchLogsClient::new(Region::UsEast1);

    // We need the log stream to get the sequence token
    let req: DescribeLogGroupsRequest = Default::default();

    let resp = client.describe_log_groups(req).await;

    match resp {
        Ok(log_groups) => {
            Some(
                log_groups.log_groups
                    .unwrap()
                    .into_iter()
                    .map(|lg| lg.into())
                    .collect()
            )
        },
        Err(_) => None
    }
}

pub async fn ls_log_groups() -> Result<(), ()> {

    match get_log_groups().await {
        Some(log_groups) => {
            println!("\n=============================");
            println!("=== ls: Log Groups ==========");
            println!("=============================\n");
            for group in log_groups {
                println!("{}", group);
            }
            Ok(())
        } 
        None => {
            println!("No log groups found!");
            Err(())
        }
    }
}

// pub async fn list_log_streams_for_group(group: LogGroup) -> Option<Vec<LogStream>> {
//     let client = CloudWatchLogsClient::new(Region::UsEast1);

//     // We need the log stream to get the sequence token
//     let mut req: DescribeLogStreamsRequest = Default::default();
//     req.log_group_name = group.name;

//     let resp = client.describe_log_streams(req).await;

//     match resp {
//         Ok(log_groups) => {
//             Some(
//                 log_groups.log_streams
//                     .unwrap()
//                     .into_iter()
//                     .map(|lg| lg.into())
//                     .collect()
//             )
//         },
//         Err(_) => None

//     }
// }
