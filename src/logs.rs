use std::fmt;

use colored::*;

use chrono::Utc;

use rusoto_core::Region;
use rusoto_logs::{
    CloudWatchLogs, CloudWatchLogsClient, DescribeLogGroupsRequest, DescribeLogGroupsResponse,
    DescribeLogStreamsRequest, LogGroup as rusoto_LogGroup, LogStream as rusoto_LogStream,
    OutputLogEvent, GetLogEventsRequest, GetLogEventsResponse
};


/* =======================================================

    Structs, Enums, Traits for Log Groups.

======================================================= */ 
pub enum AmazonService {
    Lambda,
    Unknown,
}

impl fmt::Display for AmazonService {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AmazonService::Lambda => write!(f, "{}", "AWS Lambda".green()),
            AmazonService::Unknown => write!(f, "{}", "Unknown".red().on_yellow()),
        }
    }
}

pub struct LogGroup {
    name: String,
    service: AmazonService,
    stored_bytes: Option<i64>,
}

impl LogGroup {
    pub fn new(name: String) -> Self {
        Self {
            name,
            service: AmazonService::Unknown,
            stored_bytes: None
        }
    }
}

impl Default for LogGroup {
    fn default() -> Self {
        Self {
            name: String::from(""),
            service: AmazonService::Unknown,
            stored_bytes: None,
        }
    }

}

impl<'a> Default for &'a LogGroup {
    fn default() -> Self {
        let lg = LogGroup {
            name: String::from(""),
            service: AmazonService::Unknown,
            stored_bytes: None,
        };

        &lg
    }
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
            self.name.blue().bold(),
            self.service,
            self.stored_bytes.unwrap_or(0).to_string().red()
        )
    }
}

/* =======================================================

    Structs, Enums, Traits for Log Streams

======================================================= */ 
#[derive(Default)]            
pub struct LogStream<'a> {
    name: String,
    group: &'a LogGroup,
    last_event_ts: Option<i64>,
    last_ingest_ts: Option<i64>,
    events_page: Option<Vec<OutputLogEvent>>,
    next_page: Option<String>,
    prev_page: Option<String>,
}

impl<'a> LogStream<'a> {
    pub fn new(name: String, group: &'a LogGroup) -> Self {

        Self {
            name,
            group, 
            ..Default::default() 
        }
    }


    pub async fn get_log_stream_events(&mut self) -> Result<(), ()> {
        let client = CloudWatchLogsClient::new(Region::UsEast1);

        let mut gle_req: GetLogEventsRequest = Default::default();
        gle_req.log_group_name = String::from(&self.group.name);
        gle_req.log_stream_name = String::from(&self.name);

        // ugly. How to do better?
        if let Some(page) = &self.next_page {
            gle_req.start_from_head = Some(true);
            gle_req.next_token = Some(String::from(page))
        } 
        //

        let log_event_resp = client.get_log_events(gle_req).await;

        match log_event_resp {
            Ok(event_vec) => {
                self.events_page = event_vec.events;
                self.next_page = event_vec.next_forward_token;
                self.prev_page = event_vec.next_backward_token;

                Ok(())
            },
            Err(_) => Err(())
        }
    }

}

impl<'a> fmt::Display for LogStream<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Name: {}\n================\n", 
            self.name.blue())
    }
}


/* =======================================================

    Functions

======================================================= */ 
pub async fn get_log_groups() -> Option<Vec<LogGroup>> {
    let client = CloudWatchLogsClient::new(Region::UsEast1);
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

pub async fn get_log_streams_for(group: &'static LogGroup) -> Option<Vec<LogStream<'static>>> {
    let client = CloudWatchLogsClient::new(Region::UsEast1);

    // We need the log stream to get the sequence token
    let mut dls_req: DescribeLogStreamsRequest = Default::default();
    dls_req.log_group_name = String::from(&group.name);

    let log_stream_resp = client.describe_log_streams(dls_req).await;

    match log_stream_resp {
        Ok(streams) => {
            Some(vec![Default::default()])
        },
        Err(_) => None
    }
}


pub async fn ls_log_streams_for(group: &'static LogGroup) -> Result<(), ()> {

    match get_log_streams_for(&group).await {
        Some(_log_streams) => {
            println!("\n=============================");
            println!("=== ls: Log Streams for =====");
            println!("{}", group);
            println!("=============================\n");
            // for group in log_streams {
            //     println!("{}", group);
            // }
            Ok(())
        } 
        None => {
            println!("No log groups found!");
            Err(())
        }
    }
}

// pub async fn list_log_streams_for_group(group: LogGroup) -> Option<Vec<LogStream>> {
// }
