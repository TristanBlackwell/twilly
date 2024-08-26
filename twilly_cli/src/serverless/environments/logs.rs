use chrono::{Datelike, Duration};
use std::{fs::File, io::Write, process};

use inquire::{validator::Validation, Confirm, MultiSelect, Select, Text};
use strum::IntoEnumIterator;
use strum_macros::{Display, EnumIter, EnumString};
use twilly::{
    serverless::{
        environments::{logs::Level, ServerlessEnvironment},
        services::ServerlessService,
    },
    Client, ErrorKind,
};
use twilly_cli::{
    get_action_choice_from_user, get_date_from_user, prompt_user, prompt_user_multi_selection,
    prompt_user_selection, ActionChoice, DateRange,
};

/// Actions general to Logs.
#[derive(Debug, Clone, Display, EnumIter, EnumString)]
pub enum LogsAction {
    #[strum(to_string = "Get Log")]
    GetLog,
    #[strum(to_string = "List Logs")]
    ListLogs,
    Back,
    Exit,
}

/// Actions for a specific Log Resource.
#[derive(Debug, Clone, Display, EnumIter, EnumString)]
pub enum LogAction {
    #[strum(to_string = "List details")]
    ListDetails,
    Back,
    Exit,
}

/// Quick select time range options.
#[derive(Debug, Clone, Display, EnumIter, EnumString)]
pub enum TimeRangeOptions {
    #[strum(to_string = "Last 30 minutes")]
    ThirtyMinutes,
    #[strum(to_string = "Last hour")]
    LastHour,
    #[strum(to_string = "Last 6 hours")]
    LastSixHours,
    Today,
    Custom,
}

