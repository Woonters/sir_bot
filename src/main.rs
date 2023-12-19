use reqwest::get;
use std::{
    fs::File,
    io::{stdin, Write},
};
use tokio;
use urlencoding::encode;

#[tokio::test]
async fn test_voice() {
    let body = get("https://api.novelai.net/ai/generate-voice?text=HELLO%20SIR%21%21%21&seed=aHaleAndHeartySir&voice=-1&opus=false&version=v2");
    match body.await {
        Ok(resp) => {
            if resp.status().is_success() {
                let bytes = resp.bytes().await.unwrap();
                println!("Bytes: {:?}", bytes);
                let mut file = File::create("temp.mpeg").expect("I HAVE FAILED TO CREATE THE FILE");
                file.write_all(&bytes).unwrap();
                println!("HELLO SIR I HAVE GOT A MESSAGE FOR YOU");
            } else {
                println!("Oh no you have failed the challenge: {}", resp.status())
            }
        }
        Err(e) => {
            println!("Request failed: {}", e)
        }
    }
}

#[tokio::main]
async fn get_voice_and_save(input: &str) {
    let response = get(format!("https://api.novelai.net/ai/generate-voice?text={}&seed=aHaleAndHeartySir&voice=-1&opus=false&version=v2",input));
    match response.await {
        Ok(resp) => {
            if resp.status().is_success() {
                let bytes = resp
                    .bytes()
                    .await
                    .expect("SOMETHING VERY WRONG HAS HAPPENED SIR");
                let mut file = File::create("audio/temp.mpeg").expect("File creation failed");
                file.write_all(&bytes).unwrap();
                println!("Message received, file saved, all success")
            } else {
                println!("Bad Response: {}", resp.status());
            }
        }
        Err(e) => println!("Request failed {}", e),
    }
}

fn get_input() -> String {
    println!("WHAT WOULD YOU LIKE ME TO SAY SIR?! ");
    let mut input = String::new();
    stdin()
        .read_line(&mut input)
        .expect("I DID NOT UNDERSTAND THAT SIR");
    return input;
}

fn fix_input(input: &str) -> String {
    let binding = input.to_uppercase();
    let encoded = encode(&binding);
    return encoded.into_owned();
}

fn main() {
    let to_say = fix_input(&get_input());
    get_voice_and_save(&to_say);
}
