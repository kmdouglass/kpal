extern crate kpal;

use std::sync::Arc;
use kpal::peripheral::{Peripheral, Property, Value};

pub struct HelloWorld { }

impl Peripheral for HelloWorld {
    fn properties<'a>(&'a self) -> Vec<Property<'a>> {
        let props: Vec<Property> = vec![
            Property {
                name: "x",
                value: Value::Float(0.0),
            },
        ];
    
        props
    }
}

#[no_mangle]
pub fn get_peripheral() -> Arc<HelloWorld> {
    Arc::new(HelloWorld {} )
}
