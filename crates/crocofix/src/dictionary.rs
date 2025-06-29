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

// pub trait MessageField
// {
//     fn tag(&self) -> u32;
//     fn name(&self) -> &str;
//     fn r#type(&self) -> &str;
//     fn synopsis(&self) -> &str;
//     fn pedigree(&self) -> crate::dictionary::Pedigree;
//     // constexpr dictionary::presence presence() const noexcept { return m_presence; }
//     // Nested groups are indicated using this field.
//     fn depth(&self) -> u32;
// }

// class message_field
// {
// public:

//     message_field(const orchestration_field& field, dictionary::presence presence, size_t depth)
//     : m_field(field),
//       m_presence(presence),
//       m_depth(depth)
//     {
//     }

//     constexpr int tag() const noexcept { return m_field.tag(); }
//     constexpr const std::string_view& name() const noexcept { return m_field.name(); }
//     constexpr const std::string_view& type() const noexcept { return m_field.type(); }
//     constexpr const std::string_view& synopsis() const noexcept { return m_field.synopsis(); }
//     constexpr const dictionary::pedigree& pedigree() const noexcept { return m_field.pedigree(); }
  
//     constexpr dictionary::presence presence() const noexcept { return m_presence; }
//     // Nested groups are indicated using this field.
//     constexpr size_t depth() const noexcept { return m_depth; }

// private:

//     orchestration_field m_field;
//     dictionary::presence m_presence;
//     size_t m_depth;
  
// };

// pub struct MessageFieldCollection {

//     fields: Vec<Box<dyn MessageField>>

// }

pub trait Message
{
    fn name(&self) -> &str;
    fn msg_type(&self) -> &str;
    fn category(&self) -> &str;
    fn synopsis(&self) -> &str;
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
    //     const crocofix::dictionary::message_field_collection& fields() const noexcept { return m_fields; }
}
