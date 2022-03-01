use std::{fs, thread};

const NUM_THREADS: usize = 10;

fn main() {
    let crate_names: Vec<String> = serde_json::from_str(
        &fs::read_to_string("crate_names.json").expect("Unable to read the JSON file."),
    )
    .expect("The JSON file is not well-formatted.");

    println!("Number of crate names: {}", crate_names.len());

    let mut children = Vec::with_capacity(NUM_THREADS);

    let mut chunked_crate_names = vec![vec![]; NUM_THREADS];
    crate_names.into_iter()
        .enumerate()
        .for_each(|(i, crate_name)| {
            chunked_crate_names[i % NUM_THREADS].push(crate_name)
        });

    for (i, chunk) in chunked_crate_names.into_iter().enumerate() {
        children.push(thread::spawn(move || -> usize {
            println!("Child {} started.", i);
            println!("Child {} finished.", i);
            chunk.len()
        }));
    }

    let final_result = children.into_iter().map(|c| c.join().unwrap()).sum::<usize>();

    println!("Final result: {}", final_result);
}
