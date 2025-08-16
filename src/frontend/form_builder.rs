use std::{collections::HashMap, ops::Mul, sync::{Arc, Mutex}};
use serde_json::{Value, json,Map};


use modx::{store,props};
use dioxus::prelude::*;

type ValidatorFn = Arc<dyn Fn(&String) -> bool + Send + Sync>;

#[derive(Clone)]
pub struct Validator {
    name: String,
    func: ValidatorFn,
}

impl Validator {
    pub fn new<F>(name: &str,func: F, ) -> Self
    where
        F: Fn(&String) -> bool + Send + Sync + 'static,
    {
        Self {
            name: name.to_string(),
            func: Arc::new(func)
        }
    }

    pub fn required() -> impl Fn(&String) -> bool {
        |value:&String| value.is_empty()
    }
    
    pub fn pattern(pattern: &str) -> Box<dyn Fn(&String) -> bool + Send + Sync> {
        use regex::Regex;
        let re = Regex::new(pattern);

        match re {
            Ok(re) => {
                // Box the closure that uses the compiled regex
                Box::new(move |value: &String| !re.is_match(value))
            }
            Err(_) => {
                // Box a fallback closure that always returns false
                Box::new(|_: &String| false)
            }
        }
    } 
}

#[derive(PartialEq)]
#[props(value,errors)]
#[store]
pub struct FormControl {
    value: String,
    validators: Vec<Validator>,
    errors: Vec<String>
}

impl FormControl {

    pub fn control(initial: String, validators: Vec<Validator>) -> Self {
        Self {
            value: Signal::new(initial),
            validators: Signal::new(validators),
            errors: Signal::new([].to_vec())
        }
    }

    pub fn set_value(&mut self,value: String) {
        self.value.set(value);
    }

    pub fn get_value(&self) -> String {
        self.value()
    }

    pub fn get_raw_value(&self) -> &Signal<String> {
        &self.value
    }

    pub fn validate(&mut self) -> Vec<String> {
        let errors = self.validators().iter().filter_map(|v| {
            match (v.func)(&self.value()) {
                true => Some(v.name.clone()),
                false => None
            }
        }).collect();
        self.errors.set(errors);
        self.errors()
    }

    pub fn has_error(&self, name: String) -> bool {
        let validator = self.validators()
        .iter()
        .find(|v| v.name == name)
        .map(|v| Arc::clone(&v.func));
        match validator {
            Some(func) => func(&self.value()),
            None => false
        }
    }
}

#[store]
pub struct FormGroup {
    controls: HashMap<String, Arc<Mutex<FormControl>>>,
}

impl FormGroup {
    pub fn builder() -> Self {
        Self {
            controls: Signal::new(HashMap::new()),
        }
    }

    pub fn add_control(&mut self, name: &str, control: FormControl){
        self.controls.with_mut(|controls| {
            controls.insert(name.to_string(), Arc::new(Mutex::new(control)));
        });
    }

    pub fn get_control(&self, name: &str) -> Option<Arc<Mutex<FormControl>>> {
        self.controls().get(name).cloned()
    }

    pub fn validate_all(&self) -> HashMap<String, Vec<String>> {
        self.controls()
        .iter()
        .filter(|f| !f.1.lock().unwrap().validate().is_empty())
        .map(|(name, control)| (name.clone(), control.lock().unwrap().validate()))
        .collect()
    }

    pub fn to_json(&self) -> Map<String,Value> {
        self.controls()
        .iter()
        .map(|(key,control)| (key.clone(),serde_json::to_value(control.lock().unwrap().value()).unwrap()))
        .collect::<Map<String,Value>>()
    }
}
