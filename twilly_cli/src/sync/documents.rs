use std::process;

use inquire::{validator::Validation, Confirm, Select, Text};
use strum::IntoEnumIterator;
use strum_macros::{Display, EnumIter, EnumString};
use twilly::{sync::services::SyncService, Client, ErrorKind};
use twilly_cli::{get_action_choice_from_user, prompt_user, prompt_user_selection, ActionChoice};

#[derive(Debug, Clone, Display, EnumIter, EnumString)]
pub enum Action {
    #[strum(to_string = "Get Document")]
    GetDocument,
    #[strum(to_string = "List Documents")]
    ListDocuments,
    Back,
    Exit,
}

pub fn choose_document_action(twilio: &Client, sync_service: &SyncService) {
    let options: Vec<Action> = Action::iter().collect();

    loop {
        let action_selection_prompt = Select::new("Select an action:", options.clone());

        if let Some(action) = prompt_user_selection(action_selection_prompt) {
            match action {
                Action::GetDocument => {
                    let document_sid_prompt =
                        Text::new("Please provide a document SID (or unique name):")
                            .with_placeholder("ET...")
                            .with_validator(|val: &str| match val.starts_with("ET") {
                                true => Ok(Validation::Valid),
                                false => Ok(Validation::Invalid(
                                    "Document SID must start with ET".into(),
                                )),
                            })
                            .with_validator(|val: &str| match val.len() {
                                34 => Ok(Validation::Valid),
                                _ => Ok(Validation::Invalid(
                                    "Your SID should be 34 characters in length".into(),
                                )),
                            });

                    if let Some(document_sid) = prompt_user(document_sid_prompt) {
                        match twilio
                            .sync()
                            .service(&sync_service.sid)
                            .document(&document_sid)
                            .get()
                        {
                            Ok(document) => loop {
                                if let Some(action_choice) = get_action_choice_from_user(
                                    vec![String::from("List Details"), String::from("Delete")],
                                    "Select an action: ",
                                ) {
                                    match action_choice {
                                        ActionChoice::Back => break,
                                        ActionChoice::Exit => process::exit(0),
                                        ActionChoice::Other(choice) => match choice.as_str() {
                                            "List Details" => {
                                                println!("{:#?}", document);
                                                println!();
                                            }
                                            "Delete" => {
                                                let confirm_prompt = Confirm::new(
                                                "Are you sure to wish to delete the Document? (Yes / No)",
                                            );
                                                let confirmation = prompt_user(confirm_prompt);
                                                if confirmation.is_some()
                                                    && confirmation.unwrap() == true
                                                {
                                                    println!("Deleting Document...");
                                                    twilio
                                                        .conversations()
                                                        .delete(&document_sid)
                                                        .unwrap_or_else(|error| {
                                                            panic!("{}", error)
                                                        });
                                                    println!("Document deleted.");
                                                    println!();
                                                    break;
                                                }
                                            }
                                            _ => println!("Unknown action '{}'", choice),
                                        },
                                    }
                                }
                            },
                            Err(error) => match error.kind {
                                ErrorKind::TwilioError(twilio_error) => {
                                    if twilio_error.status == 404 {
                                        println!(
                                            "A Document with SID '{}' was not found.",
                                            &document_sid
                                        );
                                        println!("");
                                    } else {
                                        panic!("{}", twilio_error)
                                    }
                                }
                                _ => panic!("{}", error),
                            },
                        }
                    }
                }
                Action::ListDocuments => {
                    println!("Fetching Documents...");
                    let mut documents = twilio
                        .sync()
                        .service(&sync_service.sid)
                        .documents()
                        .list()
                        .unwrap_or_else(|error| panic!("{}", error));

                    let number_of_documents = documents.len();

                    if number_of_documents == 0 {
                        println!("No Documents found.");
                        println!();
                    } else {
                        println!("Found {} Documents.", number_of_documents);

                        let mut selected_document_index: Option<usize> = None;
                        loop {
                            let selected_document = if let Some(index) = selected_document_index {
                                &mut documents[index]
                            } else {
                                if let Some(action_choice) = get_action_choice_from_user(
                                    documents
                                        .iter()
                                        .map(|doc| format!("({}) {}", doc.sid, doc.unique_name))
                                        .collect::<Vec<String>>(),
                                    "Documents: ",
                                ) {
                                    match action_choice {
                                        ActionChoice::Back => {
                                            break;
                                        }
                                        ActionChoice::Exit => process::exit(0),
                                        ActionChoice::Other(choice) => {
                                            let document_position = documents
											.iter()
											.position(|doc| doc.sid == choice[..34])
											.expect("Could not find documnet in existing documents list");

                                            selected_document_index = Some(document_position);
                                            &mut documents[document_position]
                                        }
                                    }
                                } else {
                                    break;
                                }
                            };

                            loop {
                                if let Some(action_choice) = get_action_choice_from_user(
                                    vec![String::from("List Details"), String::from("Delete")],
                                    "Select an action: ",
                                ) {
                                    match action_choice {
                                        ActionChoice::Back => break,
                                        ActionChoice::Exit => process::exit(0),
                                        ActionChoice::Other(choice) => match choice.as_str() {
                                            "List Details" => {
                                                println!("{:#?}", selected_document);
                                                println!();
                                            }
                                            "Delete" => {
                                                let confirm_prompt = Confirm::new(
                                                "Are you sure to wish to delete the Document? (Yes / No)",
                                            );
                                                let confirmation = prompt_user(confirm_prompt);
                                                if confirmation.is_some()
                                                    && confirmation.unwrap() == true
                                                {
                                                    println!("Deleting Document...");
                                                    twilio
                                                        .conversations()
                                                        .delete(&selected_document.sid)
                                                        .unwrap_or_else(|error| {
                                                            panic!("{}", error)
                                                        });
                                                    documents.remove(selected_document_index.expect("Could not fin document in existing documents list"));
                                                    selected_document_index = None;
                                                    println!("Document deleted.");
                                                    println!();
                                                    break;
                                                }
                                            }
                                            _ => println!("Unknown action '{}'", choice),
                                        },
                                    }
                                }
                            }
                        }
                    }
                }
                Action::Back => break,
                Action::Exit => process::exit(0),
            }
        } else {
            break;
        }
    }
}
