use std::env;
use futures::StreamExt;
use telegram_bot::*;
use json;
use std::io::Read;
use serde::Deserialize;
use serde_json;

async fn pong(api : &Api, message : &Message) -> Result<(), Error> {
    api.send(message.text_reply("pong")).await?;
    Ok(())
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
struct Person {
    firsname : String,
    lastname : String,
    birthplace : String,
    birthdate : String,
    location : String,
    signature : String
}

fn read_json_to_person() -> Person {
    let mut value = String::new();
    let mut file = std::fs::File::open("../../data.json").unwrap();
    file.read_to_string(&mut value).unwrap();
    serde_json::from_str(&value).expect("JSON file we incorrectly formatted")
}


#[tokio::main]
async fn main() -> Result<(), Error> {
    let token = env::var("TELEGRAM_BOT_TOKEN").expect("TELEGRAM_BOT_TOKEN not set :(");
    let api = Api::new(token);

    let mut stream = api.stream();
    while let Some(update) = stream.next().await {
        let update = update?;
        if let UpdateKind::Message(message) = update.kind {
            if let MessageKind::Text { ref data, .. } = message.kind {
                println!("<{}>: {}", &message.from.first_name, data);
                api.send(message.text_reply(format!(
                    "Hi, {}! You just wrote '{}'",
                    &message.from.first_name, data
                )))
                .await?;
                if data == "/ping" {
                    pong(&api, &message).await?;
                }
            }
        }
    }
    Ok(())
}
