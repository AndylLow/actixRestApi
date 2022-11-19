use actix_web::{get, post, put, delete, web, App, HttpRequest, HttpResponse, HttpServer, Responder, ResponseError};
use actix_web::http::header::ContentType;
use actix_web::http::StatusCode;
use actix_web::body::BoxBody;

use serde::{Serialize, Deserialize};

use std::fmt::Display;
use std::sync::Mutex;

#[derive(Serialize, Deserialize)]
struct obje{
   id: u32,
   author: String,
}

// Implement Responder Trait for obje
impl Responder for obje {
   type Body = BoxBody;

   fn respond_to(self, _req: &HttpRequest) -> HttpResponse<Self::Body> {
       let res_body = serde_json::to_string(&self).unwrap();

       // Create HttpResponse and set Content Type
       HttpResponse::Ok()
           .content_type(ContentType::json())
           .body(res_body)
   }
}

#[derive(Debug, Serialize)]
struct ErrNoId {
   id: u32,
   err: String,
}

// Implement ResponseError for ErrNoId
impl ResponseError for ErrNoId {
   fn status_code(&self) -> StatusCode {
       StatusCode::NOT_FOUND
   }

   fn error_response(&self) -> HttpResponse<BoxBody> {
       let body = serde_json::to_string(&self).unwrap();
       let res = HttpResponse::new(self.status_code());
       res.set_body(BoxBody::new(body))
   }
}

// Implement Display for ErrNoId
impl Display for ErrNoId {
   fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
       write!(f, "{:?}", self)
   }
}

struct AppState {
   objeler: Mutex<Vec<obje>>,
}

// Create a obje
#[post("/objeler")]
async fn post_obje(req: web::Json<obje>, data: web::Data<AppState>) -> impl Responder {
   let new_obje = obje {
       id: req.id,
       author: String::from(&req.author),
   };

   let mut objeler = data.objeler.lock().unwrap();

   let response = serde_json::to_string(&new_obje).unwrap();

   objeler.push(new_obje);
   HttpResponse::Created()
       .content_type(ContentType::json())
       .body(response)
}

// Get all objeler
#[get("/objeler")]
async fn get_objeler(data: web::Data<AppState>) -> impl Responder {
   let objeler = data.objeler.lock().unwrap();

   let response = serde_json::to_string(&(*objeler)).unwrap();

   HttpResponse::Ok()
       .content_type(ContentType::json())
       .body(response)
}

// Get a obje with the corresponding id
#[get("/objeler/{id}")]
async fn get_obje(id: web::Path<u32>, data: web::Data<AppState>) -> Result<obje, ErrNoId> {
   let obje_id: u32 = *id;
   let objeler = data.objeler.lock().unwrap();

   let obje: Vec<_> = objeler.iter()
                               .filter(|x| x.id == obje_id)
                               .collect();

   if !obje.is_empty() {
       Ok(obje {
           id: obje[0].id,
           author: String::from(&obje[0].author)
       })
   } else {
       let response = ErrNoId {
           id: obje_id,
           err: String::from("obje not found")
       };
       Err(response)
   }
}

// Update the obje with the corresponding id
#[put("/objeler/{id}")]
async fn update_obje(id: web::Path<u32>, req: web::Json<obje>, data: web::Data<AppState>) -> Result<HttpResponse, ErrNoId> {
   let obje_id: u32 = *id;

   let new_obje = obje {
       id: req.id,
       author: String::from(&req.author),
   };

   let mut objeler = data.objeler.lock().unwrap();

   let id_index = objeler.iter()
                         .position(|x| x.id == obje_id);

   match id_index {
       Some(id) => {
           let response = serde_json::to_string(&new_obje).unwrap();
           objeler[id] = new_obje;
           Ok(HttpResponse::Ok()
               .content_type(ContentType::json())
               .body(response)
           )
       },
       None => {
           let response = ErrNoId {
               id: obje_id,
               err: String::from("obje not found")
           };
           Err(response)
       }
   }
}

// Delete the obje with the corresponding id
#[delete("/objeler/{id}")]
async fn delete_obje(id: web::Path<u32>, data: web::Data<AppState>) -> Result<obje, ErrNoId> {
   let obje_id: u32 = *id;
   let mut objeler = data.objeler.lock().unwrap();

   let id_index = objeler.iter()
                         .position(|x| x.id == obje_id);

   match id_index {
       Some(id) => {
           let deleted_obje = objeler.remove(id);
           Ok(deleted_obje)
       },
       None => {
           let response = ErrNoId {
               id: obje_id,
               err: String::from("obje not found")
           };
           Err(response)
       }
   }
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
   let app_state = web::Data::new(AppState {
                        objeler: Mutex::new(vec![
                            obje {
                                id: 1,
                                author: String::from("Jane Doe")
                            },
                            obje {
                                id: 2,
                                author: String::from("Patrick Star")
                                }
                        ])
                    });

   HttpServer::new(move || {
       App::new()
           .app_data(app_state.clone())
           .service(post_obje)
           .service(get_obje)
           .service(get_objeler)
           .service(update_obje)
           .service(delete_obje)
   })
   .bind(("127.0.0.1", 8000))?
   .run()
   .await
}