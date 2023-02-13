use std::{collections::HashMap, error::Error};

use bcrypt::{hash, DEFAULT_COST};
use passwords::PasswordGenerator;
use tera::{from_value, Function, Tera, Value};

const PASSWORD_LEN: usize = 30;

pub fn new_templating_instance() -> Result<Tera, Box<dyn Error>> {
    // tera requires a glob of templates files to parse to offer better performance.
    // As we only template the files once there is no benefit to collect the needed files and pass it in here.
    // We just give some non-existent file and pass the filename to template during templating itself.
    let mut tera = Tera::new("this-folder-does-not-exist/*.yaml")?;
    tera.register_function("random_password", random_password());
    tera.register_function("bcrypt", bcrypt());
    Ok(tera)
}

fn random_password() -> impl Function {
    Box::new(
        move |_args: &HashMap<String, Value>| -> tera::Result<Value> {
            let password = PasswordGenerator::new()
                .length(PASSWORD_LEN)
                .generate_one()
                .map_err(|err| format!("Failed to generate password: {err}"))?;
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
