use core::fmt;
use crate::dictionary::FieldValue;

// TODO - consider encoding this with an enum
#[derive(fmt::Debug, PartialEq, Clone, Default)]
pub struct Field {
    pub tag: u32,
    pub value: String
}

impl Field {

    pub fn from_str(tag: u32, value: &str) -> Self
    {
        Self {
            tag,
            value: value.to_string()
        }
    }

    pub fn from_field_value(field: &FieldValue) -> Self
    {
        Self {
            tag: field.tag,
            value: field.value.to_string()
        }    
    }

}

impl fmt::Display for Field {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}={}", self.tag, self.value)
    }
}

impl PartialEq<crate::dictionary::FieldValue> for Field {
    fn eq(&self, other: &crate::dictionary::FieldValue) -> bool {
        self.tag == other.tag && self.value == other.value        
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    pub fn from_str()
    {
        let field = Field::from_str(crate::FIX_5_0SP2::ExDestination::TAG, "ASX");
        assert_eq!(field.tag, crate::FIX_5_0SP2::ExDestination::TAG);
        assert_eq!(field.value, "ASX".to_string());
    }

    #[test]
    pub fn from_field_value()
    {
        let field = Field::from_field_value(crate::FIX_5_0SP2::OrdStatus::PendingReplace());
        assert_eq!(field.tag, crate::FIX_5_0SP2::OrdStatus::TAG);
        assert_eq!(field.value, "E");

    }

}