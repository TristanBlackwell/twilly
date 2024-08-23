use std::process;

use inquire::Select;
use strum::IntoEnumIterator;
use strum_macros::{Display, EnumIter, EnumString};
use twilly::{
    serverless::{environments::ServerlessEnvironment, services::ServerlessService},
    Client,
};
use twilly_cli::{get_action_choice_from_user, prompt_user_selection, ActionChoice};

#[derive(Debug, Clone, Display, EnumIter, EnumString)]
pub enum Action {
    // #[strum(to_string = "List Items")]
    // ListItem,
    #[strum(to_string = "List Details")]
    ListDetails,
    Back,
    Exit,
}

pub async fn choose_log_action(
    twilio: &Client,
    serverless_service: &ServerlessService,
    serverless_environment: &ServerlessEnvironment,
) {
    let mut serverless_logs = twilio
        .serverless()
        .service(&serverless_service.sid)
        .environment(&serverless_environment.sid)
        .logs()
        .list()
        .await
        .unwrap_or_else(|error| panic!("{}", error));

    if serverless_logs.is_empty() {
        println!("No Serverless Logs found.");
        return;
    }

    println!("Found {} Serverless Logs.", serverless_logs.len());

    // Sort date descending (latest first)
    serverless_logs.sort_by(|a, b| b.date_created.cmp(&a.date_created));

    let mut selected_serverless_log_index: Option<usize> = None;
    loop {
        let selected_serverless_log = if let Some(index) = selected_serverless_log_index {
            &mut serverless_logs[index]
        } else if let Some(action_choice) = get_action_choice_from_user(
            serverless_logs
                .iter()
                .map(|log| format!("({}) {} - {}", log.sid, log.date_created, log.message))
                .collect::<Vec<String>>(),
            "Choose a Serverless Log: ",
        ) {
            match action_choice {
                ActionChoice::Back => {
                    break;
                }
                ActionChoice::Exit => process::exit(0),
                ActionChoice::Other(choice) => {
                    let serverless_log_position = serverless_logs
                        .iter()
                        .position(|list| list.sid == choice[1..35])
                        .expect("Could not find Serverless Log in existing Serverless Log list");

                    selected_serverless_log_index = Some(serverless_log_position);
                    &mut serverless_logs[serverless_log_position]
                }
            }
        } else {
            break;
        };

        let options: Vec<Action> = Action::iter().collect();
        let resource_selection_prompt = Select::new("Select an action:", options.clone());
        if let Some(resource) = prompt_user_selection(resource_selection_prompt) {
            match resource {
                Action::ListDetails => {
                    println!("{:#?}", selected_serverless_log);
                    println!();
                }
                Action::Back => {
                    break;
                }
                Action::Exit => process::exit(0),
            }
        }
    }
}
