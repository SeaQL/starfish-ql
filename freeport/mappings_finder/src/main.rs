use futures::{stream, StreamExt};
use reqwest::{Client, Error};
use serde_json::Value;
use std::{collections::HashMap, fs};

const PARALLEL_FACTOR: usize = 1000;
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

    println!("Number of crate names: {}", crate_names.len());

    let client_builder = Client::builder().user_agent("SeaQL (hello@sea-ql.org)");
    let client = client_builder.build().expect("Unable to build client.");

    let bodies: Vec<_> = stream::iter(crate_names)
        .map(|crate_name| {
            let client = client.clone();
            tokio::spawn(async move {
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

                Result::<CrateMeta, Error>::Ok(crate_meta)
            })
        })
        .buffer_unordered(PARALLEL_FACTOR)
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
            println!("{:?}", crate_meta);

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

    println!("{} crate metas successfully processed.", num_crate_metas);

    fs::write(
        "out/mappings.json",
        serde_json::to_string(&mappings).unwrap(),
    )
    .expect("Unable to write mappings to file.");
}
