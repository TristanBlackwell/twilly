use std::process;

use inquire::{validator::Validation, Confirm, Select, Text};
use regex::Regex;
use strum::IntoEnumIterator;
use strum_macros::{Display, EnumIter, EnumString};
use twilly::{
    sync::{
        mapitems::{CreateParams as CreateMapItemParams, ListParams},
        maps::CreateParams as CreateMapParams,
        services::SyncService,
    },
    Client,
};
use twilly_cli::{get_action_choice_from_user, prompt_user, prompt_user_selection, ActionChoice};

use crate::sync::mapitems;

#[derive(Debug, Clone, Display, EnumIter, EnumString)]
pub enum Action {
    #[strum(to_string = "Map Items")]
    MapItem,
    #[strum(to_string = "List Details")]
    ListDetails,
    Rename,
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

    if sync_maps.is_empty() {
        println!("No Sync Maps found.");
        return;
    }

    println!("Found {} Sync Maps.", sync_maps.len());

    let mut selected_sync_map_index: Option<usize> = None;
    loop {
        let selected_sync_map = if let Some(index) = selected_sync_map_index {
            &mut sync_maps[index]
        } else if let Some(action_choice) = get_action_choice_from_user(
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
        };

        let options: Vec<Action> = Action::iter().collect();
        let resource_selection_prompt = Select::new("Select an action:", options.clone());
        if let Some(resource) = prompt_user_selection(resource_selection_prompt) {
            match resource {
                Action::MapItem => {
                    mapitems::choose_map_item_action(twilio, sync_service, selected_sync_map).await;
                }

                Action::ListDetails => {
                    println!("{:#?}", selected_sync_map);
                    println!();
                }
                Action::Rename => {
                    let get_name_prompt = Text::new(
                        "What would you like to rename this map to? Must be supported characters '^[a-zA-Z0-9-_]+$'"
                    ).with_validator(|val: &str| {
                        let allowed_chars = Regex::new(r"^[a-zA-Z0-9-_]+$").unwrap();
                        let trimmed_name = val.trim();
                        if !allowed_chars.is_match(trimmed_name) {
                            return Ok(Validation::Invalid("Name doesn't match required filter '^[a-zA-Z0-9-_]+$'".into()));
                        }

                        return Ok(Validation::Valid);
                    });
                    let get_name_result = prompt_user(get_name_prompt);

                    if let None = get_name_result {
                        break;
                    }

                    let trimmed_name = get_name_result.unwrap();

                    println!("Name confirmed '{trimmed_name}'");

                    let confirmation_message = "⚠️ Warning ⚠️

This process is non-reversible. We will:
    1. Create a temporary map to hold a copy of the map items
    2. Copy the items into the temporary map
    3. Confirm the copy worked
    4. Delete the original map
    5. Create a new map with your new name
    6. Copy all items from the temporary map into the new map

💡 Please note the TTL will not be preserved for the Map or items.

We will not delete the temporary map after the process has completed.
You can remove this using the CLI after you've confirmed the rename was successful.

Would you like to continue?";
                    let confirm_operation = Confirm::new(confirmation_message)
                        .with_placeholder("N")
                        .with_default(false);

                    let confirmation_result = prompt_user(confirm_operation);

                    match confirmation_result {
                        None => return,
                        Some(false) => return,
                        _ => (),
                    }

                    println!("Starting map rename process");

                    // create temporary map
                    println!("(1/6) Creating temporary map");
                    let temp_map_result = twilio
                        .sync()
                        .service(&sync_service.sid)
                        .maps()
                        .create(CreateMapParams {
                            ttl: None,
                            unique_name: Some(String::from(format!(
                                "temp-{}",
                                selected_sync_map.unique_name
                            ))),
                        })
                        .await;

                    if let Err(error) = temp_map_result {
                        println!("Errored: Failed to create map: {:?}", error);
                        break;
                    }

                    let temp_map = temp_map_result.unwrap();

                    // clone all items into temp map
                    println!("(2/6) Clone items into temporary map");
                    let fetch_items_result = twilio
                        .sync()
                        .service(&sync_service.sid)
                        .map(&selected_sync_map.sid)
                        .mapitems()
                        .list(ListParams {
                            bounds: None,
                            from: None,
                            order: None,
                        })
                        .await;

                    if let Err(error) = fetch_items_result {
                        println!("Errored: Failed to fetch current map items: {:?}", error);
                        break;
                    }

                    let items = fetch_items_result.unwrap();

                    for item in items.iter() {
                        let create_item_result = twilio
                            .sync()
                            .service(&sync_service.sid)
                            .map(&temp_map.sid)
                            .mapitems()
                            .create(CreateMapItemParams {
                                key: String::from(&item.key),
                                data: &item.data,
                                collection_ttl: None,
                                ttl: None,
                            })
                            .await;

                        if let Err(error) = create_item_result {
                            println!("Errored: Failed while taking copy of items: {:?}", error);
                            return;
                        }
                    }

                    // confirm copy
                    println!("(3/6) Confirm copy was successful");
                    let confirm_copy_message = Confirm::new("Copy completed. Please confirm the temporary map created correctly to continue.")
                    .with_placeholder("N")
                    .with_default(false);
                    let confirm_copy = prompt_user(confirm_copy_message);

                    match confirm_copy {
                        None => {
                            println!("Canceling operation. Copy was not successful.");
                            return;
                        }
                        Some(false) => {
                            println!("Canceling operation. Copy was not successful.");
                            return;
                        }
                        _ => (),
                    }

                    // delete original map
                    println!("(4/6) Delete original map");
                    let _ = twilio
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

                    // create new map
                    println!("(5/6) Create new map");
                    let create_map_result = twilio
                        .sync()
                        .service(&sync_service.sid)
                        .maps()
                        .create(CreateMapParams {
                            ttl: None,
                            unique_name: Some(String::from(trimmed_name)),
                        })
                        .await;

                    if let Err(error) = create_map_result {
                        println!("Errored: Failed while creating new map: {:?}", error);
                        break;
                    }

                    let new_map = create_map_result.unwrap();

                    // clone all items into new map
                    println!("(6/6) Clone items into new map");
                    for item in items.iter() {
                        let create_item_result = twilio
                            .sync()
                            .service(&sync_service.sid)
                            .map(&new_map.sid)
                            .mapitems()
                            .create(CreateMapItemParams {
                                key: String::from(&item.key),
                                data: &item.data,
                                collection_ttl: None,
                                ttl: None,
                            })
                            .await;

                        if let Err(error) = create_item_result {
                            println!(
                                "Errored: Failed while copying items to new map: {:?}",
                                error
                            );
                            return;
                        }
                    }

                    println!("Map rename complete");
                    break;
                }
                Action::Delete => {
                    let confirm_prompt =
                        Confirm::new("Are you sure you wish to delete the Sync Map?")
                            .with_placeholder("N")
                            .with_default(false);
                    let confirmation = prompt_user(confirm_prompt);
                    if confirmation.is_some() && confirmation.unwrap() {
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
