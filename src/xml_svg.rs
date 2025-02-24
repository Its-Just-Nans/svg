use core::fmt;

use crate::{
    node::element::{tag::Type, Element},
    parser::{Error, Event},
    Node, Parser,
};

#[derive(Debug)]
pub struct XMLSvg {
    inner: Vec<Box<dyn Node>>,
}

impl XMLSvg {
    pub fn from_childrens(childrens: &Vec<Box<dyn Node>>) -> Self {
        Self {
            inner: childrens.to_vec(),
        }
    }

    pub fn from_string(svg_str: &str) -> Result<Self, Error> {
        let svg = Element::new("document");
        let mut stack: Vec<Element> = Vec::new();
        stack.push(svg.clone());

        for event in Parser::new(svg_str) {
            match event {
                Event::Tag(tag, typed, attributes) => {
                    let mut node = Element::new(tag);
                    node.get_attributes_mut().extend(attributes);
                    match typed {
                        Type::Start => {
                            stack.push(node);
                        }
                        Type::End => {
                            if stack.len() > 1 {
                                let to_add = stack.pop().unwrap();
                                if let Some(parent) = stack.last_mut() {
                                    parent.append(to_add);
                                }
                            }
                        }
                        Type::Empty => {
                            if let Some(parent) = stack.last_mut() {
                                parent.append(node);
                            }
                        }
                    }
                }
                Event::Comment(comment) => {
                    // remove 4 first chart and 3 last chart
                    if let Some(parent) = stack.last_mut() {
                        parent.append(crate::node::Comment::new(comment));
                    }
                }
                Event::Text(text) => {
                    if let Some(parent) = stack.last_mut() {
                        parent.append(crate::node::Text::new(text));
                    }
                }
                Event::Declaration(declaration) => {
                    if let Some(parent) = stack.last_mut() {
                        parent.append(crate::node::Blob::new(declaration));
                    }
                }
                Event::Instruction(instruction) => {
                    if let Some(parent) = stack.last_mut() {
                        parent.append(crate::node::Blob::new(instruction));
                    }
                }
                Event::Error(e) => {
                    // return the error
                    return Err(e);
                }
            }
        }

        if let Some(root) = stack.first() {
            return Ok(XMLSvg::from_childrens(root.get_children()));
        }

        Err(Error::new((0, 0), "No root element found"))
    }

    pub fn get_svg(&self) -> Option<&dyn Node> {
        self.inner.iter().find_map(|node| {
            if node.get_name() == "svg" {
                Some(node.as_ref())
            } else {
                None
            }
        })
    }
}

impl fmt::Display for XMLSvg {
    #[inline]
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        for node in &self.inner {
            node.fmt(formatter)?;
            writeln!(formatter)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::XMLSvg;

    #[test]

    fn high_level_parsing() {
        let svg = include_str!("../tests/fixtures/benton.svg");
        // replace line endings
        let svg = svg.replace("\r\n", "\n");
        let xml_svg = XMLSvg::from_string(&svg).unwrap();
        println!("{:#?}", xml_svg);
        // assert only the first 60 characters
        // attributes are not in the same order
        assert_eq!(xml_svg.to_string()[..60], svg[..60]);
        assert_eq!(xml_svg.get_svg().unwrap().get_name(), "svg")
    }
}
