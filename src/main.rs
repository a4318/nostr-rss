use rss::{Channel, Item};
use serde::{Serialize, Deserialize};
use std::error::Error;
use std::io::{Read, Write};
use std::fs::File;
use std::io::BufReader;
use chrono::{DateTime, Duration, FixedOffset};
use std::collections::HashMap;
use std::path::Path;

use nostr_sdk::prelude::*;

fn fetch_feed_items(url: &str) -> Result<Vec<Item>, Box<dyn Error>> {
    let resp = ureq::get(url)
        .call()
        .unwrap();
    if resp.has("Content-Length") {
        resp.header("Content-Length")
        .and_then(|s| s.parse::<usize>().ok()).unwrap();
    }

    let mut reader = resp.into_reader();
    let mut bytes = vec![];
    let size = reader.read_to_end(&mut bytes);
    let channel = Channel::read_from(&bytes[..]).unwrap();

    Ok(channel.into_items())

}

#[derive(Serialize, Deserialize)]
struct Settings {
    name: String,
    url: String,
}

fn get_settings() -> Vec<Settings> {
    let filename = "settings.json";
    let file = File::open(filename).unwrap_or_else(|_| panic!("{} not found", filename));
    let reader = BufReader::new(file);
    return serde_json::from_reader(reader).unwrap_or_else(|_| panic!("{} not read", filename));
}

#[derive(Serialize, Deserialize)]
struct NostrSettings {
    seckey: String,
    channel_id: String,
    relay: String,
}

fn get_nostr_settings() -> NostrSettings {
    let filename = "nostr_settings.json";
    let file = File::open(filename).unwrap_or_else(|_| panic!("{} not found", filename));
    let reader = BufReader::new(file);
    return serde_json::from_reader(reader).unwrap_or_else(|_| panic!("{} not read", filename));
}

#[derive(Serialize, Deserialize, Debug)]
struct FeedItem {
    title: String,
    link: String,
    pub_date: String,
    #[serde(skip)]
    enclosure_url: String,
}

fn read_latest_json() -> HashMap<String, FeedItem> {
    let filename = "latest.json";
    if Path::new(filename).is_file() {
        let file = File::open(filename).unwrap_or_else(|_| panic!("{} not found", filename));
        let reader = BufReader::new(file);
        return serde_json::from_reader(reader).unwrap_or_else(|_| panic!("{} not read", filename));
    }
    else {
        return HashMap::new();
    }
}

fn get_leatest_items(settings: Vec<Settings>) -> HashMap<String, FeedItem> {
    let mut latest_items = HashMap::new();
    for setting in settings {
        let items = fetch_feed_items(&setting.url).unwrap();

        latest_items.insert(setting.name, FeedItem {
            title: items[0].title().unwrap().to_string(),
            link: items[0].link().unwrap().to_string(),
            pub_date: items[0].pub_date().unwrap().to_string(),
            enclosure_url: items[0].enclosure().unwrap().url().to_string(),
        });
    }
    return latest_items;
}

async fn connect_nostr(nostr_setting: &NostrSettings) -> nostr_sdk::Client {
    let my_keys = Keys::parse(&nostr_setting.seckey).unwrap();
    let bech32_pubkey: String = my_keys.public_key().to_bech32().unwrap();
    println!("Bech32 PubKey: {}", bech32_pubkey);
    // Create new client
    let client = Client::new(&my_keys);

    // Add relays
    client.add_relay("wss://relay.damus.io").await.unwrap();
    client.add_relay("wss://relay-jp.nostr.wirednet.jp").await.unwrap();

    // Connect to relays
    client.connect().await;

    return client;
}

async fn post_nostr_channel(nostr_setting: NostrSettings, msgs: Vec<String>) -> Result<()> {
    let client = connect_nostr(&nostr_setting).await;
    /*let metadata = Metadata::new()
        .name("comic magazine bot")
        .display_name("comic magazine bot")
        .about("comic magazine bot")
        .picture(Url::parse("https://example.com/avatar.png")?)
        .banner(Url::parse("https://example.com/banner.png")?);*/

    // Update metadata
    //client.set_metadata(&metadata).await?;

    // Publish a text note
    //client.publish_text_note("My first text note from Nostr SDK!", []).await?;

    /*let channel_metadata = Metadata::new()
        .name("comic magazine publish day");
    client.new_channel(&channel_metadata).await?;*/
    for msg in msgs {
        client.send_channel_msg(EventId::parse(&nostr_setting.channel_id).unwrap(), Url::parse(&nostr_setting.relay).unwrap(), msg).await?;
    }

    Ok(())
}

fn compare_pub_date(new: &HashMap<String, FeedItem>, old :&HashMap<String, FeedItem>) -> Vec<String> {
    let mut res = Vec::new();
    for (k, v) in new.iter() {
        if old.contains_key(k) {
            let new_date = DateTime::parse_from_rfc2822(&v.pub_date).unwrap();
            let old_date = DateTime::parse_from_rfc2822(&old.get(k).unwrap().pub_date).unwrap();
            if (new_date - old_date).num_milliseconds() > 0 {
                println!("{}: {} : {}\n {}", v.title, v.link, v.pub_date, v.enclosure_url);
                res.push(format!("{}: {}\n {}\n {}", v.title, v.pub_date, v.link, v.enclosure_url));
            }
        } else {
            println!("{}: {} : {}\n {}", v.title, v.link, v.pub_date, v.enclosure_url);
            res.push(format!("{}: {}\n {}\n {}", v.title, v.pub_date, v.link, v.enclosure_url));
        }
    }
    return res;
}



#[tokio::main]
async fn main() {
    let settings = get_settings();

    let old_items = read_latest_json();
    let latest_items = get_leatest_items(settings);

    let nostr_setting = get_nostr_settings();

    let msgs = compare_pub_date(&latest_items, &old_items);

    if !msgs.is_empty() {
        let json = serde_json::to_string(&latest_items).unwrap();
        //println!("{}", &json);

        std::fs::write("latest.json", json).unwrap();

        post_nostr_channel(nostr_setting, msgs).await;
    }



}
