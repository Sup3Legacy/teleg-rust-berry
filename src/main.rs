use std::env;
use futures::StreamExt;
use telegram_bot::*;
use std::io::Read;
use serde::Deserialize;
use serde_json;
use std::fs::{self};
use std::path::Path;
use std::collections::HashMap;
//use pdf_form::{Form, FieldType};
use lopdf::Document;

async fn pong(api : &Api, message : &Message) -> Result<(), Error> {
    api.send(message.text_reply("pong")).await?;
    Ok(())
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
struct Person {
    firstname : String,
    lastname : String,
    birthplace : String,
    birthdate : String,
    location : String,
    signature : String
}

fn read_json_to_person(path : &Path) -> Person {
    let mut value = String::new();
    let mut file = std::fs::File::open(path.to_str().unwrap()).unwrap();
    file.read_to_string(&mut value).unwrap();
    serde_json::from_str(&value).expect("JSON file incorrectly formatted")
}

fn get_persons<'a>() -> Result<HashMap<String, Person>, Error> {
    let path = Path::new("persons");
    let mut map = HashMap::new();
    for entry in fs::read_dir(&path).unwrap() {
        let file_path = entry.unwrap().path();
        let person : Person = read_json_to_person(&file_path);
        let key = String::from(file_path.file_stem().unwrap().to_str().unwrap());
        //println!("Inserted ({}, {:#?}", key, person);
        map.insert(key, person);
    }
    Ok(map)
}


#[tokio::main]
async fn main() -> Result<(), Error> {
    let token = env::var("TELEGRAM_BOT_TOKEN").expect("TELEGRAM_BOT_TOKEN not set :(");
    let api = Api::new(token);
    let mut stream = api.stream();

    let personnes_hash : HashMap<String, Person> = match get_persons() {
        Ok(p) => p,
        _ => {
            println!("Could not initialize data.");
            HashMap::new()
        }
    };

    /*
    let mut form = Form::load("modele.pdf").unwrap();
    let field_types = form.get_all_types();
    for ty in field_types {
        println!("{:?}", ty);
    };    
    */
    

    let mut doc = Document::load("modele2.pdf").unwrap();
    doc.replace_text(1, "ATTESTATION DE DÉPLACEMENT DÉROGATOIRE
    DURANT LES HORAIRES DU COUVRE-FEU", "mdr").unwrap();
    doc.save("modified.pdf").unwrap();

    while let Some(update) = stream.next().await {
        let update = update?;
        if let UpdateKind::Message(message) = update.kind {
            if let MessageKind::Text { ref data, .. } = message.kind {
                println!("<{}>: {}", &message.from.first_name, data);
                let mut message_iter = data.split_whitespace();
                let first = (&mut message_iter).next().unwrap();
                if first == "/attest" {
                    match message_iter.next() {
                        Some(key) => {
                            match &personnes_hash.get(key) {
                                Some(p) => {
                                    api.send(message.text_reply(format!("{:#?}", p))).await?;
                                },
                                None => {
                                    api.send(message.text_reply(format!("Key not found : {}", key))).await?;
                                }
                            }
                        },
                        None => {
                            api.send(message.text_reply(format!("/attest needs exactly one argument"))).await?;
                        }
                    }
                }
                if first == "/ping" {
                    pong(&api, &message).await.unwrap();
                }
            }
        }
    }
    Ok(())
}
