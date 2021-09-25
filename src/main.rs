use actix_web::{web, App, HttpRequest, HttpServer, Responder};

use crossbeam_channel::{unbounded, Sender};

use chrono::{SecondsFormat, Utc};
use std::thread;
use std::time::Duration;

async fn greet(req: HttpRequest, txs: web::Data<Sender<String>>) -> impl Responder {
    // get name from url
    let name = req.match_info().get("name").unwrap_or("World");

    // send to channel
    let now = Utc::now().to_rfc3339_opts(SecondsFormat::Secs, true);
    let msg = format!("{}: {}", now, name);
    txs.send(msg).unwrap();

    // send result back to user
    format!("Hello {}!", &name)
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Channels have two endpoints: the `Sender<T>` and the `Receiver<T>`,
    // where `T` is the type of the message to be transferred
    // (type annotation is superfluous)
    let (tx, rx) = unbounded();
    let worker = thread::spawn(move || loop {
        let val: String = rx.recv().unwrap();
        thread::sleep(Duration::from_secs(1));
        println!("received: {}", val)
    });

    match HttpServer::new(move || {
        App::new()
            .data(tx.clone())
            .route("/hello", web::get().to(greet))
            .route("/hello/{name}", web::get().to(greet))
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
    {
        Ok(_) => {
            println!("finished");
            worker.join().expect("oops! the child thread panicked");
            Ok(())
        }
        Err(e) => {
            eprintln!("{:?}", e);
            Err(e)
        }
    }
}
