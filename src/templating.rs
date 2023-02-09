use std::{collections::HashMap, error::Error};

use bcrypt::{hash, DEFAULT_COST};
use rand::Rng;
use tera::{from_value, Function, Tera, Value};

const PASSWORD_CHARSET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789";
const PASSWORD_LEN: usize = 30;

pub fn new_templating_instance() -> Result<Tera, Box<dyn Error>> {
    let mut tera = Tera::new("this-folder-does-not-exist/*.yaml")?;
    tera.register_function("random_password", random_password());
    tera.register_function("bcrypt", bcrypt());
    Ok(tera)
}

fn random_password() -> impl Function {
    Box::new(
        move |_args: &HashMap<String, Value>| -> tera::Result<Value> {
            let mut rng = rand::thread_rng();
            let password: String = (0..PASSWORD_LEN)
                .map(|_| {
                    let idx = rng.gen_range(0..PASSWORD_CHARSET.len());
                    PASSWORD_CHARSET[idx] as char
                })
                .collect();

            Ok(password.into())
        },
    )
}

fn bcrypt() -> impl Function {
    Box::new(
        move |args: &HashMap<String, Value>| -> tera::Result<Value> {
            match args.get("password") {
                Some(val) => match from_value::<String>(val.clone()) {
                    Ok(password) => {
                        let hash = hash(password, DEFAULT_COST).unwrap();
                        Ok(hash.into())
                    }
                    Err(_) => Err("Cant get value of password".into()),
                },
                None => Err("Parameter password missing".into()),
            }
        },
    )
}
