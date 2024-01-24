use std::{process, str::FromStr};

use chrono::{Datelike, NaiveDate};
use inquire::{validator::Validation, Confirm, DateSelect, Select, Text};
use strum::IntoEnumIterator;
use strum_macros::{Display, EnumIter, EnumString};
use twilio_cli::{
    get_action_choice_from_user, get_filter_choice_from_user, ActionChoice, FilterChoice,
};
use twilio_rust::{conversation::State, Client, ErrorKind};

#[derive(Clone, Display, EnumIter, EnumString)]
pub enum Action {
    #[strum(serialize = "Get conversation")]
    GetConversation,
    #[strum(serialize = "List Conversations")]
    ListConversations,
    #[strum(serialize = "Delete Conversation")]
    DeleteConversation,
    #[strum(serialize = "Delete all Conversations")]
    DeleteAllConversations,
    Back,
    Exit,
}

pub fn choose_conversation_account(twilio: &Client) {
    let options: Vec<Action> = Action::iter().collect();

    loop {
        let action_selection = Select::new("Select an action:", options.clone()).prompt();
        let action = action_selection.unwrap();
        match action {
            Action::GetConversation => {
                let conversation_sid =
                    Text::new("Please provide a conversation SID, or unique name:")
                        .with_placeholder("CH...")
                        .with_validator(|val: &str| {
                            if val.starts_with("CH") && val.len() == 34 {
                                Ok(Validation::Valid)
                            } else {
                                Ok(Validation::Invalid(
                                    "Conversation SID should be 34 characters in length".into(),
                                ))
                            }
                        })
                        .prompt()
                        .unwrap();

                let get_result = twilio.conversations().get(&conversation_sid);

                if get_result.is_ok() {
                    let conversation_action_choice = get_action_choice_from_user(
                        vec![String::from("Delete")],
                        "Select an action: ",
                    );

                    match conversation_action_choice {
                        Some(conversation_action) => match conversation_action {
                            ActionChoice::Back => break,
                            ActionChoice::Exit => process::exit(0),
                            ActionChoice::Other(choice) => match choice.as_str() {
                                "Delete" => {
                                    if Confirm::new(
                                        "Are you sure to wish to delete the Conversation? (Yes / No)",
                                    )
                                    .prompt()
                                    .unwrap()
                                    {
									println!("Deleting Conversation...");
                                        twilio.conversations().delete(&conversation_sid).unwrap_or_else(|error| panic!("{}", error) );
									println!("Conversation deleted.");
									println!();
                                    }
                                }
                                _ => println!("Unknown action '{}'", choice),
                            },
                        },
                        None => break,
                    }
                } else {
                    let get_error = get_result.unwrap_err();
                    match get_error.kind {
                        ErrorKind::TwilioError(twilio_error) => {
                            if twilio_error.status == 404 {
                                println!(
                                    "A Conversation with SID '{}' was not found.",
                                    &conversation_sid
                                );
                                println!("");
                            } else {
                                panic!("{}", twilio_error)
                            }
                        }
                        _ => panic!("{}", get_error),
                    };
                }
            }
            Action::ListConversations => {
                let mut start_date: Option<chrono::NaiveDate> = None;
                let mut end_date: Option<chrono::NaiveDate> = None;

                if Confirm::new("Would you like to filter between specified dates? (Yes / No)")
                    .prompt()
                    .unwrap()
                {
                    let utc_now = chrono::Utc::now();
                    let utc_one_year_ago = utc_now - chrono::Duration::days(365);
                    start_date = Some(get_date_from_user(
                        "Choose a start date:",
                        Some(DateRange {
                            minimum_date: chrono::NaiveDate::from_ymd_opt(
                                utc_one_year_ago.year(),
                                utc_one_year_ago.month(),
                                utc_one_year_ago.day(),
                            )
                            .unwrap(),
                            maximum_date: chrono::NaiveDate::from_ymd_opt(
                                utc_now.year(),
                                utc_now.month(),
                                utc_now.day(),
                            )
                            .unwrap(),
                        }),
                    ));
                    end_date = Some(get_date_from_user(
                        "Choose an end date:",
                        Some(DateRange {
                            minimum_date: chrono::NaiveDate::from_ymd_opt(
                                start_date.unwrap().year_ce().1.try_into().unwrap(),
                                start_date.unwrap().month0() + 1,
                                start_date.unwrap().day0() + 1,
                            )
                            .unwrap(),
                            maximum_date: chrono::NaiveDate::from_ymd_opt(
                                utc_now.year(),
                                utc_now.month(),
                                utc_now.day(),
                            )
                            .unwrap(),
                        }),
                    ));
                }

                let state: Option<State> = match get_filter_choice_from_user(
                    State::iter().map(|state| state.to_string()).collect(),
                    "Filter by state? ",
                ) {
                    Some(state_choice) => match state_choice {
                        FilterChoice::Any => None,
                        FilterChoice::Other(choice) => Some(State::from_str(&choice).unwrap()),
                    },
                    None => None,
                };

                if state.is_some() {
                    println!("Fetching conversations...");
                    let conversations = twilio
                        .conversations()
                        .list(start_date, end_date, state)
                        .unwrap_or_else(|error| panic!("{}", error));

                    if conversations.len() == 0 {
                        println!("No conversations found.");
                        println!();
                    } else {
                        println!("Found {} conversations.", conversations.len());
                        conversations
                            .into_iter()
                            .for_each(|conv| match conv.unique_name {
                                Some(unique_name) => {
                                    println!("({}) {} - {}", conv.sid, unique_name, conv.state)
                                }
                                None => println!("{} - {}", conv.sid, conv.state),
                            });
                    }
                }
            }
            Action::DeleteConversation => {
                let conversation_sid =
                    Text::new("Please provide a conversation SID, or unique name:")
                        .with_placeholder("CH...")
                        .with_validator(|val: &str| {
                            if val.starts_with("CH") && val.len() == 34 {
                                Ok(Validation::Valid)
                            } else {
                                Ok(Validation::Invalid(
                                    "Conversation SID should be 34 characters in length".into(),
                                ))
                            }
                        })
                        .prompt()
                        .unwrap();

                if Confirm::new("Are you sure to wish to delete the Conversation? (Yes / No)")
                    .prompt()
                    .unwrap()
                {
                    let delete_result = twilio.conversations().delete(&conversation_sid);

                    if delete_result.is_ok() {
                        println!("Conversation deleted.");
                        println!("");
                    } else {
                        let delete_error = delete_result.unwrap_err();
                        match delete_error.kind {
                            ErrorKind::TwilioError(twilio_error) => {
                                if twilio_error.status == 404 {
                                    println!(
                                        "A Conversation with SID '{}' was not found.",
                                        &conversation_sid
                                    );
                                    println!("");
                                } else {
                                    panic!("{}", twilio_error)
                                }
                            }
                            _ => panic!("{}", delete_error),
                        };
                    }
                } else {
                    println!("Operation canceled. No changes were made.");
                }
            }
            Action::DeleteAllConversations => {
                if Confirm::new("Are you sure to wish to delete **all** Conversations? (Yes / No)")
                    .prompt()
                    .unwrap()
                {
                    if Confirm::new("Are you double sure? There is no going back. (Yes / No)")
                        .prompt()
                        .unwrap()
                    {
                        println!("Proceeding with deletion. Please wait...");
                        twilio
                            .conversations()
                            .delete_all(None)
                            .unwrap_or_else(|error| panic!("{}", error));

                        println!("All conversations deleleted.");
                        println!("");
                    } else {
                        println!("Operation canceled. No changes were made.");
                        println!("");
                    }
                } else {
                    println!("Operation canceled. No changes were made.");
                    println!("");
                }
            }
            Action::Back => break,
            Action::Exit => process::exit(0),
        }
    }
}

struct DateRange {
    minimum_date: chrono::NaiveDate,
    maximum_date: chrono::NaiveDate,
}

fn get_date_from_user(message: &str, date_range: Option<DateRange>) -> NaiveDate {
    let selected_date = match date_range {
        Some(date_range) => {
            let date_selection = DateSelect::new(message)
                .with_min_date(
                    chrono::NaiveDate::from_ymd_opt(
                        date_range.minimum_date.year(),
                        date_range.minimum_date.month(),
                        date_range.minimum_date.day(),
                    )
                    .unwrap(),
                )
                .with_max_date(
                    chrono::NaiveDate::from_ymd_opt(
                        date_range.maximum_date.year(),
                        date_range.maximum_date.month(),
                        date_range.maximum_date.day(),
                    )
                    .unwrap(),
                )
                .with_week_start(chrono::Weekday::Mon)
                .prompt();
            date_selection.unwrap()
        }
        None => {
            let date_selection = DateSelect::new(message)
                .with_week_start(chrono::Weekday::Mon)
                .prompt();
            date_selection.unwrap()
        }
    };

    selected_date
}
