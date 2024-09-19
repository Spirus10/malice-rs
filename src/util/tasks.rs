use std::{
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    collections::{
        HashMap,
        hash_map::Entry,
    },
    clone::Clone,
};

use tokio::{
    sync::Mutex,
    task,
};
use serde::{Serialize, Deserialize};
use serde_json::{Value, json};

use uuid::Uuid;


#[derive(Serialize, Deserialize)]
pub struct Task {
    task_type: String,
    task_options: Vec<String>,
}

impl Task {

    pub fn new(task_type: String, task_options: Vec<String>) -> Self {
        Self {
             task_type: task_type,
             task_options: task_options,
        }
    }
}

impl Clone for Task {
    fn clone(&self) -> Self {
        Self {
            task_type: self.task_type.clone(),
            task_options: self.task_options.clone(),
        }
    }
}

pub struct TaskManager {
    tasks: Arc<Mutex<HashMap<Uuid, Vec<Task>>>>,
}

impl Clone for TaskManager {
    fn clone(&self) -> Self {
        Self {
            tasks: Arc::clone(&self.tasks),
        }
    }
}

impl TaskManager {

    pub fn new() -> Self {
        Self {
            tasks: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub async fn add_task(&self, uuid: &Uuid, task: Task) {
        
        let mut lock = self.tasks.lock().await;

        // let clone = Arc::clone(&self.tasks);

        match lock.entry(*uuid) {
            Entry::Vacant(e) => { e.insert(vec![task]); },
            Entry::Occupied(mut e) => { e.get_mut().push(task); },
        }
    }

    pub async fn get_tasks(&self, uuid: &Uuid) -> Option<Vec<String>> {
        
        let mut lock = self.tasks.lock().await;

        match lock.entry(*uuid) {
            Entry::Vacant(e) => { println!("No Task list for that UUID"); None  },
            Entry::Occupied(mut e) => { 
                
                let mut vec: Vec<String> = vec![];

                if !e.get().is_empty() {
                    for task in e.get().iter() {
                        vec.push(serde_json::to_string(&task).unwrap());
                    }
                    e.get_mut().clear();
                    return Some(vec)
                }
                None

            },
        }

    }

    pub async fn populate_test_entry(&self, uuid: &Uuid) {
        self.add_task(uuid, Task { task_type: "ping".to_string(), task_options: vec!["verbose".to_string(), "cock".to_string(), "balls".to_string()] }).await;
    }
}


#[cfg(test)]
mod tests {

use super::*;

#[test]
    fn test_json_output() {

        let task_comparison = r#"
            {
                "task_type": "ping",
                "task_options": [
                    "verbose",
                    "cock",
                    "balls",
                ],
                "received": false,
            }"#;

        let task = Task::new("ping".to_string(), vec!["verbose".to_string(), "cock".to_string(), "balls".to_string()]);

        println!("{}", serde_json::to_string_pretty(&task).unwrap());
        println!("{:#?}", serde_json::to_vec_pretty(&task).unwrap());

        assert_eq!(serde_json::to_string(&task).unwrap(), task_comparison);
    }

#[tokio::test]
    async fn test_task_manager_add_function() {

        let t_manager = TaskManager::new();

        let uuid = Uuid::new_v4();

        let task = Task::new("ping".to_string(), vec!["verbose".to_string(), "cock".to_string(), "balls".to_string()]);

        let task1 = task.clone();

        t_manager.add_task(&uuid, task).await;

        let og = serde_json::to_string(&t_manager.get_tasks(&uuid).await.unwrap()).unwrap();

        assert_eq!(og, serde_json::to_string(&task1).unwrap());

    }
}
