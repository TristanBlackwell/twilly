use std::process;

use inquire::Select;
use strum::IntoEnumIterator;
use strum_macros::{Display, EnumIter, EnumString};
use twilly::{sync::services::SyncService, Client};
use twilly_cli::{get_action_choice_from_user, prompt_user_selection, ActionChoice};

#[derive(Debug, Clone, Display, EnumIter, EnumString)]
pub enum SyncSubResource {
    #[strum(to_string = "Map Items")]
    MapItem,
    Back,
    Exit,
}

pub fn choose_map_action(twilio: &Client, sync_service: &SyncService) {
    let mut sync_maps = twilio
        .sync()
        .service(&sync_service.sid)
        .maps()
        .list()
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
                            .position(|conv| conv.sid == choice[1..35])
                            .expect("Could not find Sync Map in existing Sync Map list");

                        selected_sync_map_index = Some(sync_map_position);
                        &mut sync_maps[sync_map_position]
                    }
                }
            } else {
                break;
            }
        };

        let options: Vec<SyncSubResource> = SyncSubResource::iter().collect();
        let resource_selection_prompt = Select::new("Select an resource:", options.clone());
        if let Some(resource) = prompt_user_selection(resource_selection_prompt) {
            match resource {
                SyncSubResource::MapItem => {
                    println!("Map Items!");
                }
                SyncSubResource::Back => break,
                SyncSubResource::Exit => process::exit(0),
            }
        }
    }
}
