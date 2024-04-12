use std::process;

use inquire::{Confirm, Select, Text};
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
                        .await
                }

                Action::ListDetails => {
                    println!("{:#?}", selected_sync_map);
                    println!()
                }
                Action::Rename => {
                    let get_renamed_map_name = Text::new(
                        "What would you like to rename this map to? The new map name must match `^[a-zA-Z-_0-9]+$`"
                    );
                    let get_rename_prompt_result = prompt_user(get_renamed_map_name);
                    let mut new_name = String::from("");
                    if get_rename_prompt_result.is_none() {
                        println!("No name entered");
                        break;
                    }

                    let unwrapped = get_rename_prompt_result.unwrap();
                    new_name = unwrapped
                        .chars()
                        .filter(|c| {
                            (c.is_alphabetic() || c.is_numeric() || c.eq(&'-') || c.eq(&'_'))
                        })
                        .collect();

                    if new_name.is_empty() {
                        println!("Name was not valid");
                        break;
                    }

                    let message =
                        format!("We will rename the map to {}.\nThis process will create a temporary map copy which you can delete after.\nDo you wish to continue?", new_name);

                    let info_prompt = Confirm::new(&message.as_str())
                        .with_placeholder("No")
                        .with_default(false);
                    let confirmation = prompt_user(info_prompt);
                    if confirmation.is_none() || confirmation.unwrap() == false {
                        break;
                    }

                    println!("(0/6) -> Starting map rename");
                    println!("(1/6) -> Creating temporary duplicate map");
                    let temp_map = twilio
                        .sync()
                        .service(&sync_service.sid)
                        .maps()
                        .create(CreateMapParams {
                            unique_name: Some(String::from("test-temp")),
                            ttl: None,
                        })
                        .await
                        .unwrap_or_else(|error| panic!("{}", error));
                    println!("(2/6) -> Cloning items to temporary map");
                    let items = twilio
                        .sync()
                        .service(&sync_service.sid)
                        .map(&selected_sync_map.sid)
                        .mapitems()
                        .list(ListParams {
                            from: None,
                            bounds: None,
                            order: None,
                        })
                        .await
                        .unwrap_or_else(|error| panic!("{}", error));
                    let mut items_iter = items.into_iter();
                    while let Some(item) = items_iter.next() {
                        twilio
                            .sync()
                            .service(&sync_service.sid)
                            .map(&temp_map.sid)
                            .mapitems()
                            .create(CreateMapItemParams {
                                key: item.key,
                                ttl: None,
                                collection_ttl: None,
                                data: &item.data,
                            })
                            .await
                            .unwrap_or_else(|error| panic!("{}", error));
                    }
                    println!("(3/6) -> Deleting original map");
                    println!("(4/6) -> Creating renamed map");
                    println!("(5/6) -> Cloning items to renamed map");
                    println!("(6/6) -> Done");
                }
                Action::Delete => {
                    let confirm_prompt =
                        Confirm::new("Are you sure to wish to delete the Sync Map? (Yes / No)");
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
                Action::Back => break,
                Action::Exit => process::exit(0),
            }
        }
    }
}
