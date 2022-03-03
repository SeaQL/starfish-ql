use futures::{stream, StreamExt};
use reqwest::{Client, Error};
use serde_json::Value;
use std::{collections::HashMap, fs, sync::Arc, time::Duration};
use tokio::{sync::Mutex, time::sleep};

const REQUEST_BATCH_SIZE: usize = 150;
const SLEEP_BETWEEN_BATCHES: u64 = 4000; // in ms
const BASE_URL: &str = "https://crates.io/api/v1/crates/";

#[derive(Debug)]
struct CrateMeta {
    name: String,
    categories: Vec<String>,
    keywords: Vec<String>,
}

#[tokio::main]
async fn main() {
    let crate_names: Vec<String> = serde_json::from_str(
        &fs::read_to_string("in/crate_names.json").expect("Unable to read the JSON file."),
    )
    .expect("The JSON file is not well-formatted.");
    let num_crate_names = crate_names.len();

    println!("Number of crate names: {}", num_crate_names);

    let client_builder = Client::builder().user_agent("SeaQL (hello@sea-ql.org)");
    let client = client_builder.build().expect("Unable to build client.");

    let batch = Arc::new(Mutex::new(REQUEST_BATCH_SIZE));
    let progress = Arc::new(Mutex::new(0));

    let bodies: Vec<_> = stream::iter(crate_names)
        .map(|crate_name| {
            let client = client.clone();
            let batch = Arc::clone(&batch);
            let progress = Arc::clone(&progress);

            tokio::spawn(async move {
                {
                    let mut lock = batch.lock().await;
                    if *lock == 0 {
                        sleep(Duration::from_millis(SLEEP_BETWEEN_BATCHES)).await;
                        *lock = REQUEST_BATCH_SIZE;
                    }
                    *lock -= 1;
                }

                let url = BASE_URL.to_owned() + &crate_name;

                let res = client.get(url).send().await?;
                let text = res.text().await?;

                let json: Value =
                    serde_json::from_str(&text).expect("The JSON file is not well-formatted.");

                let categories: Vec<String> = json
                    .get("categories")
                    .unwrap()
                    .as_array()
                    .unwrap()
                    .iter()
                    .map(|obj| obj.get("category").unwrap().as_str().unwrap().to_owned())
                    .collect();
                let keywords: Vec<String> = json
                    .get("crate")
                    .unwrap()
                    .get("keywords")
                    .unwrap()
                    .as_array()
                    .unwrap()
                    .iter()
                    .map(|val| val.as_str().unwrap().to_owned())
                    .collect();

                let crate_meta = CrateMeta {
                    name: crate_name,
                    categories,
                    keywords,
                };

                {
                    let mut lock = progress.lock().await;
                    *lock += 1;
                    if *lock == 1 || (*lock % 500) == 0 || *lock == num_crate_names {
                        println!("{}/{} processed.", *lock, num_crate_names);
                    }
                }

                Result::<CrateMeta, Error>::Ok(crate_meta)
            })
        })
        .buffer_unordered(num_crate_names)
        .collect()
        .await;

    let mut mappings = HashMap::new();
    mappings.insert("categories", HashMap::new());
    mappings.insert("keywords", HashMap::new());

    let mut req_errs = vec![];
    let mut join_errs = vec![];

    let mut num_crate_metas = 0;

    bodies.into_iter().for_each(|body| match body {
        Ok(Ok(crate_meta)) => {
            num_crate_metas += 1;

            crate_meta.categories.into_iter().for_each(|category| {
                if !mappings
                    .get_mut("categories")
                    .unwrap()
                    .contains_key(&category)
                {
                    mappings
                        .get_mut("categories")
                        .unwrap()
                        .insert(category.clone(), vec![]);
                }
                mappings
                    .get_mut("categories")
                    .unwrap()
                    .get_mut(&category)
                    .unwrap()
                    .push(crate_meta.name.clone());
            });

            crate_meta.keywords.into_iter().for_each(|keyword| {
                if !mappings.get_mut("keywords").unwrap().contains_key(&keyword) {
                    mappings
                        .get_mut("keywords")
                        .unwrap()
                        .insert(keyword.clone(), vec![]);
                }
                mappings
                    .get_mut("keywords")
                    .unwrap()
                    .get_mut(&keyword)
                    .unwrap()
                    .push(crate_meta.name.clone());
            });
        }
        Ok(Err(e)) => {
            eprintln!("Got a reqwest::Error: {}", e);
            req_errs.push(e);
        }
        Err(e) => {
            eprintln!("Got a tokio::JoinError: {}", e);
            join_errs.push(e);
        }
    });

    println!();
    eprintln!("reqwest errors:");
    req_errs
        .into_iter()
        .for_each(|e| eprintln!("Got a reqwest::Error: {}", e));
    eprintln!("join errors:");
    join_errs
        .into_iter()
        .for_each(|e| eprintln!("Got a tokio::JoinError: {}", e));

    println!("{}/{} crate metas successfully processed.", num_crate_metas, num_crate_names);

    fs::write(
        "out/mappings.json",
        serde_json::to_string(&mappings).unwrap(),
    )
    .expect("Unable to write mappings to file.");
}
