use std::ops::Index;
use std::collections::hash_map::HashMap;
use core::fmt::Debug;

pub struct Pedigree {
    pub added: Option<&'static str>,
    pub added_ep: Option<&'static str>,
    pub updated: Option<&'static str>,
    pub updated_ep: Option<&'static str>,
    pub deprecated: Option<&'static str>,
    pub deprecated_ep: Option<&'static str>
}

#[derive(Debug)]
pub struct FieldValue {
    pub tag: u32,
    pub name: &'static str,
    pub value: &'static str
}

pub trait OrchestrationField {
    fn tag(&self) -> u32;
    fn is_data(&self) -> bool;
    fn name(&self) -> &'static str;
    fn data_type(&self) -> &'static str;
    fn synopsis(&self) -> &'static str;
    fn pedigree(&self) -> crate::dictionary::Pedigree { 
        crate::dictionary::Pedigree {
            added: None,
            added_ep: None,
            updated: None,
            updated_ep: None,
            deprecated: None,
            deprecated_ep: None
        } 
    } 
    fn values(&self) -> &'static Vec<&'static crate::dictionary::FieldValue>;
    fn is_valid(&self) -> bool { self.tag() != 0 }

    // Return the name of an enumerated value if it is defined for this field e.g. 1 -> 'Buy' for Side.
    // Returns an empty string if no such value is defined.
    fn name_of_value(&self, value: &str) -> Option<&'static str>
    {
        match self.values().iter().find(|&item| item.value == value) {
            None => None,
            Some(entry) => Some(entry.name)
        }
    }

    fn is_numeric(&self) -> bool
    {
        matches!(
            self.data_type().to_lowercase().as_str(),
            "int" | "length" | "tagnum" | "seqnum" | "numingroup" 
            | "float" | "qty" | "price" | "priceoffset" 
            | "amt" | "percentage"
        )
    }

}

impl Debug for dyn OrchestrationField {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "OrchestrationField {{ name: \"{}\", tag: {} }}", self.name(), self.tag())
    }
}

pub struct InvalidField {
}

impl crate::dictionary::OrchestrationField for InvalidField {

    fn tag(&self) -> u32 { 0 }
    fn is_data(&self) -> bool { false }
    fn name(&self) -> &'static str { "" }
    fn data_type(&self) -> &'static str { "" }
    fn synopsis(&self) -> &'static str { "" }
    
    fn pedigree(&self) -> crate::dictionary::Pedigree {
        crate::dictionary::Pedigree {
            added: None,
            added_ep: None,
            updated: None,
            updated_ep: None,
            deprecated: None,
            deprecated_ep: None
        }
    }
    
    fn values(&self) -> &'static Vec<&'static crate::dictionary::FieldValue> {
        static VALUES: std::sync::OnceLock<Vec<&'static crate::dictionary::FieldValue>> = std::sync::OnceLock::new();
        VALUES.get_or_init(|| {
            vec![]
        })
    }
}

pub struct OrchestrationFieldCollection {

    offsets: Vec<usize>,
    fields: Vec<Box<dyn OrchestrationField>>,
    offsets_by_name: HashMap<&'static str, usize>
}

impl OrchestrationFieldCollection {

    pub fn new(offsets: Vec<usize>, fields: Vec<Box<dyn OrchestrationField>>) -> Self
    {
        let offsets_by_name: HashMap<&'static str, usize> = offsets
            .iter()
            .filter(|offset| **offset != 0)
            .map(|offset| {
                let field = &fields[*offset];

                (field.name(), *offset)
            })
            .collect();

        Self { offsets, fields, offsets_by_name }
    }

    pub fn len(&self) -> usize {
        self.fields.len()
    }

    pub fn is_empty(&self) -> bool {
        self.fields.is_empty()
    }

    pub fn is_tag_valid(&self, tag: usize) -> bool
    {
        match self.offsets
                .get(tag)
                .and_then(|&offset| self.fields.get(offset))
                .map(|field| field.is_valid())
        {
            Some(is_valid) => is_valid,
            None => false
        }
    }

    pub fn name_of_field(&self, tag: usize) -> Option<&'static str> 
    {
        self.offsets
            .get(tag)
            .and_then(|&offset| self.fields.get(offset))
            .map(|field| field.name())
    }

