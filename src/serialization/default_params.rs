use crate::{serialization::xml, SerializationError};

use serde::{Deserialize, Serialize};
use std::{cell::RefCell, rc::Rc};
use xmltree::Element;

/// Used to factorize the extraction of twins parameters like volume A and volume B.
/// This enum allows to select which one you want.
pub enum TwinSelector {
    A,
    B,
}

impl TwinSelector {
    pub fn get_key<'l>(&self, key_a: &'l str, key_b: &'l str) -> &'l str {
        match self {
            TwinSelector::A => key_a,
            TwinSelector::B => key_b,
        }
    }
}

/// Wrapper helping to get values from the default params node.
/// There are few case of variables with 2 versions (A and B or 1 and 2). To handle that without duplicating code
/// I provide the enum TwinSelector.
pub struct DefaultParams<'l> {
    selector: TwinSelector,
    default_params_node: &'l Element,
}

impl<'l> DefaultParams<'l> {
    pub fn new(letter: TwinSelector, default_params_node: &'l Element) -> Self {
        Self {
            selector: letter,
            default_params_node,
        }
    }

    pub fn get_key<'a>(&self, key_a: &'a str, key_b: &'a str) -> &'a str {
        self.selector.get_key(key_a, key_b)
    }

    pub fn parse_twin_attribute<T: Deserialize<'l>>(&'l self, key_a: &'l str, key_b: &'l str) -> Result<T, SerializationError> {
        let key = self.get_key(key_a, key_b);

        xml::parse_attribute::<T>(self.default_params_node, key)
    }

    pub fn parse_twin_children_content<T: Deserialize<'l>>(&self, key_a: &'l str, key_b: &'l str) -> Result<T, SerializationError> {
        let key = self.get_key(key_a, key_b);

        xml::parse_children_element_content::<T>(self.default_params_node, key)
    }
}

pub struct DefaultParamsMut {
    selector: TwinSelector,
    default_params_node: Rc<RefCell<Element>>,
}

impl DefaultParamsMut {
    pub fn new(letter: TwinSelector, default_params_node: Rc<RefCell<Element>>) -> Self {
        Self {
            selector: letter,
            default_params_node,
        }
    }

    fn get_key<'a>(&self, key_a: &'a str, key_b: &'a str) -> &'a str {
        self.selector.get_key(key_a, key_b)
    }

    pub fn create_element(&self, name_a: &str, name_b: &str) -> Element {
        Element::new(self.selector.get_key(name_a, name_b))
    }

    pub fn insert_attribute<'a, T: Serialize>(&self, key_a: &'a str, key_b: &'a str, value: &T) -> Result<(), SerializationError> {
        let key = self.get_key(key_a, key_b);

        xml::insert_attribute(&mut self.default_params_node.borrow_mut(), key, value)
    }
}
