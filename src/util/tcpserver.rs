use std::{
    collections::HashMap, convert::Infallible, net::{Incoming, SocketAddr, TcpStream}, 
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    clone::Clone,
};

use http_body_util::{BodyExt, Full};
use hyper::body::Bytes;
use hyper::server::conn::http1;
use hyper::service::{service_fn, Service};
use hyper::{body::Incoming as IncomingBody, Request, Response, Method};
use hyper_util::rt::TokioIo;

use tokio::{
    sync::Mutex,
    net::TcpListener,
    task,
};

use uuid::Uuid;

use serde_json::{Value, json};
use warp::test;

use super::tasks::TaskManager;
pub struct HttpServer {

     local_addr: SocketAddr,
     closed: Arc<AtomicBool>,
     connections: Arc<Mutex<HashMap<SocketAddr, TcpStream>>>,
     t_manager: TaskManager,
     handle: Arc<Mutex<Option<task::JoinHandle<()>>>>,

}

impl Clone for HttpServer {
    fn clone(&self) -> Self {
        Self {
            local_addr: self.local_addr.clone(),
            closed: Arc::clone(&self.closed),
            connections: Arc::clone(&self.connections),
            t_manager: self.t_manager.clone(),
            handle: Arc::clone(&self.handle),
        }
    }
}

impl HttpServer {
    pub async fn handle_request(server: &HttpServer, req: Request<IncomingBody>) -> Result<Response<Full<Bytes>>, Infallible> {

        let endpoint = req.uri().path();
      
        match endpoint {

            "/tasks" => {

                match req.method() {
                    &Method::GET => {

                        let bytes = req.collect().await.unwrap().to_bytes();

                        let json: Value = serde_json::from_str(String::from_utf8(bytes.to_vec()).unwrap().as_str()).unwrap();

                        println!("{}", json);

                        if let Some(value) = json.get("agent-id") {
                            println!("Value: {}", value);
                            let value = value.to_string();
                            println!("Value as a String: {}", value);
                            
                            match Uuid::parse_str(&value.trim_matches('"').trim()) {
                                Ok(v) => { 
                                    println!("Successfully parsed Uuid: {}", v);
                                    match server.task_get(&v).await {
                                        Ok(r) => return Ok(r),
                                        Err(e) => {
                                            eprintln!("Error fetching tasks: {e}");
                                            return Ok(Response::new(Full::new(Bytes::from("[]")))); 
                                        }
                                    }
                                },
                                Err(e) => { 
                                    eprintln!("Error parsing Uuid! {e}");
                                    return Ok(Response::new(Full::new(Bytes::from("[]")))); 
                                },
                            };

                            return Ok(Response::new(Full::new(Bytes::from("[]")))); 
                        } else {
                            return Ok(Response::new(Full::new(Bytes::from("Incorrectly formatted Http Request!"))));
                        }

                        

                        // println!("{}", json);

                        
                    },
                    &Method::POST => {
                        return Ok(Response::new(Full::new(Bytes::from("[]"))));
                    },
                    _ => {
                        return Ok(Response::new(Full::new(Bytes::from("METHOD NOT ALLOWED"))));
                    },
                }
            },
            "/echo" => {
                println!("{:#?}", req);
                return Ok(Response::new(Full::new(Bytes::from("[]"))))
            }

            _ => return Ok(Response::new(Full::new(Bytes::from("Not a valid endpoint!")))),

        }
        Ok(Response::new(Full::new(Bytes::from("uh oh fucky aaaaa!!!!"))))

    }

    pub async fn start(&mut self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {

        let listener = TcpListener::bind(self.local_addr).await?;

        let server_arc = Arc::new(Mutex::new(self.clone()));

        let server_clone = Arc::clone(&server_arc);

        server_clone.lock().await.populate_test_tasks().await;

        let handle = tokio::task::spawn(async move {
            println!("ws main thread spawned");
            loop {

                let (stream, _) = listener.accept().await.unwrap();

                let io = TokioIo::new(stream);
                
                // Clone the Arc again for the service function
                let server_clone = Arc::clone(&server_arc);
                
                if let Err(err) = http1::Builder::new()
                    .serve_connection(io, service_fn(move |req| {
                        let server_clone = Arc::clone(&server_clone); // Clone for each request
                        async move {

                            // Lock the mutex to access server instance
                            let server = server_clone.lock().await;
                            HttpServer::handle_request(&*server, req).await
                        }
                    }))
                    .await
                {
                    println!("Error serving connection: {:?}", err);
                }
            }
        });

        *self.handle.lock().await = Some(handle);

        Ok(())
    }


    pub async fn close(&mut self) {

        let handle_clone = Arc::clone(&self.handle);

        let mut lock = handle_clone.lock().await;

        match lock.take() {
            Some(h) => {
                h.abort();
                *lock = None;
            },
            None => {
                eprintln!("HttpServer is not running... ");
            }
        }
    }

    pub async fn new () -> std::io::Result<Self> {

        Ok(Self { 
            local_addr: SocketAddr::from(([127, 0, 0, 1], 42069)), 
            closed: Arc::new(AtomicBool::new(false)), 
            connections: Arc::new(Mutex::new(HashMap::new())), 
            t_manager: TaskManager::new(),
            handle: Arc::new(Mutex::new(None)),
        })
    }

    pub async fn task_post(&self, uuid: &Uuid) -> Result<Response<Full<Bytes>>, Infallible> {
        todo!();
    }

    pub async fn task_get(&self, uuid: &Uuid) -> Result<Response<Full<Bytes>>, Infallible> {
        
        if let Some(v) = self.t_manager.get_tasks(uuid).await {
            let joined = v.join("");
            Ok(Response::new(Full::new(Bytes::from(joined))))
        } else {
            Ok(Response::new(Full::new(Bytes::from("No tasks found for that uuid!"))))
        }
    }

    async fn populate_test_tasks(&self) {
        let test_uuid = Uuid::new_v4();

        println!("Test uuid: {}", test_uuid);
        self.t_manager.populate_test_entry(&test_uuid).await;
    }

}

// #[derive(Debug, Clone)]
// struct Svc {
//     server: Arc<Mutex<HttpServer>>,
// }

// impl Service<Request<IncomingBody>> for Svc {

// }


#[cfg(test)]
mod tests {

    use super::*;

    #[tokio::test]
    async fn test_task_get_api() {

        let mut http_server = HttpServer::new().await.unwrap();

        if let Err(e) = http_server.start().await {
            eprintln!("Error starting Http Server: {e}");
        }

        assert_eq!(1, 1);
    }

}