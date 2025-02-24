mod environments;

use std::{process, sync::Arc};

use inquire::{validator::Validation, Confirm, Select, Text};
use regex::Regex;
use strum::IntoEnumIterator;
use strum_macros::{Display, EnumIter, EnumString};
use twilly::{serverless::services::CreateOrUpdateParams, Client};
use twilly_cli::{get_action_choice_from_user, prompt_user, prompt_user_selection, ActionChoice};

#[derive(Debug, Clone, Display, EnumIter, EnumString)]
pub enum Action {
    #[strum(to_string = "List Details")]
    ListDetails,
    Environments,
    Delete,
    Back,
    Exit,
}

pub async fn choose_serverless_resource(twilio: &Client) {
    let mut serverless_services = twilio
        .serverless()
        .services()
        .list()
        .await
        .unwrap_or_else(|error| panic!("{}", error));

    if serverless_services.is_empty() {
        println!("No Serverless Services found.");
        return;
    }

    println!("Found {} Serverless Services.", serverless_services.len());

    let mut selected_serverless_service_index: Option<usize> = None;
    let unique_name_regex = Arc::new(Regex::new(r"^[a-zA-Z0-9-_]+$").unwrap());

    loop {
        let allowed_chars = Arc::clone(&unique_name_regex);

        let selected_serverless_service = if let Some(index) = selected_serverless_service_index {
            &mut serverless_services[index]
        } else {
            let mut existing_services = serverless_services
                .iter()
                .map(|service| format!("({}) {}", service.sid, service.unique_name))
                .collect::<Vec<String>>();
            existing_services.push("Create Serverless Service".into());
            if let Some(action_choice) =
                get_action_choice_from_user(existing_services, "Choose a Serverless Service: ")
            {
                match action_choice {
                    ActionChoice::Back => {
                        break;
                    }
                    ActionChoice::Exit => process::exit(0),
                    ActionChoice::Other(choice) => {
                        if choice == "Create Serverless Service" {
                            let unique_name_prompt = Text::new("Enter a unique name:")
                                .with_validator(|val: &str| {
                                    if val.len() <= 50 {
                                        Ok(Validation::Valid)
                                    } else {
                                        Ok(Validation::Invalid(
                                            "Unique name must be less than 50 characters".into(),
                                        ))
                                    }
                                })
                                .with_validator(move |val: &str| {
                                    let trimmed_name = val.trim();
                                    if !allowed_chars.is_match(trimmed_name) {
                                        return Ok(Validation::Invalid(
                                            "Name doesn't match required filter '^[a-zA-Z0-9-_]+$'"
                                                .into(),
                                        ));
                                    }

                                    Ok(Validation::Valid)
                                });

                            if let Some(unique_name) = prompt_user(unique_name_prompt) {
                                let friendly_name_prompt = Text::new(
                                    "Enter a friendly name (empty to use the unique name):",
                                );

                                if let Some(friendly_name) = prompt_user(friendly_name_prompt) {
                                    let mut friendly_name = friendly_name;
                                    if friendly_name.is_empty() {
                                        friendly_name = unique_name.clone()
                                    }

                                    let credentials_confirmation_prompt =
                                    Confirm::new("Would you like to include Twilio credentials for function invocations?")
                                        .with_placeholder("Y")
                                        .with_default(true);

                                    if let Some(credentials_confirmation) =
                                        prompt_user(credentials_confirmation_prompt)
                                    {
                                        let ui_editable_confirmation_prompt =
                                    Confirm::new("Would you like the service to be editable via the Console?")
                                        .with_placeholder("N")
                                        .with_default(false);

                                        if let Some(ui_editable_confirmation) =
                                            prompt_user(ui_editable_confirmation_prompt)
                                        {
                                            let serverless_service = twilio
                                                .serverless()
                                                .services()
                                                .create(CreateOrUpdateParams {
                                                    unique_name,
                                                    friendly_name,
                                                    include_credentials: Some(
                                                        credentials_confirmation,
                                                    ),
                                                    ui_editable: Some(ui_editable_confirmation),
                                                })
                                                .await
                                                .unwrap_or_else(|error| panic!("{}", error));
                                            serverless_services.push(serverless_service);
                                            selected_serverless_service_index =
                                                Some(serverless_services.len() - 1);
                                            &mut serverless_services
                                                [selected_serverless_service_index.unwrap()]
                                        } else {
                                            break;
                                        }
                                    } else {
                                        break;
                                    }
                                } else {
                                    break;
                                }
                            } else {
                                break;
                            }
                        } else {
                            let serverless_service_position = serverless_services
                                .iter()
                                .position(|serverless_service| {
                                    serverless_service.sid == choice[1..35]
                                })
                                .expect(
                                    "Could not find Serverless Service in existing Serverless Service list",
                                );

                            selected_serverless_service_index = Some(serverless_service_position);
                            &mut serverless_services[serverless_service_position]
                        }
                    }
                }
            } else {
                break;
            }
        };

        let options: Vec<Action> = Action::iter().collect();
        let resource_selection_prompt = Select::new("Select an action:", options.clone());
        if let Some(resource) = prompt_user_selection(resource_selection_prompt) {
            match resource {
                Action::ListDetails => {
                    println!("{:#?}", selected_serverless_service);
                    println!();
                }
                Action::Environments => {
                    environments::choose_environment_action(twilio, selected_serverless_service)
                        .await
                }
                Action::Delete => {
                    let confirm_prompt =
                        Confirm::new("Are you sure you wish to delete the Serverless Service?")
                            .with_placeholder("N")
                            .with_default(false);
                    let confirmation = prompt_user(confirm_prompt);
                    if confirmation.is_some() && confirmation.unwrap() {
                        println!("Deleting Serverless Service...");
                        twilio
                            .serverless()
                            .service(&selected_serverless_service.sid)
                            .delete()
                            .await
                            .unwrap_or_else(|error| panic!("{}", error));
                        serverless_services.remove(
                            selected_serverless_service_index.expect(
                                "Could not find Serverless Service in existing Serverless Services list",
                            ),
                        );
                        println!("Serverless Service deleted.");
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
