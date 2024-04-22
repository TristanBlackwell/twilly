use std::process;

use inquire::{Confirm, Select};
use strum::IntoEnumIterator;
use strum_macros::{Display, EnumIter, EnumString};
use twilly::{
    sync::{mapitems::ListParams, maps::SyncMap, services::SyncService},
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

pub async fn choose_map_item_action(twilio: &Client, sync_service: &SyncService, map: &SyncMap) {
    let mut sync_map_items = twilio
        .sync()
        .service(&sync_service.sid)
        .map(&map.sid)
        .mapitems()
        .list(ListParams {
            order: None,
            bounds: None,
            from: None,
        })
        .await
        .unwrap_or_else(|error| panic!("{}", error));

    if sync_map_items.is_empty() {
        println!("No Sync Map items found.");
        return;
    }

    println!("Found {} Sync Maps items.", sync_map_items.len());

    let mut selected_sync_map_index: Option<usize> = None;
    loop {
        let selected_sync_map_item = if let Some(index) = selected_sync_map_index {
            &mut sync_map_items[index]
        } else if let Some(action_choice) = get_action_choice_from_user(
            sync_map_items
                .iter()
                .map(|map_item| map_item.key.to_string())
                .collect::<Vec<String>>(),
            "Choose a Sync Map item: ",
        ) {
            match action_choice {
                ActionChoice::Back => {
                    break;
                }
                ActionChoice::Exit => process::exit(0),
                ActionChoice::Other(choice) => {
                    let sync_map_position = sync_map_items
                        .iter()
                        .position(|map| map.key == choice)
                        .expect("Could not find Sync Map in existing Sync Map list");

                    selected_sync_map_index = Some(sync_map_position);
                    &mut sync_map_items[sync_map_position]
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
                    println!("{:#?}", selected_sync_map_item);
                    println!();
                }
                Action::Delete => {
                    let confirm_prompt =
                        Confirm::new("Are you sure you wish to delete the Sync Map item?")
                            .with_placeholder("N")
                            .with_default(false);
                    let confirmation = prompt_user(confirm_prompt);
                    if confirmation.is_some() && confirmation.unwrap() {
                        println!("Deleting Sync Map item...");
                        twilio
                            .sync()
                            .service(&sync_service.sid)
                            .map(&map.sid)
                            .mapitem(&selected_sync_map_item.key)
                            .delete()
                            .await
                            .unwrap_or_else(|error| panic!("{}", error));
                        sync_map_items.remove(selected_sync_map_index.expect(
                            "Could not find Sync Map item in existing Sync Map items list",
                        ));
                        println!("Sync Map item deleted.");
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
