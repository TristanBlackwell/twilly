use std::{process, str::FromStr};

use chrono::Datelike;
use inquire::{validator::Validation, Confirm, DateSelect, Select, Text};
use strum::IntoEnumIterator;
use strum_macros::{Display, EnumIter, EnumString};
use twilio_cli::{
    get_action_choice_from_user, get_filter_choice_from_user, prompt_user, prompt_user_selection,
    ActionChoice, FilterChoice,
};
use twilio_rust::{
    conversation::{Conversation, State, UpdateConversation},
    Client, ErrorKind, TwilioError,
};

#[derive(Clone, Display, EnumIter, EnumString)]
pub enum Action {
    #[strum(to_string = "Get conversation")]
    GetConversation,
    #[strum(to_string = "List Conversations")]
    ListConversations,
    #[strum(to_string = "Delete Conversation")]
    DeleteConversation,
    #[strum(to_string = "Delete all Conversations")]
    DeleteAllConversations,
    Back,
    Exit,
}

pub fn choose_conversation_action(twilio: &Client) {
    let options: Vec<Action> = Action::iter().collect();

    loop {
        let action_selection_prompt = Select::new("Select an action:", options.clone());

        if let Some(action) = prompt_user_selection(action_selection_prompt) {
            match action {
                Action::GetConversation => {
                    let conversation_sid_prompt =
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
                            });

                    if let Some(conversation_sid) = prompt_user(conversation_sid_prompt) {
                        match twilio.conversations().get(&conversation_sid) {
                            Ok(conversation) => {
                                println!("Conversation found.");
                                println!();

                                if let Some(action_choice) = get_action_choice_from_user(
                                    vec![String::from("List Details"), String::from("Delete")],
                                    "Select an action: ",
                                ) {
                                    match action_choice {
                                        ActionChoice::Back => break,
                                        ActionChoice::Exit => process::exit(0),
                                        ActionChoice::Other(choice) => match choice.as_str() {
                                            "List Details" => {
                                                println!("{:#?}", conversation);
                                                println!();
                                            }
                                            "Delete" => {
                                                let confirm_prompt = Confirm::new(
                                                "Are you sure to wish to delete the Conversation? (Yes / No)",
                                            );
                                                let confirmation = prompt_user(confirm_prompt);
                                                if confirmation.is_some()
                                                    && confirmation.unwrap() == true
                                                {
                                                    println!("Deleting Conversation...");
                                                    twilio
                                                        .conversations()
                                                        .delete(&conversation_sid)
                                                        .unwrap_or_else(|error| {
                                                            panic!("{}", error)
                                                        });
                                                    println!("Conversation deleted.");
                                                    println!();
                                                }
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
                                        println!(
                                            "A Conversation with SID '{}' was not found.",
                                            &conversation_sid
                                        );
                                        println!("");
                                    } else {
                                        panic!("{}", twilio_error)
                                    }
                                }
                                _ => panic!("{}", error),
                            },
                        }
                    }
                }
                Action::ListConversations => {
                    let mut start_date: Option<chrono::NaiveDate> = None;
                    let mut end_date: Option<chrono::NaiveDate> = None;

                    let mut user_filtered_dates = false;

                    let filter_dates_prompt = Confirm::new(
                        "Would you like to filter between specified dates? (Yes / No)",
                    );

                    if let Some(decision) = prompt_user(filter_dates_prompt) {
                        if decision == true {
                            user_filtered_dates = true;
                            let utc_now = chrono::Utc::now();
                            let utc_one_year_ago = utc_now - chrono::Duration::days(365);
                            if let Some(user_start_date) = get_date_from_user(
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
                            ) {
                                start_date = Some(user_start_date);
                                end_date = get_date_from_user(
                                    "Choose an end date:",
                                    Some(DateRange {
                                        minimum_date: chrono::NaiveDate::from_ymd_opt(
                                            user_start_date.year_ce().1.try_into().unwrap(),
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
                                );
                            }
                        }
                    }

                    // Only continue if the user filtered by dates *and* provided both options.
                    // If they didn't then they must of cancelled the operation.
                    if !user_filtered_dates
                        || user_filtered_dates && (start_date.is_some() && end_date.is_some())
                    {
                        if let Some(filter_choice) = get_filter_choice_from_user(
                            State::iter().map(|state| state.to_string()).collect(),
                            "Filter by state? ",
                        ) {
                            let state = match filter_choice {
                                FilterChoice::Any => None,
                                FilterChoice::Other(choice) => {
                                    Some(State::from_str(&choice).unwrap())
                                }
                            };

                            println!("Fetching conversations...");
                            let mut conversations = twilio
                                .conversations()
                                .list(start_date, end_date, state)
                                .unwrap_or_else(|error| panic!("{}", error));

                            let number_of_conversations = conversations.len();

                            if number_of_conversations == 0 {
                                println!("No conversations found.");
                                println!();
                            } else {
                                println!("Found {} conversations.", number_of_conversations);

                                // Stores the index of the conversation the user is currently interacting
                                // with. For the first loop this is certainly `None`.
                                let mut selected_conversation_index: Option<usize> = None;
                                loop {
                                    // If we know the index (a.k.a it hasn't been cleared by some other operation)
                                    // then use this conversation otherwise let the user choice.
                                    let selected_conversation = if let Some(index) =
                                        selected_conversation_index
                                    {
                                        &mut conversations[index]
                                    } else {
                                        if let Some(action_choice) = get_action_choice_from_user(
                                            conversations
                                                .iter()
                                                .map(|conv| {
                                                    let display_name = match &conv.unique_name {
                                                        Some(unique_name) => format!(
                                                            "({}) {} - {}",
                                                            conv.sid, unique_name, conv.state
                                                        ),
                                                        None => {
                                                            format!("{} - {}", conv.sid, conv.state)
                                                        }
                                                    };
                                                    display_name
                                                })
                                                .collect::<Vec<String>>(),
                                            "Conversations: ",
                                        ) {
                                            match action_choice {
                                                ActionChoice::Back => {
                                                    break;
                                                }
                                                ActionChoice::Exit => process::exit(0),
                                                ActionChoice::Other(choice) => {
                                                    let conversation_position = conversations
                                                        .iter()
                                                        .position(|conv| conv.sid == choice[..34])
                                                        .expect("Could not find conversation in existing conversation list");

                                                    selected_conversation_index =
                                                        Some(conversation_position);
                                                    &mut conversations[conversation_position]
                                                }
                                            }
                                        } else {
                                            break;
                                        }
                                    };

                                    match selected_conversation.state {
                                        State::Closed => loop {
                                            if let Some(conversation_action) =
                                                get_action_choice_from_user(
                                                    vec![
                                                        String::from("List details"),
                                                        String::from("Delete"),
                                                    ],
                                                    "Select an action: ",
                                                )
                                            {
                                                match conversation_action {
                                                    ActionChoice::Back => {
                                                        selected_conversation_index = None;
                                                        break;
                                                    }
                                                    ActionChoice::Exit => process::exit(0),
                                                    ActionChoice::Other(choice) => match choice
                                                        .as_str()
                                                    {
                                                        "List details" => {
                                                            println!(
                                                                "{:#?}",
                                                                selected_conversation
                                                            );
                                                            println!();
                                                        }
                                                        "Delete" => {
                                                            delete_conversation(
                                                                twilio,
                                                                &selected_conversation.sid,
                                                            );
                                                            conversations.remove(
                                                                selected_conversation_index
                                                                    .expect("Could not find conversation in existing conversation list"),
                                                            );
                                                            selected_conversation_index = None;
                                                            break;
                                                        }
                                                        _ => {
                                                            println!("Unknown action '{}'", choice)
                                                        }
                                                    },
                                                }
                                            } else {
                                                selected_conversation_index = None;
                                                break;
                                            }
                                        },
                                        State::Inactive => loop {
                                            if let Some(conversation_action) =
                                                get_action_choice_from_user(
                                                    vec![
                                                        String::from("List details"),
                                                        String::from("Re-activate"),
                                                        String::from("Delete"),
                                                    ],
                                                    "Select an action: ",
                                                )
                                            {
                                                match conversation_action {
                                                    ActionChoice::Back => {
                                                        selected_conversation_index = None;
                                                        break;
                                                    }
                                                    ActionChoice::Exit => process::exit(0),
                                                    ActionChoice::Other(choice) => match choice
                                                        .as_str()
                                                    {
                                                        "List details" => {
                                                            println!(
                                                                "{:#?}",
                                                                selected_conversation
                                                            );
                                                            println!();
                                                        }
                                                        "Re-activate" => {
                                                            let updated_conversation =
                                                                update_conversation(
                                                                    twilio,
                                                                    &selected_conversation.sid,
                                                                    UpdateConversation {
                                                                        state: Some(State::Active),
                                                                        friendly_name: None,
                                                                        unique_name: None,
                                                                        attributes: None,
                                                                        timers: None,
                                                                    },
                                                                );
                                                            conversations
                                                                [selected_conversation_index
                                                                    .expect("Could not find conversation in existing conversation list")] =
                                                                updated_conversation;
                                                            break;
                                                        }
                                                        "Delete" => {
                                                            delete_conversation(
                                                                twilio,
                                                                &selected_conversation.sid,
                                                            );
                                                            conversations.remove(
                                                                selected_conversation_index
                                                                    .expect("Could not find conversation in existing conversation list"),
                                                            );
                                                            selected_conversation_index = None;
                                                            break;
                                                        }
                                                        _ => {
                                                            println!("Unknown action '{}'", choice)
                                                        }
                                                    },
                                                }
                                            } else {
                                                selected_conversation_index = None;
                                                break;
                                            }
                                        },
                                        State::Active => loop {
                                            if let Some(conversation_action) =
                                                get_action_choice_from_user(
                                                    vec![
                                                        String::from("List details"),
                                                        String::from("De-activate"),
                                                        String::from("Delete"),
                                                    ],
                                                    "Select an action: ",
                                                )
                                            {
                                                match conversation_action {
                                                    ActionChoice::Back => {
                                                        selected_conversation_index = None;
                                                        break;
                                                    }
                                                    ActionChoice::Exit => process::exit(0),
                                                    ActionChoice::Other(choice) => match choice
                                                        .as_str()
                                                    {
                                                        "List details" => {
                                                            println!(
                                                                "{:#?}",
                                                                selected_conversation
                                                            );
                                                            println!();
                                                        }
                                                        "De-activate" => {
                                                            let updated_conversation =
                                                                update_conversation(
                                                                    twilio,
                                                                    &selected_conversation.sid,
                                                                    UpdateConversation {
                                                                        state: Some(
                                                                            State::Inactive,
                                                                        ),
                                                                        friendly_name: None,
                                                                        unique_name: None,
                                                                        attributes: None,
                                                                        timers: None,
                                                                    },
                                                                );
                                                            conversations
                                                                [selected_conversation_index
                                                                    .expect("Could not find conversation in existing conversation list")] =
                                                                updated_conversation;
                                                            break;
                                                        }
                                                        "Delete" => {
                                                            delete_conversation(
                                                                twilio,
                                                                &selected_conversation.sid,
                                                            );
                                                            conversations.remove(
                                                                selected_conversation_index
                                                                    .expect("Could not find conversation in existing conversation list"),
                                                            );
                                                            selected_conversation_index = None;
                                                            break;
                                                        }
                                                        _ => {
                                                            println!("Unknown action '{}'", choice)
                                                        }
                                                    },
                                                }
                                            }
                                        },
                                    }
                                }
                            }
                        }
                    }
                }
                Action::DeleteConversation => {
                    let conversation_sid_prompt =
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
                            });

                    if let Some(conversation_sid) = prompt_user(conversation_sid_prompt) {
                        delete_conversation(twilio, &conversation_sid);
                    } else {
                        println!("Operation canceled. No changes were made.");
                    }
                }
                Action::DeleteAllConversations => {
                    let first_confirmation_prompt = Confirm::new(
                        "Are you sure to wish to delete **all** Conversations? (Yes / No)",
                    );
                    let second_confirmation_prompt =
                        Confirm::new("Are you double sure? There is no going back. (Yes / No)");

                    if let Some(first_confirmation) = prompt_user(first_confirmation_prompt) {
                        if first_confirmation == true {
                            if let Some(second_confirmation) =
                                prompt_user(second_confirmation_prompt)
                            {
                                if second_confirmation == true {
                                    println!("Proceeding with deletion. Please wait...");
                                    let conversations = twilio
                                        .conversations()
                                        .list(None, None, None)
                                        .unwrap_or_else(|error| panic!("{}", error));

                                    conversations
                                        .into_iter()
                                        .try_for_each(|conversation| -> Result<(), TwilioError> {
                                            twilio.conversations().delete(&conversation.sid)
                                        })
                                        .unwrap_or_else(|error| panic!("{}", error));

                                    println!("All conversations deleted.");
                                    println!("");
                                    return;
                                }
                            }
                        }
                    }

                    println!("Operation canceled. No changes were made.");
                    println!("");
                }
                Action::Back => break,
                Action::Exit => process::exit(0),
            }
        } else {
            break;
        };
    }
}

/// Prompts the user for confirmation before deleting the conversation with
/// the SID provided. Will panic if the delete operation fails.
fn update_conversation(twilio: &Client, sid: &str, updates: UpdateConversation) -> Conversation {
    match twilio.conversations().update(sid, updates) {
        Ok(updated_conversation) => {
            println!("Conversation updated.");
            println!();

            return updated_conversation;
        }
        Err(error) => panic!("{}", error),
    }
}

/// Prompts the user for confirmation before deleting the conversation with
/// the SID provided. Will panic if the delete operation fails.
fn delete_conversation(twilio: &Client, sid: &str) {
    let confirmation_prompt =
        Confirm::new("Are you sure to wish to delete the Conversation? (Yes / No)");

    if let Some(confirmation) = prompt_user(confirmation_prompt) {
        if confirmation == true {
            match twilio.conversations().delete(&sid) {
                Ok(_) => {
                    println!("Conversation deleted.");
                    println!("");
                }
                Err(error) => match error.kind {
                    ErrorKind::TwilioError(twilio_error) => {
                        if twilio_error.status == 404 {
                            println!("A Conversation with SID '{}' was not found.", &sid);
                            println!("");
                        } else {
                            panic!("{}", twilio_error)
                        }
                    }
                    _ => panic!("{}", error),
                },
            }
        }
    }
}

struct DateRange {
    minimum_date: chrono::NaiveDate,
    maximum_date: chrono::NaiveDate,
}

fn get_date_from_user(message: &str, date_range: Option<DateRange>) -> Option<chrono::NaiveDate> {
    let selected_date = match date_range {
        Some(date_range) => {
            let date_selection_prompt = DateSelect::new(message)
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
                .with_week_start(chrono::Weekday::Mon);

            prompt_user(date_selection_prompt)
        }
        None => {
            let date_selection_prompt =
                DateSelect::new(message).with_week_start(chrono::Weekday::Mon);
            prompt_user(date_selection_prompt)
        }
    };

    selected_date
}
