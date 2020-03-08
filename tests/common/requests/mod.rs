//! Datatypes and traits for modeling HTTP requests and responses.
use std::boxed::Box;

use {
    reqwest::{Client, Error as ReqwestError, RequestBuilder, Response},
    serde::Serialize,
    url::Url,
};

/// The set HTTP request verbs used by the tests.
#[derive(Debug)]
pub enum HttpVerb {
    Get,
    Post,
    Patch,
}

pub trait Request {
    fn exec(&self, client: &Client) -> Result<Response, ReqwestError>;

    fn init(&self) -> Box<dyn Fn(&Client, &Url) -> RequestBuilder> {
        match self.verb() {
            HttpVerb::Get => Box::new(|client: &Client, url: &Url| client.get(url.as_str())),
            HttpVerb::Post => Box::new(|client: &Client, url: &Url| client.post(url.as_str())),
            HttpVerb::Patch => Box::new(|client: &Client, url: &Url| client.patch(url.as_str())),
        }
    }

    fn url(&self) -> &Url;

    fn verb(&self) -> HttpVerb;
}

/// A GET request.
pub struct Get {
    url: Url,
}

impl Get {
    pub fn new(domain: &Url, route: &str) -> Get {
        let url = domain
            .join(route)
            .expect("Could not produce full URL for the test");
        Get { url }
    }
}

impl Request for Get {
    fn exec(&self, client: &Client) -> Result<Response, ReqwestError> {
        self.init()(client, &self.url()).send()
    }

    fn url(&self) -> &Url {
        &self.url
    }

    fn verb(&self) -> HttpVerb {
        HttpVerb::Get
    }
}

/// A POST request.
pub struct Post<T: Serialize> {
    data: T,
    url: Url,
}

impl<T: Serialize> Post<T> {
    pub fn new(domain: &Url, route: &str, data: T) -> Post<T> {
        let url = domain
            .join(route)
            .expect("Could not produce full URL for the test");
        Post { data, url }
    }
}

impl<T: Serialize> Request for Post<T> {
    fn exec(&self, client: &Client) -> Result<Response, ReqwestError> {
        self.init()(client, &self.url()).json(&self.data).send()
    }

    fn url(&self) -> &Url {
        &self.url
    }

    fn verb(&self) -> HttpVerb {
        HttpVerb::Post
    }
}

/// A PATCH request.
pub struct Patch<T: Serialize> {
    data: T,
    url: Url,
}

impl<T: Serialize> Patch<T> {
    pub fn new(domain: &Url, route: &str, data: T) -> Patch<T> {
        let url = domain
            .join(route)
            .expect("Could not produce full URL for the test");
        Patch { data, url }
    }
}

impl<T: Serialize> Request for Patch<T> {
    fn exec(&self, client: &Client) -> Result<Response, ReqwestError> {
        self.init()(client, &self.url).json(&self.data).send()
    }

    fn url(&self) -> &Url {
        &self.url
    }

    fn verb(&self) -> HttpVerb {
        HttpVerb::Patch
    }
}
