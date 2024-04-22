use std::process;

use inquire::{Confirm, Select};
use strum::IntoEnumIterator;
use strum_macros::{Display, EnumIter, EnumString};
use twilly::{sync::services::SyncService, Client};
use twilly_cli::{get_action_choice_from_user, prompt_user, prompt_user_selection, ActionChoice};

use crate::sync::listitems;

#[derive(Debug, Clone, Display, EnumIter, EnumString)]
pub enum Action {
    #[strum(to_string = "List Items")]
    ListItem,
    #[strum(to_string = "List Details")]
    ListDetails,
    Delete,
    Back,
    Exit,
}

pub async fn choose_list_action(twilio: &Client, sync_service: &SyncService) {
    let mut sync_lists = twilio
        .sync()
        .service(&sync_service.sid)
        .lists()
        .list()
        .await
        .unwrap_or_else(|error| panic!("{}", error));

    if sync_lists.is_empty() {
        println!("No Sync Lists found.");
        return;
    }

    println!("Found {} Sync Lists.", sync_lists.len());

    let mut selected_sync_list_index: Option<usize> = None;
    loop {
        let selected_sync_list = if let Some(index) = selected_sync_list_index {
            &mut sync_lists[index]
        } else if let Some(action_choice) = get_action_choice_from_user(
            sync_lists
                .iter()
                .map(|list| format!("({}) {}", list.sid, list.unique_name))
                .collect::<Vec<String>>(),
            "Choose a Sync List: ",
        ) {
            match action_choice {
                ActionChoice::Back => {
                    break;
                }
                ActionChoice::Exit => process::exit(0),
                ActionChoice::Other(choice) => {
                    let sync_list_position = sync_lists
                        .iter()
                        .position(|list| list.sid == choice[1..35])
                        .expect("Could not find Sync List in existing Sync Map list");

                    selected_sync_list_index = Some(sync_list_position);
                    &mut sync_lists[sync_list_position]
                }
            }
        } else {
            break;
        };

        let options: Vec<Action> = Action::iter().collect();
        let resource_selection_prompt = Select::new("Select an action:", options.clone());
        if let Some(resource) = prompt_user_selection(resource_selection_prompt) {
            match resource {
                Action::ListItem => {
                    listitems::choose_list_item_action(twilio, sync_service, selected_sync_list)
                        .await;
                }

                Action::ListDetails => {
                    println!("{:#?}", selected_sync_list);
                    println!();
                }
                Action::Delete => {
                    let confirm_prompt =
                        Confirm::new("Are you sure you wish to delete the Sync List?")
                            .with_placeholder("N")
                            .with_default(false);
                    let confirmation = prompt_user(confirm_prompt);
                    if confirmation.is_some() && confirmation.unwrap() {
                        println!("Deleting Sync List...");
                        twilio
                            .sync()
                            .service(&sync_service.sid)
                            .list(&selected_sync_list.sid)
                            .delete()
                            .await
                            .unwrap_or_else(|error| panic!("{}", error));
                        sync_lists.remove(
                            selected_sync_list_index
                                .expect("Could not find Sync List in existing Sync Maps list"),
                        );
                        println!("Sync List deleted.");
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
