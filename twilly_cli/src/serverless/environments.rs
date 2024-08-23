mod logs;

use std::process;

use inquire::{Confirm, Select};
use strum::IntoEnumIterator;
use strum_macros::{Display, EnumIter, EnumString};
use twilly::{serverless::services::ServerlessService, Client};
use twilly_cli::{get_action_choice_from_user, prompt_user, prompt_user_selection, ActionChoice};

#[derive(Debug, Clone, Display, EnumIter, EnumString)]
pub enum Action {
    // #[strum(to_string = "List Items")]
    // ListItem,
    #[strum(to_string = "List Details")]
    ListDetails,
    Logs,
    Delete,
    Back,
    Exit,
}

pub async fn choose_environment_action(twilio: &Client, serverless_service: &ServerlessService) {
    let mut serverless_environments = twilio
        .serverless()
        .service(&serverless_service.sid)
        .environments()
        .list()
        .await
        .unwrap_or_else(|error| panic!("{}", error));

    if serverless_environments.is_empty() {
        println!("No Serverless Environments found.");
        return;
    }

    println!(
        "Found {} Serverless Environments.",
        serverless_environments.len()
    );

    let mut selected_serverless_environment_index: Option<usize> = None;
    loop {
        let selected_serverless_environment = if let Some(index) =
            selected_serverless_environment_index
        {
            &mut serverless_environments[index]
        } else if let Some(action_choice) = get_action_choice_from_user(
            serverless_environments
                .iter()
                .map(|environment| format!("({}) {}", environment.sid, environment.unique_name))
                .collect::<Vec<String>>(),
            "Choose a Serverless Environment: ",
        ) {
            match action_choice {
                ActionChoice::Back => {
                    break;
                }
                ActionChoice::Exit => process::exit(0),
                ActionChoice::Other(choice) => {
                    let serverless_environment_position = serverless_environments
                        .iter()
                        .position(|list| list.sid == choice[1..35])
                        .expect("Could not find Serverless Environment in existing Serverless Environment list");

                    selected_serverless_environment_index = Some(serverless_environment_position);
                    &mut serverless_environments[serverless_environment_position]
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
                    println!("{:#?}", selected_serverless_environment);
                    println!();
                }
                Action::Logs => {
                    logs::choose_log_action(
                        twilio,
                        serverless_service,
                        selected_serverless_environment,
                    )
                    .await
                }
                Action::Delete => {
                    let confirm_prompt =
                        Confirm::new("Are you sure you wish to delete the Serverless Environment?")
                            .with_placeholder("N")
                            .with_default(false);
                    let confirmation = prompt_user(confirm_prompt);
                    if confirmation.is_some() && confirmation.unwrap() {
                        println!("Deleting Serverless Environment...");
                        twilio
                            .sync()
                            .service(&serverless_service.sid)
                            .list(&selected_serverless_environment.sid)
                            .delete()
                            .await
                            .unwrap_or_else(|error| panic!("{}", error));
                        serverless_environments.remove(
                            selected_serverless_environment_index
                                .expect("Could not find Serverless Environment in existing Serverless Environment list"),
                        );
                        println!("Serverless Environment deleted.");
                        println!();
                        break;
                    }
                }
                Action::Back => {
                    break;
                }
                Action::Exit => process::exit(0),
            }
        }
    }
}
