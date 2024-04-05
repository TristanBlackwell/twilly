mod documents;
mod listitems;
mod lists;
mod mapitems;
mod maps;

use std::process;

use inquire::{Confirm, Select, Text};
use strum::IntoEnumIterator;
use strum_macros::{Display, EnumIter, EnumString};
use twilly::{sync::services::CreateOrUpdateParams, Client};
use twilly_cli::{get_action_choice_from_user, prompt_user, prompt_user_selection, ActionChoice};

#[derive(Debug, Clone, Display, EnumIter, EnumString)]
pub enum Action {
    #[strum(to_string = "Documents")]
    Document,
    #[strum(to_string = "Maps")]
    Map,
    #[strum(to_string = "Lists")]
    List,
    #[strum(to_string = "List Details")]
    ListDetails,
    Delete,
    Back,
    Exit,
}

pub async fn choose_sync_resource(twilio: &Client) {
    let mut sync_services = twilio
        .sync()
        .services()
        .list()
        .await
        .unwrap_or_else(|error| panic!("{}", error));

    if sync_services.len() == 0 {
        println!("No Sync Services found.");
        return;
    }

    println!("Found {} Sync Services.", sync_services.len());

    let mut selected_sync_service_index: Option<usize> = None;
    loop {
        let selected_sync_service = if let Some(index) = selected_sync_service_index {
            &mut sync_services[index]
        } else {
            let mut existing_services = sync_services
                .iter()
                .map(|service| match &service.unique_name {
                    Some(unique_name) => format!("({}) {}", service.sid, unique_name),
                    None => match &service.friendly_name {
                        Some(friendly_name) => format!("({}) {}", service.sid, friendly_name),
                        None => format!("{}", service.sid),
                    },
                })
                .collect::<Vec<String>>();
            existing_services.push("Create Sync Service".into());
            if let Some(action_choice) =
                get_action_choice_from_user(existing_services, "Choose a Sync Service: ")
            {
                match action_choice {
                    ActionChoice::Back => {
                        break;
                    }
                    ActionChoice::Exit => process::exit(0),
                    ActionChoice::Other(choice) => {
                        if choice == "Create Sync Service" {
                            let friendly_name_prompt =
                                Text::new("Enter a friendly name (empty for default):");

                            if let Some(friendly_name) = prompt_user(friendly_name_prompt) {
                                let acl_confirmation_prompt =
                                    Confirm::new("Would you like to enable ACL? (Yes / No)");

                                if let Some(acl_confirmation) = prompt_user(acl_confirmation_prompt)
                                {
                                    let sync_service = twilio
                                        .sync()
                                        .services()
                                        .create(CreateOrUpdateParams {
                                            friendly_name: Some(friendly_name),
                                            acl_enabled: Some(acl_confirmation),
                                            reachability_debouncing_enabled: None,
                                            reachability_debouncing_window: None,
                                            reachability_webhooks_enabled: None,
                                            webhooks_from_rest_enabled: None,
                                            webhook_url: None,
                                        })
                                        .await
                                        .unwrap_or_else(|error| panic!("{}", error));
                                    sync_services.push(sync_service);
                                    selected_sync_service_index = Some(sync_services.len() - 1);
                                    &mut sync_services[selected_sync_service_index.unwrap()]
                                } else {
                                    break;
                                }
                            } else {
                                break;
                            }
                        } else {
                            let sync_service_position = sync_services
                                .iter()
                                .position(|sync_service| sync_service.sid == choice[1..35])
                                .expect(
                                    "Could not find Sync Service in existing Sync Service list",
                                );

                            selected_sync_service_index = Some(sync_service_position);
                            &mut sync_services[sync_service_position]
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
                Action::Document => {
                    documents::choose_document_action(&twilio, selected_sync_service).await
                }
                Action::Map => maps::choose_map_action(&twilio, selected_sync_service).await,
                Action::List => lists::choose_list_action(&twilio, selected_sync_service).await,
                Action::ListDetails => {
                    println!("{:#?}", selected_sync_service);
                    println!()
                }
                Action::Delete => {
                    let confirm_prompt =
                        Confirm::new("Are you sure to wish to delete the Sync Service? (Yes / No)");
                    let confirmation = prompt_user(confirm_prompt);
                    if confirmation.is_some() && confirmation.unwrap() == true {
                        println!("Deleting Sync Service...");
                        twilio
                            .sync()
                            .service(&selected_sync_service.sid)
                            .delete()
                            .await
                            .unwrap_or_else(|error| panic!("{}", error));
                        sync_services.remove(
                            selected_sync_service_index.expect(
                                "Could not find Sync Service in existing Sync Services list",
                            ),
                        );
                        println!("Sync Service deleted.");
                        println!();
                        break;
                    }
                }
                Action::Back => break,
                Action::Exit => process::exit(0),
            }
        }
    }
}
