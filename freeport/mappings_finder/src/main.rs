use std::{fs, thread};
use tokio::runtime::Runtime;

const NUM_THREADS: usize = 10;
const BASE_URL: &str = "https://crates.io/api/v1/crates/";

struct CrateMeta {
    name: String,
    categories: Vec<String>,
    keywords: Vec<String>,
}

fn main() {
    let crate_names: Vec<String> = serde_json::from_str(
        &fs::read_to_string("crate_names.json").expect("Unable to read the JSON file."),
    )
    .expect("The JSON file is not well-formatted.");

    println!("Number of crate names: {}", crate_names.len());

    // let mut children = Vec::with_capacity(NUM_THREADS);

    let mut chunked_crate_names = vec![vec![]; NUM_THREADS];
    crate_names.into_iter()
        .enumerate()
        .for_each(|(i, crate_name)| {
            chunked_crate_names[i % NUM_THREADS].push(crate_name)
        });

    let rt = Runtime::new().expect("Unable to create Tokio runtime.");

    let client_builder = reqwest::Client::builder().user_agent("SeaQL (hello@sea-ql.org)");
    let client = client_builder.build().expect("Unable to build client.");

    let res = rt.block_on(client.get(BASE_URL.to_owned() + "serde").send()).expect("Cannot send GET request");

    let text = rt.block_on(res.text()).expect("Unable to parse body of response.");

    let json: serde_json::Value = serde_json::from_str(&text).expect("The JSON file is not well-formatted.");

    let categories: Vec<&str> = json.get("categories").unwrap().as_array().unwrap().iter().map(|obj| obj.get("category").unwrap().as_str().unwrap()).collect();
    let keywords: Vec<&str> = json.get("crate").unwrap().get("keywords").unwrap().as_array().unwrap().iter().map(|val| val.as_str().unwrap()).collect();

    println!("{:?}", categories);
    println!("{:?}", keywords);

    // for (i, chunk) in chunked_crate_names.into_iter().enumerate() {
    //     children.push(thread::spawn(move || -> Result<Vec<CrateMeta>, reqwest::Error> {
    //         println!("Child {} started.", i);

    //         let mut j = 0;
    //         for crate_name in chunk {
    //             println!("{}", 1);

    //             let res = block_on(reqwest::get(BASE_URL.to_owned() + &crate_name))?;
    //             println!("{}", 2);

    //             let text = block_on(res.text())?;

    //             println!("HIHI: {}", text);

    //             j += 1;
    //             if j > 10 {
    //                 break;
    //             }
    //         }

    //         println!("Child {} finished.", i);
    //         Ok(vec![])
    //     }));
    // }
}
