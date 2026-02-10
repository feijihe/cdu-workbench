
use sensors::Sensor;
use std::collections::HashMap;

pub struct Plugin {
    sensors: HashMap<String, Sensor>,
}