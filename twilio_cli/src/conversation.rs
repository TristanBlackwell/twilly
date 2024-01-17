use std::process;

use chrono::Datelike;
use inquire::{validator::Validation, Confirm, DateSelect, Select, Text};
use strum::IntoEnumIterator;
use strum_macros::{Display, EnumIter, EnumString};
use twilio_rust::Client;

#[derive(Clone, Display, EnumIter, EnumString)]
pub enum Action {
    #[strum(serialize = "Get conversation")]
    GetConversation,
    #[strum(serialize = "List Conversations")]
    ListConversations,
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
                let conversation = twilio
                    .conversations()
                    .get(&conversation_sid)
                    .unwrap_or_else(|error| panic!("{}", error));
                println!("{:?}", conversation);
            }
            Action::ListConversations => {
                println!("Fetching conversations...");
                let mut start_date: Option<&chrono::NaiveDate> = None;
                let mut end_date: Option<&chrono::NaiveDate> = None;
                // if Confirm::new("Would you like to filter between specified dates? (Yes / No)")
                //     .prompt()
                //     .unwrap()
                // {
                //     let utc_now = chrono::Utc::now();
                //     let utc_one_year_ago = utc_now - chrono::Duration::days(365);
                //     let start_date_selection = DateSelect::new("Choose a start date:")
                //         .with_min_date(chrono::NaiveDate::from_ymd(
                //             utc_one_year_ago.year(),
                //             utc_one_year_ago.month(),
                //             utc_one_year_ago.day(),
                //         ))
                //         .with_max_date(chrono::NaiveDate::from_ymd(
                //             utc_now.year(),
                //             utc_now.month(),
                //             utc_now.day(),
                //         ))
                //         .with_week_start(chrono::Weekday::Mon)
                //         .with_help_message(
                //             "You can retrieve Conversations up to a year in the past.",
                //         )
                //         .prompt();
                //     let chosen_start_date = start_date_selection.unwrap();
                //     start_date = Some(&chosen_start_date);

                //     let end_date_selection = DateSelect::new("Choose an end date:")
                //         .with_min_date(chrono::NaiveDate::from_ymd(
                //             start_date.unwrap().year_ce().1.try_into().unwrap(),
                //             start_date.unwrap().month0() + 1,
                //             start_date.unwrap().day0(),
                //         ))
                //         .with_max_date(chrono::NaiveDate::from_ymd(
                //             utc_now.year(),
                //             utc_now.month(),
                //             utc_now.day(),
                //         ))
                //         .with_week_start(chrono::Weekday::Mon)
                //         .with_help_message(
                //             "You can retrieve Conversations up to a year in the past.",
                //         )
                //         .prompt();
                //     end_date = Some(&end_date_selection.unwrap());
                // }

                // TODO: State option

                let conversations = twilio
                    .conversations()
                    .list(None, None, None)
                    .unwrap_or_else(|error| panic!("{}", error));

                if conversations.len() == 0 {
                    println!("No conversations found.");
                    println!();
                } else {
                    println!("Found {} conversations.", conversations.len());
                    conversations
                        .into_iter()
                        .for_each(|conv| println!("{} - {}", conv.sid, conv.state));
                }
            }
            Action::Back => break,
            Action::Exit => process::exit(0),
        }
    }
}