    pub fn name_of_value(&self, tag: usize, value: &str) -> Option<&'static str> 
    {
        self.offsets
            .get(tag)
            .and_then(|&offset| self.fields.get(offset))
            .and_then(|field| field.name_of_value(value))
    }

    pub fn field_with_name(&self, name: &str) -> Option<&Box<dyn OrchestrationField>>
    {
        let Some(offset) = self.offsets_by_name.get(name) else {
            return None;
        };    

        Some(&self.fields[*offset])
    }

}

unsafe impl Sync for OrchestrationFieldCollection {}
unsafe impl Send for OrchestrationFieldCollection {}

impl Index<usize> for OrchestrationFieldCollection {
    
    type Output = Box<dyn OrchestrationField>;

    // TODO - Consider a different trait like array_opts and return an Option
    fn index(&self, tag: usize) -> &Self::Output
    {
        let offset = self.offsets[tag];
        &self.fields[offset]
    }
  
}

#[derive(Copy, Clone, PartialEq)]
pub enum Presence {
    Required,
    Optional,
    Forbidden,
    Ignored,
    Constant
}

pub struct MessageField
{
    field: Box<dyn OrchestrationField>,
    field_presence: Presence,
    nesting_depth: u32,
}

impl MessageField {

    pub fn new(field: Box<dyn OrchestrationField>, field_presence: Presence, nesting_depth: u32) -> Self
    {
        Self { field, field_presence, nesting_depth }
    }

    pub fn tag(&self) -> u32 { self.field.tag() }
    pub fn name(&self) -> &str { self.field.name() }
    pub fn data_type(&self) -> &str { self.field.data_type() }
    pub fn synopsis(&self) -> &str { self.field.synopsis() }
    pub fn pedigree(&self) -> crate::dictionary::Pedigree { self.field.pedigree() }
    pub fn presence(&self) -> Presence { self.field_presence }
    // Nested groups are indicated using this field.
    pub fn depth(&self) -> u32 { self.nesting_depth }
}

unsafe impl Sync for MessageField {}
unsafe impl Send for MessageField {}

pub trait Message
{
    fn name(&self) -> &'static str;
    fn msg_type(&self) -> &'static str;
    fn category(&self) -> &'static str;
    fn synopsis(&self) -> &'static str;
    fn pedigree(&self) -> crate::dictionary::Pedigree {
        crate::dictionary::Pedigree {
            added: None,
            added_ep: None,
            updated: None,
            updated_ep: None,
            deprecated: None,
            deprecated_ep: None
        } 
    } 
    fn fields(&self) -> &'static Vec<crate::dictionary::MessageField>;
}

pub struct MessageCollection {

    messages: Vec<Box<dyn Message>>,
    messages_by_msg_type: HashMap<&'static str, usize>

}

impl MessageCollection {

    pub fn new(messages: Vec<Box<dyn Message>>) -> Self 
    {
        let messages_by_msg_type = messages
            .iter()
            .enumerate()
            .map(|(index, msg)| (msg.msg_type(), index))
            .collect();
        
        Self {
            messages,
            messages_by_msg_type,
        }
    }

    pub fn len(&self) -> usize {
        self.messages.len()
    }

    pub fn is_empty(&self) -> bool {
        self.messages.is_empty()
    }

    pub fn name_of_message(&self, msg_type: &str) -> Option<&'static str> 
    {
        self.messages_by_msg_type
            .get(msg_type)
            .and_then(|&index| self.messages.get(index))
            .map(|message| message.name())
   }
   
}

unsafe impl Sync for MessageCollection {}
unsafe impl Send for MessageCollection {}

impl Index<usize> for MessageCollection {
    
    type Output = Box<dyn Message>;

    // TODO - Consider a different trait like array_opts and return an Option
    fn index(&self, index: usize) -> &Self::Output
    {
        &self.messages[index]
    }
  
}

pub trait Orchestration
{
    fn name(&self) -> &'static str;
    fn fields(&self) -> &'static OrchestrationFieldCollection;
    fn messages(&self) -> &'static MessageCollection;
}