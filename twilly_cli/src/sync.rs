mod documents;
mod maps;

use std::process;

use inquire::Select;
use strum::IntoEnumIterator;
use strum_macros::{Display, EnumIter, EnumString};
use twilly::Client;
use twilly_cli::{get_action_choice_from_user, prompt_user_selection, ActionChoice};

#[derive(Debug, Clone, Display, EnumIter, EnumString)]
pub enum Action {
    #[strum(to_string = "Documents")]
    Document,
    #[strum(to_string = "Maps")]
    Map,
    #[strum(to_string = "List Details")]
    ListDetails,
    Delete,
    Back,
    Exit,
}

pub fn choose_sync_resource(twilio: &Client) {
    let mut sync_services = twilio
        .sync()
        .services()
        .list()
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
            if let Some(action_choice) = get_action_choice_from_user(
                sync_services
                    .iter()
                    .map(|service| format!("({}) {}", service.sid, service.unique_name))
                    .collect::<Vec<String>>(),
                "Choose a Sync Service: ",
            ) {
                match action_choice {
                    ActionChoice::Back => {
                        break;
                    }
                    ActionChoice::Exit => process::exit(0),
                    ActionChoice::Other(choice) => {
                        let sync_service_position = sync_services
                            .iter()
                            .position(|conv| conv.sid == choice[1..35])
                            .expect("Could not find Sync Service in existing Sync Service list");

                        selected_sync_service_index = Some(sync_service_position);
                        &mut sync_services[sync_service_position]
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
                    documents::choose_document_action(&twilio, selected_sync_service)
                }
                Action::Map => maps::choose_map_action(&twilio, selected_sync_service),
                Action::ListDetails => {
                    println!("{:#?}", selected_sync_service);
                    println!()
                }
                Action::Delete => println!("Delete!"),
                Action::Back => break,
                Action::Exit => process::exit(0),
            }
        }
    }
}
