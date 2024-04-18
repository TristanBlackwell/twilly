use std::process;

use inquire::{Confirm, Select};
use strum::IntoEnumIterator;
use strum_macros::{Display, EnumIter, EnumString};
use twilly::{sync::services::SyncService, Client};
use twilly_cli::{get_action_choice_from_user, prompt_user, prompt_user_selection, ActionChoice};

use crate::sync::mapitems;

#[derive(Debug, Clone, Display, EnumIter, EnumString)]
pub enum Action {
    #[strum(to_string = "Map Items")]
    MapItem,
    #[strum(to_string = "List Details")]
    ListDetails,
    Delete,
    Back,
    Exit,
}

pub async fn choose_map_action(twilio: &Client, sync_service: &SyncService) {
    let mut sync_maps = twilio
        .sync()
        .service(&sync_service.sid)
        .maps()
        .list()
        .await
        .unwrap_or_else(|error| panic!("{}", error));

    if sync_maps.len() == 0 {
        println!("No Sync Maps found.");
        return;
    }

    println!("Found {} Sync Maps.", sync_maps.len());

    let mut selected_sync_map_index: Option<usize> = None;
    loop {
        let selected_sync_map = if let Some(index) = selected_sync_map_index {
            &mut sync_maps[index]
        } else {
            if let Some(action_choice) = get_action_choice_from_user(
                sync_maps
                    .iter()
                    .map(|map| format!("({}) {}", map.sid, map.unique_name))
                    .collect::<Vec<String>>(),
                "Choose a Sync Map: ",
            ) {
                match action_choice {
                    ActionChoice::Back => {
                        break;
                    }
                    ActionChoice::Exit => process::exit(0),
                    ActionChoice::Other(choice) => {
                        let sync_map_position = sync_maps
                            .iter()
                            .position(|map| map.sid == choice[1..35])
                            .expect("Could not find Sync Map in existing Sync Map list");

                        selected_sync_map_index = Some(sync_map_position);
                        &mut sync_maps[sync_map_position]
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
                Action::MapItem => {
                    mapitems::choose_map_item_action(&twilio, sync_service, &selected_sync_map)
                        .await;
                }

                Action::ListDetails => {
                    println!("{:#?}", selected_sync_map);
                    println!();
                }
                Action::Delete => {
                    let confirm_prompt =
                        Confirm::new("Are you sure to wish to delete the Sync Map?")
                            .with_placeholder("N")
                            .with_default(false);
                    let confirmation = prompt_user(confirm_prompt);
                    if confirmation.is_some() && confirmation.unwrap() == true {
                        println!("Deleting Sync Map...");
                        twilio
                            .sync()
                            .service(&sync_service.sid)
                            .map(&selected_sync_map.sid)
                            .delete()
                            .await
                            .unwrap_or_else(|error| panic!("{}", error));
                        sync_maps.remove(
                            selected_sync_map_index
                                .expect("Could not find Sync Map in existing Sync Maps list"),
                        );
                        println!("Sync Map deleted.");
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
