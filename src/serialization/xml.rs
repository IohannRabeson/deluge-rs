use crate::Error;

use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::{cell::RefCell, rc::Rc};
use xmltree::{Element, EmitterConfig, XMLNode};

pub fn write_xml(elements: &[Element]) -> String {
    let mut buffer: Vec<u8> = Vec::with_capacity(1024);
    let mut config: EmitterConfig = EmitterConfig::new();

    config.perform_indent = true;
    for element in elements {
        element.write_with_config(&mut buffer, config.clone()).unwrap();
    }

    String::from_utf8(buffer).unwrap()
}

pub fn load_xml(xml: &str) -> Result<Vec<Element>, Error> {
    Ok(Element::parse_all(xml.as_bytes())
        .map_err(|e| Error::XmlParsingFailed(Arc::new(e)))?
        .iter()
        .filter_map(|n| n.as_element())
        .cloned()
        .collect::<Vec<Element>>())
}

pub fn keep_element_only(node: &XMLNode) -> Option<&Element> {
    node.as_element()
}

pub fn get_element<'a>(elements: &'a [Element], name: &'a str) -> Result<&'a Element, Error> {
    get_opt_element(elements, name).ok_or_else(|| Error::MissingElement(name.to_string()))
}

pub fn get_opt_element<'a>(elements: &'a [Element], name: &'a str) -> Option<&'a Element> {
    elements.iter().find(|e| e.name == name)
}

pub fn get_attribute<'a>(element: &'a Element, name: &'a str) -> Result<&'a String, Error> {
    get_opt_attribute(element, name).ok_or_else(|| Error::MissingAttribute(element.name.to_string(), name.to_string()))
}

pub fn get_opt_attribute<'a>(element: &'a Element, name: &'a str) -> Option<&'a String> {
    match element.attributes.get(name) {
        Some(attribute_value) => Some(attribute_value),
        None => None,
    }
}

pub fn get_children_element<'a>(element: &'a Element, name: &'a str) -> Result<&'a Element, Error> {
    get_opt_children_element(element, name).ok_or_else(|| Error::MissingChild(element.name.to_string(), name.to_string()))
}

pub fn get_opt_children_element<'a>(element: &'a Element, name: &'a str) -> Option<&'a Element> {
    element.children.iter().filter_map(keep_element_only).find(|e| e.name == name)
}

pub fn get_all_children_element_with_name<'a>(element: &'a Element, name: &'a str) -> Vec<&'a Element> {
    element
        .children
        .iter()
        .filter_map(keep_element_only)
        .filter(|e| e.name == name)
        .collect()
}

pub fn get_children_element_content<'a>(element: &'a Element, name: &'a str) -> Result<String, Error> {
    get_children_element(element, name).map(get_text)
}

pub fn parse_children_element_content<'a, T: Deserialize<'a>>(element: &'a Element, name: &'a str) -> Result<T, Error> {
    let element = get_children_element(element, name)?;

    parse_content(element)
}

pub fn parse_opt_children_element_content<'a, T: Deserialize<'a>>(
    element: &'a Element,
    name: &'a str,
) -> Result<Option<T>, Error> {
    Ok(match get_opt_children_element(element, name) {
        Some(element) => Some(parse_content(element)?),
        None => None,
    })
}

pub fn get_text(element: &Element) -> String {
    element
        .get_text()
        .unwrap_or_else(|| std::borrow::Cow::Owned("".to_string()))
        .into_owned()
}

pub fn parse_attribute<'a, T: Deserialize<'a>>(element: &'a Element, name: &'a str) -> Result<T, Error> {
    serde_plain::from_str::<T>(get_attribute(element, name)?).map_err(Error::SerdeError)
}

const NULL_STRING: &str = "";

fn get_text_impl<'a>(element: &'a Element) -> &'a str {
    let text_nodes: Vec<&'a str> = element
        .children
        .iter()
        .filter_map(|node| node.as_text().or_else(|| node.as_cdata()))
        .collect();

    // Hack: to be able to use serde_plain, I must return a reference with the lifetime 'a.
    // Returning NULL_STRING works because its lifetime is implicitly 'static that outlives 'a.
    if !text_nodes.is_empty() {
        text_nodes[0]
    } else {
        NULL_STRING
    }
}

pub fn parse_content<'a, T: Deserialize<'a>>(element: &'a Element) -> Result<T, Error> {
    serde_plain::from_str::<T>(get_text_impl(element)).map_err(Error::SerdeError)
}

pub fn parse_opt_attribute<'a, T: Deserialize<'a>>(element: &'a Element, name: &'a str) -> Result<Option<T>, Error> {
    let mut result = None;

    if let Some(attribute) = element.attributes.get(name) {
        result = Some(serde_plain::from_str::<T>(attribute).map_err(Error::SerdeError)?);
    }

    Ok(result)
}

pub fn insert_attribute<T: Serialize>(element: &mut Element, attribute_name: &str, value: &T) -> Result<(), Error> {
    let value_as_string = serde_plain::to_string::<T>(value).map_err(Error::SerdeError)?;

    element.attributes.insert(attribute_name.to_owned(), value_as_string);

    Ok(())
}

pub fn insert_opt_attribute<T: Serialize>(element: &mut Element, attribute_name: &str, value: &Option<T>) -> Result<(), Error> {
    if let Some(value) = value {
        insert_attribute(element, attribute_name, value)?;
    }

    Ok(())
}

pub fn insert_opt_attribute_if_not_default<T: Serialize + Default + PartialEq>(
    element: &mut Element,
    attribute_name: &str,
    value: &T,
) -> Result<(), Error> {
    if value != &T::default() {
        insert_attribute(element, attribute_name, value)?;
    }

    Ok(())
}

pub fn insert_child(element: &mut Element, child: Element) -> Result<(), Error> {
    element.children.push(XMLNode::Element(child));
    Ok(())
}

pub fn insert_child_rc(element: &Rc<RefCell<Element>>, child: Element) {
    element.borrow_mut().children.push(XMLNode::Element(child));
}

pub fn insert_attribute_rc<T: Serialize>(element: &Rc<RefCell<Element>>, attribute_name: &str, value: &T) -> Result<(), Error> {
    let value_as_string = serde_plain::to_string::<T>(value).map_err(Error::SerdeError)?;

    element
        .borrow_mut()
        .attributes
        .insert(attribute_name.to_owned(), value_as_string);

    Ok(())
}
