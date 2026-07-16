use std::collections::HashMap;

use bcrypt::DEFAULT_COST;
use rand::distr::{Alphanumeric, SampleString};
use tera::{Context, Kwargs, State, Tera, TeraResult};

use crate::constants::PASSWORD_LENGTH;

/// Renders the templated `content` by replacing template strings with the
/// appropiate `parameters`. Internally this uses [`tera`] to render the final
/// output. Available helper functions are:
///
/// - `random_password`: Returns a random password with a relatively secure RNG.
///   See [`rand::rng`] for more information.
/// - `bcrypt`: Returns the bcyrpt hash of the provided `password` parameter.
pub fn render(content: &str, parameters: &HashMap<String, String>) -> TeraResult<String> {
    // Create templating context
    let context = Context::from_serialize(parameters)?;

    // Create render engine
    let mut tera = Tera::new();
    tera.register_function("random_password", random_password);
    tera.register_function("bcrypt", bcrypt);

    // Render template
    tera.render(content, &context)
}

pub fn random_password(_: Kwargs, _: &State) -> TeraResult<String> {
    Ok(Alphanumeric.sample_string(&mut rand::rng(), PASSWORD_LENGTH))
}

pub fn bcrypt(kwargs: Kwargs, _: &State) -> TeraResult<String> {
    let password = kwargs.must_get::<String>("password")?;
    bcrypt::hash(password, DEFAULT_COST)
        .map_err(|err| tera::Error::message(format!("Failed to create bcrypt hash: {err}")))
}
