use std::process;

use inquire::{Confirm, Select};
use strum::IntoEnumIterator;
use strum_macros::{Display, EnumIter, EnumString};
use twilly::{
    sync::{listitems::ListParams, lists::SyncList, services::SyncService},
    Client,
};
use twilly_cli::{get_action_choice_from_user, prompt_user, prompt_user_selection, ActionChoice};

#[derive(Debug, Clone, Display, EnumIter, EnumString)]
pub enum Action {
    #[strum(to_string = "List Details")]
    ListDetails,
    Delete,
    Back,
    Exit,
}

pub async fn choose_list_item_action(twilio: &Client, sync_service: &SyncService, list: &SyncList) {
    let mut sync_list_items = twilio
        .sync()
        .service(&sync_service.sid)
        .list(&list.sid)
        .listitems()
        .list(ListParams {
            order: None,
            bounds: None,
            from: None,
        })
        .await
        .unwrap_or_else(|error| panic!("{}", error));

    if sync_list_items.len() == 0 {
        println!("No Sync List items found.");
        return;
    }

    println!("Found {} Sync List items.", sync_list_items.len());

    let mut selected_sync_list_index: Option<usize> = None;
    loop {
        let selected_sync_list_item = if let Some(index) = selected_sync_list_index {
            &mut sync_list_items[index]
        } else {
            if let Some(action_choice) = get_action_choice_from_user(
                sync_list_items
                    .iter()
                    .map(|list_item| format!("{}", list_item.index))
                    .collect::<Vec<String>>(),
                "Choose a Sync List item: ",
            ) {
                match action_choice {
                    ActionChoice::Back => {
                        break;
                    }
                    ActionChoice::Exit => process::exit(0),
                    ActionChoice::Other(choice) => {
                        let sync_list_position = sync_list_items
                            .iter()
                            .position(|list| list.index.to_string() == choice)
                            .expect("Could not find Sync List in existing Sync Map list");

                        selected_sync_list_index = Some(sync_list_position);
                        &mut sync_list_items[sync_list_position]
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
                    println!("{:#?}", selected_sync_list_item);
                    println!();
                }
                Action::Delete => {
                    let confirm_prompt =
                        Confirm::new("Are you sure to wish to delete the Sync List item?")
                            .with_placeholder("N")
                            .with_default(false);
                    let confirmation = prompt_user(confirm_prompt);
                    if confirmation.is_some() && confirmation.unwrap() == true {
                        println!("Deleting Sync Map item...");
                        twilio
                            .sync()
                            .service(&sync_service.sid)
                            .list(&list.sid)
                            .listitem(&selected_sync_list_item.index)
                            .delete()
                            .await
                            .unwrap_or_else(|error| panic!("{}", error));
                        sync_list_items.remove(selected_sync_list_index.expect(
                            "Could not find Sync List item in existing Sync List items list",
                        ));
                        println!("Sync List item deleted.");
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