pub async fn choose_log_action(
    twilio: &Client,
    serverless_service: &ServerlessService,
    serverless_environment: &ServerlessEnvironment,
) {
    let options: Vec<LogsAction> = LogsAction::iter().collect();

    loop {
        let resource_selection_prompt = Select::new("Select an action:", options.clone());
        if let Some(resource) = prompt_user_selection(resource_selection_prompt) {
            match resource {
                LogsAction::GetLog => {
                    let log_sid_prompt = Text::new("Please provide a Log SID:")
                        .with_placeholder("NO...")
                        .with_validator(|val: &str| {
                            if val.starts_with("NO") && val.len() == 34 {
                                Ok(Validation::Valid)
                            } else {
                                Ok(Validation::Invalid(
                                    "Log SID should be 34 characters in length".into(),
                                ))
                            }
                        });

                    if let Some(log_sid) = prompt_user(log_sid_prompt) {
                        match twilio
                            .serverless()
                            .service(&serverless_service.sid)
                            .environment(&serverless_environment.sid)
                            .log(&log_sid)
                            .get()
                            .await
                        {
                            Ok(log) => {
                                println!("Log found.");
                                println!();

                                if let Some(action_choice) = get_action_choice_from_user(
                                    vec![String::from("List Details")],
                                    "Select an action: ",
                                ) {
                                    match action_choice {
                                        ActionChoice::Back => {
                                            break;
                                        }
                                        ActionChoice::Exit => process::exit(0),
                                        ActionChoice::Other(choice) => match choice.as_str() {
                                            "List Details" => {
                                                println!("{:#?}", log);
                                                println!();
                                            }
                                            _ => println!("Unknown action '{}'", choice),
                                        },
                                    }
                                } else {
                                    break;
                                }
                            }
                            Err(error) => match error.kind {
                                ErrorKind::TwilioError(twilio_error) => {
                                    if twilio_error.status == 404 {
                                        println!("A Log with SID '{}' was not found.", &log_sid);
                                        println!();
                                    } else {
                                        panic!("{}", twilio_error);
                                    }
                                }
                                _ => panic!("{}", error),
                            },
                        }
                    }
                }
                LogsAction::ListLogs => {
                    let utc_now = chrono::Utc::now();
                    let mut start_date: Option<chrono::DateTime<chrono::Utc>> = None;
                    let mut end_date: Option<chrono::DateTime<chrono::Utc>> = Some(utc_now);

                    let mut user_selected_time_range = false;

                    let time_range_prompt = Confirm::new("Would you like to select a time range?")
                        .with_placeholder("N")
                        .with_help_message("Will retrieve the last 24 hours by default.")
                        .with_default(false);

                    if let Some(time_range_decision) = prompt_user(time_range_prompt) {
                        if time_range_decision {
                            user_selected_time_range = true;

                            let options: Vec<TimeRangeOptions> = TimeRangeOptions::iter().collect();
                            let time_range_selection_prompt =
                                Select::new("Select a time range:", options.clone());
                            if let Some(time_range) =
                                prompt_user_selection(time_range_selection_prompt)
                            {
                                match time_range {
                                    TimeRangeOptions::ThirtyMinutes => {
                                        start_date = Some(utc_now - Duration::minutes(30));
                                    }
                                    TimeRangeOptions::LastHour => {
                                        start_date = Some(utc_now - Duration::hours(1));
                                    }
                                    TimeRangeOptions::LastSixHours => {
                                        start_date = Some(utc_now - Duration::hours(6));
                                    }
                                    TimeRangeOptions::Today => {
                                        start_date = Some(
                                            chrono::DateTime::parse_from_str(
                                                utc_now
                                                    .format("%Y-%m-%dT00:00:00%z")
                                                    .to_string()
                                                    .as_str(),
                                                "%Y-%m-%dT%H:%M:%S%z",
                                            )
                                            .unwrap()
                                            .into(),
                                        );
                                    }
                                    TimeRangeOptions::Custom => {
                                        let utc_30_days_ago = utc_now - chrono::Duration::days(30);
                                        if let Some(user_start_date) = get_date_from_user(
                                            "Choose a start date:",
                                            Some(DateRange {
                                                minimum_date: chrono::NaiveDate::from_ymd_opt(
                                                    utc_30_days_ago.year(),
                                                    utc_30_days_ago.month(),
                                                    utc_30_days_ago.day(),
                                                )
                                                .unwrap(),
                                                maximum_date: chrono::NaiveDate::from_ymd_opt(
                                                    utc_now.year(),
                                                    utc_now.month(),
                                                    utc_now.day(),
                                                )
                                                .unwrap(),
                                            }),
                                        ) {
                                            start_date = Some(
                                                chrono::DateTime::parse_from_str(
                                                    user_start_date
                                                        .format("%Y-%m-%dT00:00:00+0000")
                                                        .to_string()
                                                        .as_str(),
                                                    "%Y-%m-%dT%H:%M:%S%z",
                                                )
                                                .unwrap()
                                                .into(),
                                            );
                                            if let Some(user_end_date) = get_date_from_user(
                                                "Choose an end date:",
                                                Some(DateRange {
                                                    minimum_date: chrono::NaiveDate::from_ymd_opt(
                                                        user_start_date
                                                            .year_ce()
                                                            .1
                                                            .try_into()
                                                            .unwrap(),
                                                        user_start_date.month0() + 1,
                                                        user_start_date.day0() + 1,
                                                    )
                                                    .unwrap(),
                                                    maximum_date: chrono::NaiveDate::from_ymd_opt(
                                                        utc_now.year(),
                                                        utc_now.month(),
                                                        utc_now.day(),
                                                    )
                                                    .unwrap(),
                                                }),
                                            ) {
                                                if user_end_date == utc_now.date_naive() {
                                                    // If the user selected the current day we'll assume its up to the current time
                                                    // also. So here we use the UTC now to do so.
                                                    end_date = Some(
                                                        chrono::DateTime::parse_from_str(
                                                            utc_now
                                                                .format("%Y-%m-%dT%H:%M:%S%z")
                                                                .to_string()
                                                                .as_str(),
                                                            "%Y-%m-%dT%H:%M:%S%z",
                                                        )
                                                        .unwrap()
                                                        .into(),
                                                    )
                                                } else {
                                                    // If the user did *not* select the current day then it's definitely in
                                                    // the past so we can assume they want to fetch that full day. We manually
                                                    // set the hours as such here.
                                                    end_date = Some(
                                                        chrono::DateTime::parse_from_str(
                                                            user_end_date
                                                                .format("%Y-%m-%dT23:59:59+0000")
                                                                .to_string()
                                                                .as_str(),
                                                            "%Y-%m-%dT%H:%M:%S%z",
                                                        )
                                                        .unwrap()
                                                        .into(),
                                                    )
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }

                    // Only continue if the user filtered by dates *and* provided both options.
                    // If they didn't then they must of cancelled the operation.
                    if !user_selected_time_range || (start_date.is_some() && end_date.is_some()) {
                        let filter_function =
                            Confirm::new("Would you like to filter by a specific function?")
                                .with_placeholder("N")
                                .with_default(false);

                        if let Some(filter_decision) = prompt_user(filter_function) {
                            let mut function_sid: Option<String> = None;
                            if filter_decision {
                                let function_sid_prompt =
                                    Text::new("Please provide a function SID:")
                                        .with_placeholder("ZH...")
                                        .with_validator(|val: &str| {
                                            if val.starts_with("ZH") && val.len() == 34 {
                                                Ok(Validation::Valid)
                                            } else {
                                                Ok(Validation::Invalid(
                                        "Function SID should be 34 characters in length".into(),
                                    ))
                                            }
                                        });

                                if let Some(user_function_sid) = prompt_user(function_sid_prompt) {
                                    function_sid = Some(user_function_sid);
                                }
                            }

                            let options: Vec<Level> = Level::iter().collect();
                            let log_level_prompt = MultiSelect::new(
                                "Select the log levels you would like to view:",
                                options,
                            )
                            .with_default(&[0_usize, 1, 2]);

                            if let Some(log_levels) = prompt_user_multi_selection(log_level_prompt)
                            {
                                println!("Fetching logs...");
                                let mut serverless_logs = twilio
                                    .serverless()
                                    .service(&serverless_service.sid)
                                    .environment(&serverless_environment.sid)
                                    .logs()
                                    .list(function_sid, start_date, end_date)
                                    .await
                                    .unwrap_or_else(|error| panic!("{}", error));

                                println!("Filtering...");
                                serverless_logs.retain(|log| log_levels.contains(&log.level));

                                let number_of_logs = serverless_logs.len();

                                if number_of_logs == 0 {
                                    println!("No logs found.");
                                    println!();
                                } else {
                                    println!("Found {} logs.", number_of_logs);

                                    if let Some(output_decision) = get_action_choice_from_user(
                                        vec![String::from("Write to file"), String::from("View")],
                                        "Select an output: ",
                                    ) {
                                        match output_decision {
                                            ActionChoice::Back => {
                                                break;
                                            }
                                            ActionChoice::Exit => process::exit(0),
                                            ActionChoice::Other(choice) => match choice.as_str() {
                                                "Write to file" => {
                                                    match File::create(format!(
                                                        "{}.json",
                                                        &serverless_environment.sid
                                                    )) {
                                                        Ok(mut file_buffer) => {
                                                            match file_buffer
                                                                .write_all(
                                                                    serde_json::to_string_pretty(
                                                                        &serverless_logs,
                                                                    )
                                                                    .unwrap()
                                                                    .as_bytes(),
                                                                ) {
																	Ok(_) => println!("Log file created"),
																	Err(error) => eprintln!("Failed to fully write to log file. Action aborted: {error}")
																}
                                                        }
                                                        Err(error) => eprintln!(
                                                            "Unable to create log file. Action aborted: {error}"
                                                        ),
                                                    }
                                                }
                                                "View" => {
                                                    // Sort date descending (latest first)
                                                    serverless_logs.sort_by(|a, b| {
                                                        b.date_created.cmp(&a.date_created)
                                                    });

                                                    let mut selected_serverless_log_index: Option<
                                                        usize,
                                                    > = None;
                                                    loop {
                                                        let selected_serverless_log = if let Some(
                                                            index,
                                                        ) =
                                                            selected_serverless_log_index
                                                        {
                                                            &mut serverless_logs[index]
                                                        } else if let Some(action_choice) =
                                                            get_action_choice_from_user(
                                                                serverless_logs
                                                                    .iter()
                                                                    .map(|log| {
                                                                        format!(
                                                                            "({}) {} - {}",
                                                                            log.sid,
                                                                            log.date_created,
                                                                            log.message
                                                                        )
                                                                    })
                                                                    .collect::<Vec<String>>(),
                                                                "Choose a Serverless Log: ",
                                                            )
                                                        {
                                                            match action_choice {
                                                                ActionChoice::Back => {
                                                                    break;
                                                                }
                                                                ActionChoice::Exit => {
                                                                    process::exit(0)
                                                                }
                                                                ActionChoice::Other(choice) => {
                                                                    let serverless_log_position = serverless_logs
									.iter()
									.position(|list| list.sid == choice[1..35])
									.expect("Could not find Serverless Log in existing Serverless Log list");

                                                                    selected_serverless_log_index =
                                                                        Some(
                                                                            serverless_log_position,
                                                                        );
                                                                    &mut serverless_logs
                                                                        [serverless_log_position]
                                                                }
                                                            }
                                                        } else {
                                                            break;
                                                        };

                                                        let options: Vec<LogAction> =
                                                            LogAction::iter().collect();
                                                        let action_selection_prompt = Select::new(
                                                            "Select an action:",
                                                            options,
                                                        );
                                                        if let Some(action) = prompt_user_selection(
                                                            action_selection_prompt,
                                                        ) {
                                                            match action {
                                                                LogAction::ListDetails => {
                                                                    println!(
                                                                        "{:#?}",
                                                                        selected_serverless_log
                                                                    );
                                                                    println!();
                                                                }
                                                                LogAction::Back => {
                                                                    break;
                                                                }
                                                                LogAction::Exit => process::exit(0),
                                                            }
                                                        }
                                                    }
                                                }
                                                _ => println!("Unknown action '{}'", choice),
                                            },
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
                LogsAction::Back => {
                    break;
                }
                LogsAction::Exit => process::exit(0),
            }
        }
    }
}
