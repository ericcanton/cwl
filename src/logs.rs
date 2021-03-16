use std::fmt;
use std::rc::Rc;

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
        let log_service = match name.contains("/aws/lambda/") {
            true => AmazonService::Lambda,
            false => AmazonService::Unknown,
        };

        Self {
            name,
            service: log_service,
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

impl From<rusoto_LogGroup> for LogGroup {
    fn from(lg: rusoto_LogGroup) -> Self {
        let mut new_lg = LogGroup::new(lg.log_group_name.unwrap());

        new_lg.stored_bytes = lg.stored_bytes;

        new_lg
    }
}

impl fmt::Display for LogGroup {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Group Name: {}\nService: {}\nStored bytes: {}", 
            self.name.blue().bold(),
            self.service,
            self.stored_bytes.unwrap_or(-1).to_string().red()
        )
    }
}

/* =======================================================

    Structs, Enums, Traits for Log Streams

======================================================= */ 
#[derive(Default)]            
pub struct LogStream {
    name: String,
    group: Rc<LogGroup>,
    last_event_ts: Option<i64>,
    last_ingest_ts: Option<i64>,
    events_page: Option<Vec<OutputLogEvent>>,
    next_page: Option<String>,
    prev_page: Option<String>,
}

impl LogStream {
    pub fn new(name: String, group: Rc<LogGroup>) -> Self {
        Self {
            name,
            group, 
            ..Default::default() 
        }
    }


    pub async fn get_log_stream_events(&mut self) -> Result<(), ()> {
        let client = CloudWatchLogsClient::new(Region::UsEast1);

        let mut gle_req = GetLogEventsRequest {
            log_group_name: String::from(&self.group.name),
            log_stream_name: String::from(&self.name),
            ..Default::default()
        };

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

impl fmt::Display for LogStream {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Stream Name: {}", 
            self.name.green())
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
                println!("{}\n================\n", group);
            }
            Ok(())
        } 
        None => {
            println!("No log groups found!");
            Err(())
        }
    }
}

pub async fn get_log_streams_for(group: Rc<LogGroup>) -> Option<Vec<LogStream>> {
    let client = CloudWatchLogsClient::new(Region::UsEast1);

    // We need the log stream to get the sequence token
    let dls_req = DescribeLogStreamsRequest { 
        log_group_name: String::from(&group.name),
        ..Default::default()
    };

    let log_stream_resp = client.describe_log_streams(dls_req).await;

    match log_stream_resp {
        Ok(streams) => {
            Some(
                streams.log_streams
                    .unwrap()
                    .into_iter()
                    .map(|ls| 
                        LogStream::new(
                            ls.log_stream_name.unwrap(),
                            Rc::clone(&group),
                        ) 
                    )
                    .collect()
            )
        },
        Err(_) => None
    }
}


pub async fn ls_log_streams_for(group: Rc<LogGroup>) -> Result<(), ()> {
    println!("\n=============================");
    println!("=== ls: Log Streams =========");
    println!("=============================");
    println!("for Log Group:");
    println!("{}", &group);
    println!("=============================\n");

    match get_log_streams_for(group).await {
        Some(log_streams) => {
            // println!("...Log streams here....");
            for stream in log_streams {
                println!("{}", stream);
            }
            Ok(())
        } 
        None => {
            println!("No log groups found!");
            Err(())
        }
    }
}

pub async fn get_log_events_for(stream: LogStream) -> Option<Vec<OutputLogEvent>> {
    let client = CloudWatchLogsClient::new(Region::UsEast1);

    // We need the log stream to get the sequence token
    let gle_req = GetLogEventsRequest { 
        log_stream_name: String::from(&stream.name),
        log_group_name: String::from(&stream.group.name),
        ..Default::default()
    };

    let log_event_resp = client.get_log_events(gle_req).await;

    match log_event_resp {
        Ok(event_response) => event_response.events,
        Err(_) => None
    }
}

pub async fn ls_log_events_for(stream: LogStream) -> Result<(), ()> {
    println!("\n=============================");
    println!("=== ls: Log Events ==========");
    println!("=============================");
    println!("for Log Stream:");
    println!("{}", &stream);
    println!("{}", &stream.group);
    println!("=============================\n");

    match get_log_events_for(stream).await {
        Some(log_events) => {
            let mut i = 0;
            // Cannot .enumerate() due to lack of trait...
            let n = log_events.len();
            for event in log_events {
                if i == 0 {
                    println!("Timestamp: {}\nIngestion: {}\n=== BEGIN STREAM ===\n",
                        event.timestamp.unwrap_or(-1).to_string().red(),
                        event.ingestion_time.unwrap_or(-1).to_string().red()
                    );
                } else if i == n-1 {
                    println!("\n=== END STREAM ===\nTimestamp: {}\nIngestion: {}\n",
                        event.timestamp.unwrap_or(-1).to_string().red(),
                        event.ingestion_time.unwrap_or(-1).to_string().red()
                    );

                }

                println!("{}", 
                    event.message.unwrap_or_default()
                );

                i += 1;
            }
            Ok(())
        } 
        None => {
            println!("No log streams found!");
            Err(())
        }
    }

}
