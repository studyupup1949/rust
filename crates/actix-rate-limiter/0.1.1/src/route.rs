//! Route builder and route representation itself.

use regex::Regex;

/// Builder for the `Route`.
#[derive(Clone, Debug)]
pub struct RouteBuilder {
    path: &'static str,
    method: Option<&'static str>,
    regex: Option<Regex>,
}

impl RouteBuilder {
    /// Creates a new instance of `RouteBuilder`.
    pub fn new() -> Self {
        Self {
            path: "",
            method: None,
            regex: None,
        }
    }

    /// Sets path for the route.
    pub fn set_path(&mut self, path: &'static str) -> &mut Self {
        self.path = path;
        self
    }

    /// Sets method for the route. If method isn't set, then limits will be
    /// applied to all available methods.
    pub fn set_method(&mut self, method: &'static str) -> &mut Self {
        self.method = Some(method);
        self
    }

    /// Enables regex compilation for this route.
    pub fn enable_regex(&mut self) -> &mut Self {
        self.regex = Some(Regex::new(format!("^({})$", self.path).as_str()).unwrap());
        self
    }

    /// Builds the instance of the builder and converts it into a `Route`
    /// instance with the set values.
    pub fn build(&self) -> Route {
        // TODO: add checks?
        Route {
            path: self.path,
            method: self.method,
            regex: self.regex.clone(),
        }
    }
}

impl Default for RouteBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Representation of a single Actix route.
#[derive(Clone, Debug)]
pub struct Route {
    path: &'static str,
    method: Option<&'static str>,
    regex: Option<Regex>,
}

impl Route {
    pub(crate) fn is_match(&self, route: &str, method: &str) -> bool {
        if self.method.is_some() && self.method.unwrap() != method {
            return false;
        }

        if self.regex.is_some() {
            return self.regex.clone().unwrap().is_match(route);
        }

        self.path == route
    }
}
