use std::collections::HashMap;
use std::fmt;
use std::sync::Mutex;

use crate::logger;

use lazy_static::lazy_static;

 pub struct ListenerList(HashMap<String, Listener>);

 impl ListenerList {

    pub fn push(&mut self, listener: Listener) {
        self.0.insert(listener.name.clone(), listener);
    }

    pub fn pop(&mut self) {
        todo!()
    }

    pub fn display(&self) {
        for (_, listener) in &self.0 {
            println!("{}", listener);
        }
    }
 }

 lazy_static! { static ref LISTENER_LIST: Mutex<ListenerList> = Mutex::new(ListenerList(HashMap::new())); }
 pub struct Listener {
    
    name: String,

    address: String,

    port: u16,

    uri: String,

    password: String,

}

impl Listener {

    pub fn new(name: &String, address: &String, port: u16, uri: &String, password: &String) {
        let res: Listener = Listener { name: name.clone(), address: address.clone(), port, uri: uri.clone(), password: password.clone() };
        logger::good(format!("Successfully added listener => {}", res).as_str());
        LISTENER_LIST.lock().unwrap().push(res);
        println!("--------------- Global Listener List --------------");
        LISTENER_LIST.lock().unwrap().display();
    }
}

impl std::fmt::Display for Listener {

    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Name: {}, Address: {}, Port: {}, Uri: {}, Password: {}", self.name, self.address, self.port, self.uri, self.password)
    }
}
