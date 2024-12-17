#![allow(unused_imports)]
#![allow(unused)]

mod model;
mod payments;

#[cfg(test)]
mod test;

use std::io::Error;

use actix_web::{get, post, web, App, HttpResponse, HttpServer};
use intasend::{Intasend, WalletCreateDetails};
use libsql::Connection;
use model::{
    CheckoutInfo, InitCheckoutBadRequestResponse, InitCheckoutOkResponse, InitCheckoutResponse,
};
use payments::Payments;
use rust_decimal::Decimal;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenvy::dotenv().ok();

    let db = libsql::Builder::new_local("local.db")
        .build()
        .await
        .unwrap();
    let conn = db.connect().unwrap();

    let payments = Payments::init();

    HttpServer::new(move || {
        // Configure CORS
        let cors = actix_cors::Cors::default()
            .allow_any_origin() // Allow requests from any origin
            .allow_any_method() // Allow any HTTP method
            .allow_any_header(); // Allow any header

        // let cors = actix_cors::Cors::default()
        //       .allowed_origin("https://www.rust-lang.org")
        //       .allowed_origin_fn(|origin, _req_head| {
        //           origin.as_bytes().ends_with(b".rust-lang.org")
        //       })
        //       .allowed_methods(vec!["GET", "POST"])
        //       .allowed_headers(vec![http::header::AUTHORIZATION, http::header::ACCEPT])
        //       .allowed_header(http::header::CONTENT_TYPE)
        //       .max_age(3600);

        App::new()
            .wrap(cors)
            .app_data(web::Data::new(payments.clone()))
            .app_data(web::Data::new(conn.clone()))
            .service(init_checkout)
            .service(create_wallet)
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}

// Checkout Init
#[post("/init-checkout")]
async fn init_checkout(
    db: web::Data<Connection>,
    payments: web::Data<Intasend>,
    form: web::Json<CheckoutInfo>,
) -> impl actix_web::Responder {
    println!("form data: {:#?}", form);

    /// Intasend CheckoutsAPI
    let checkouts_api = payments.checkout();

    // Try to convert the amount from a string to a float
    let amount_str = form.amount.clone(); // Assuming amount is a String
    let amount_float: f64 = match amount_str.parse() {
        Ok(value) => value,
        Err(_) => {
            // Return a bad request response
            return InitCheckoutResponse::BadRequest(InitCheckoutBadRequestResponse {
                message: "Invalid checkout amount format",
            });
        } // Handle parse error
    };

    // Convert amount to pennies (integer) and create a Decimal
    let amount_in_pennies = (amount_float * 100.0).round() as i64; // Convert to pennies
    let amount_decimal = Decimal::new(amount_in_pennies, 2); // Create Decimal

    let checkout_req = intasend::CheckoutRequest {
        first_name: Some(form.first_name.clone()),
        last_name: Some(form.last_name.clone()),
        email: Some(form.email.clone()),
        method: Some(intasend::CheckoutMethod::CardPayment),
        amount: amount_decimal,
        currency: intasend::Currency::Kes,
    };
    println!(
        "[#] Checkout request (JSON): {:#?}",
        serde_json::to_value(&checkout_req)
    );

    let checkout_response = checkouts_api
        .initiate(checkout_req)
        .await
        .expect("Error occurred while decoding checkout response data");

    println!("[#] Checkout response: {:#?}", checkout_response);

    // let rows = db.query("SELECT * FROM items", ()).await.unwrap();

    let response_data = InitCheckoutOkResponse {
        message: "Checkout initiated successfully",
        data: checkout_response,
    };

    // HttpResponse::Ok().body(response_data)
    // response_data
    //
    // Return success response
    InitCheckoutResponse::Success(response_data)
}

// Wallets Creation
#[post("/wallets")]
async fn create_wallet(
    db: web::Data<Connection>,
    payments: web::Data<Intasend>,
    form: web::Json<WalletCreateDetails>,
) -> impl actix_web::Responder {
    let wallets_api = payments.wallets();

    match wallets_api.create(form.into_inner()).await {
        Ok(wallet) => {
            println!("{wallet:#?}");

            let response = serde_json::json!({
                "message": "Wallet created!",
                "data": wallet,
            })
            .to_string();
            HttpResponse::Ok().body(response)
        }
        Err(err) => {
            println!("{err:#?}");

            // Extract the details from the error dynamically
            // let details = extract_error_details_as_json(&err);
            let details = match parse_error_as_json(&err) {
                Some(json_details) => {
                    println!("{json_details:#?}");
                    // let details = json_details[""];
                    json_details
                }
                None => serde_json::json!({ "message": format!("{:?}", err) }),
            };
            println!("{details:#?}");

            // Build the JSON response with the error details
            let response = serde_json::json!({
                "error": "Wallet creation failed",
                "data": details,
            });

            // HttpResponse::InternalServerError().body("wallet creation failed!")
            HttpResponse::BadRequest().json(response)
        }
    }
}

fn parse_error_as_json(err: &anyhow::Error) -> Option<serde_json::Value> {
    // Try to convert the error into a JSON-compatible format
    let error = err.downcast_ref::<serde_json::Error>();
    error.map(|op| {
        serde_json::json!({
            "message": op.to_string(),
        })
    })
}

// Adds a new user to the "users" collection in the database.
// #[post("/add_user")]
// async fn add_user(client: web::Data<Client>, form: web::Json<User>) -> HttpResponse {
//     println!("data: {:#?}", form);
//     let collection = client.database(DB_NAME).collection(COLL_NAME);
//     let result = collection.insert_one(form.into_inner()).await;
//     match result {
//         Ok(_) => HttpResponse::Ok().body("user added"),
//         Err(err) => HttpResponse::InternalServerError().body(err.to_string()),
//     }
// }

// Gets the user with the supplied username.
// #[get("/get_user/{username}")]
// async fn get_user(client: web::Data<Client>, username: web::Path<String>) -> HttpResponse {
//     let username = username.into_inner();
//     let collection: Collection<User> = client.database(DB_NAME).collection(COLL_NAME);
//     match collection.find_one(doc! { "username": &username }).await {
//         Ok(Some(user)) => HttpResponse::Ok().json(user),
//         Ok(None) => {
//             HttpResponse::NotFound().body(format!("No user found with username {username}"))
//         }
//         Err(err) => HttpResponse::InternalServerError().body(err.to_string()),
//     }
// }
