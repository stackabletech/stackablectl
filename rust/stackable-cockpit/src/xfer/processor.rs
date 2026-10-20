use std::{collections::HashMap, marker::PhantomData};

use serde::de::DeserializeOwned;
use snafu::{ResultExt, Snafu};

use crate::utils::templating;

pub type Result<T, E = ProcessorError> = std::result::Result<T, E>;

#[derive(Debug, Snafu)]
pub enum ProcessorError {
    #[snafu(display("failed to deserialize YAML content"))]
    DeserializeYaml { source: yaml_serde::Error },

    #[snafu(display("failed to deserialize JSON content"))]
    DeserializeJson { source: serde_json::Error },

    #[snafu(display("failed to render templated content"))]
    RenderTemplate { source: tera::Error },
}

pub trait Processor: Sized {
    type Input;
    type Output;

    /// Processes the input and returns the altered content.
    fn process(&self, input: Self::Input) -> Result<Self::Output>;

    /// Chains this processors output as the input of the `other` processor.
    fn then<P: Processor<Input = Self::Output>>(self, other: P) -> Chain<Self, P> {
        Chain(self, other)
    }
}

#[derive(Debug)]
pub struct Chain<P1, P2>(P1, P2);

impl<P1, P2> Processor for Chain<P1, P2>
where
    P1: Processor,
    P2: Processor<Input = P1::Output>,
{
    type Input = P1::Input;
    type Output = P2::Output;

    fn process(&self, input: Self::Input) -> Result<Self::Output> {
        self.1.process(self.0.process(input)?)
    }
}

/// Process plain text without any encodings like YAML or JSON and without any
/// templating applied to it.
#[derive(Debug, Default)]
pub struct Text;

impl Processor for Text {
    type Input = String;
    type Output = String;

    fn process(&self, input: Self::Input) -> Result<Self::Output> {
        Ok(input)
    }
}

/// Process the contents by parsing it as YAML.
#[derive(Debug)]
pub struct Yaml<T>(PhantomData<T>);

impl<T> Processor for Yaml<T>
where
    T: DeserializeOwned,
{
    type Input = String;
    type Output = T;

    fn process(&self, input: Self::Input) -> Result<Self::Output> {
        yaml_serde::from_str(&input).context(DeserializeYamlSnafu)
    }
}

impl<T> Default for Yaml<T> {
    fn default() -> Self {
        Self(PhantomData)
    }
}

#[derive(Debug)]
pub struct Json<T>(PhantomData<T>);

impl<T> Processor for Json<T>
where
    T: DeserializeOwned,
{
    type Input = String;
    type Output = T;

    fn process(&self, input: Self::Input) -> Result<Self::Output> {
        serde_json::from_str(&input).context(DeserializeJsonSnafu)
    }
}

impl<T> Default for Json<T> {
    fn default() -> Self {
        Self(PhantomData)
    }
}

/// Process the contents by rendering templated parts of the contents.
#[derive(Debug)]
pub struct Template<'a>(&'a HashMap<String, String>);

impl Processor for Template<'_> {
    type Input = String;
    type Output = String;

    fn process(&self, input: Self::Input) -> Result<Self::Output> {
        templating::render(&input, self.0).context(RenderTemplateSnafu)
    }
}

impl<'a> Template<'a> {
    pub fn new(parameters: &'a HashMap<String, String>) -> Self {
        Self(parameters)
    }
}
