use std::collections::HashMap;
use std::boxed::Box;

pub struct Emitter {
    events: HashMap<String, Vec<Box<dyn Fn(&[u8])>>>,
}

impl Emitter {
    #[allow(dead_code)]
    pub fn new() -> Self {
        Self {
            events: HashMap::new(),
        }
    }
    #[allow(dead_code)]
    pub fn emit(&mut self, event: &str, args: &[u8]) {
        let event = event.to_string();
        if let Some(callbacks) = self.events.get(&event) {
            for callback in callbacks {
                callback(args);
            }
        }
        println!("emit: {:?}", args);
    }
    #[allow(dead_code)]
    pub fn on(&mut self, event: &str, callback: impl Fn(&[u8]) + 'static) {
        let event = event.to_string();
        self.events.entry(event).or_insert_with(|| Vec::new()).push(Box::new(callback));
    }
    #[allow(dead_code)]
    pub fn off(&mut self, event: &str) {
        self.events.remove(event);
    }
}
