use rocket::tokio::runtime;
use starfish::api::rocket;

fn main() {
    runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .unwrap()
        .block_on(rocket().launch())
        .unwrap();
}
