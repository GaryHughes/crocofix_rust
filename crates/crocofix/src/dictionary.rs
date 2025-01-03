
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