use std::ops::Index;

pub struct Pedigree {
    pub added: Option<String>,
    pub added_ep: Option<String>,
    pub updated: Option<String>,
    pub updated_ep: Option<String>,
    pub deprecated: Option<String>,
    pub deprecated_ep: Option<String>
}

pub struct FieldValue {
    pub tag: u32,
    pub name: &'static str,
    pub value: &'static str
}

pub trait VersionField {
    fn tag(&self) -> u32;
    fn name(&self) -> &str;
    fn data_type(&self) -> &str;
    fn description(&self) -> &str;
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
    fn name_of_value(&self, value: &str) -> String
    {
        return match self.values().into_iter().find(|&item| item.value == value) {
            None => "".to_string(),
            Some(entry) => entry.name.to_string()
        }
    }

}

pub struct InvalidField {
}

impl crate::dictionary::VersionField for InvalidField {

    fn tag(&self) -> u32 { 0 }
    fn name(&self) -> &str { "" }
    fn data_type(&self) -> &str { "" }
    fn description(&self) -> &str { "" }
    
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

pub struct VersionFieldCollection {

    offsets: Vec<usize>,
    fields: Vec<Box<dyn VersionField>>

}

impl VersionFieldCollection {

    pub fn new(offsets: Vec<usize>, fields: Vec<Box<dyn VersionField>>) -> Self 
    {
        Self { offsets, fields }
    }

    pub fn name_of_field(&self, tag: usize) -> String 
    {
        if tag < self.offsets.len() {
            let offset = self.offsets[tag];
            return self.fields[offset].name().to_string()
        }

        return String::new()
    }

    pub fn name_of_value(&self, tag: usize, value: &str) -> String 
    {
        if tag < self.offsets.len() {
            let offset = self.offsets[tag];
            return self.fields[offset].name_of_value(value).to_string()
        }

        return String::new()
    }

}

unsafe impl Sync for VersionFieldCollection {}
unsafe impl Send for VersionFieldCollection {}

impl Index<usize> for VersionFieldCollection {
    
    type Output = Box<dyn VersionField>;

    // TODO - Consider a different trait like array_opts and return an Option
    fn index(&self, tag: usize) -> &Self::Output
    {
        let offset = self.offsets[tag];
        &self.fields[offset]
    }
  
}


