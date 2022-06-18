use std::env;
use std::fs::{create_dir_all, File, OpenOptions};
use std::io::{Read, Write};
use chrono::{DateTime, TimeZone, Utc};
use discord_message::{DiscordMessage, Embed, EmbedField};
use serde::{Deserialize, Deserializer};
use url::Url;

const API_URL: &str = "https://mohistmc.com/api/1.12.2/latest";

#[derive(Deserialize)]
struct PartialMohistReleaseInfo {
    number: u32,
    url: Url,
    #[serde(deserialize_with = "parse_release_date")]
    date: DateTime<Utc>,
}

fn parse_release_date<'de, D>(deserializer: D) -> Result<DateTime<Utc>, D::Error> where D: Deserializer<'de> {
    let raw_date: &str = Deserialize::deserialize(deserializer)?;
    return Ok(Utc.datetime_from_str(raw_date, "%-m/%-d/%Y %-I:%-M:%-S %p").unwrap());
}

fn main() {
    let parsed_info: PartialMohistReleaseInfo =
        reqwest::blocking::get(API_URL)
            .unwrap()
            .json()
            .expect("Failed to deserialize API response.");

    let mut file_name = dirs::data_dir().unwrap();
    file_name.push("siro256");
    file_name.push("notice_mohist_version");
    file_name.push("previous_version.txt");

    if !file_name.exists() {
        let mut directory = file_name.clone();
        directory.pop();

        create_dir_all(directory).expect("Failed to create cache file.");

        let mut file =
            File::create(file_name.clone())
                .expect("Failed to create cache file.");

        file.write_all(
            (parsed_info.number - 1).to_string().as_ref()
        ).expect("Failed to write cache data to file.");
    }

    let mut previous_data = String::new();
    File::open(file_name.clone())
        .expect("Failed to open cache file.")
        .read_to_string(&mut previous_data)
        .expect("Failed to read cache data.");

    if previous_data == parsed_info.number.to_string() { return; }

    let json = DiscordMessage {
        username: Some("Update notifier".to_string()),
        avatar_url: Some(
            Url::parse("https://avatars.githubusercontent.com/u/54493246")
                .expect("Failed to parse avatar URL.")
        ),
        content: "".to_string(),
        embeds: vec![
            Embed {
                title: "Update detected".to_string(),
                description: "Mohist update detected. Please consider to update your server.".to_string(),
                color: Some(0x7fff7f),
                fields: Some(
                    vec![
                        EmbedField {
                            title: "Detected version".to_string(),
                            value: parsed_info.number.to_string(),
                            inline: false
                        },
                        EmbedField {
                            title: "Updated at".to_string(),
                            value: parsed_info.date.format("%Y/%m/%d %H:%M:%S%z").to_string(),
                            inline: false
                        },
                        EmbedField {
                            title: "Download from".to_string(),
                            value: parsed_info.url.to_string(),
                            inline: false
                        }
                    ]
                ),
                ..Default::default()
            }
        ],
    }.to_json().expect("Failed to generate JSON.");

    reqwest::blocking::Client::new()
        .post(
            env::var("WEBHOOK_URL")
                .expect("Failed to get Webhook URL.")
        ).header("Content-Type", "application/json")
        .body(json.clone())
        .send()
        .expect("Failed to POST data to Webhook.");

    OpenOptions::new()
        .write(true)
        .truncate(true)
        .open(file_name)
        .expect("Failed to open cache file.")
        .write(parsed_info.number.to_string().as_ref())
        .expect("Failed to write cache data to file.");
}
