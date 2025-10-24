use crate::field::Field;
use crate::error::Error;
use std::ops::{Deref, DerefMut};
use std::ops::Index;

#[derive(PartialEq)]
pub enum SetOperation
{
    // Replace the first occurrence of a field with this tag. 
    // If there is no field with this tag do nothing.
    ReplaceFirst,
    // Replace the first occurrence of a field with this tag. 
    // If there is no field with this tag add a field with this tag and value.
    ReplaceFirstOrAppend,
    // Append a new field with this tag and value. 
    Append
}

#[derive(PartialEq)]
pub enum RemoveOperation
{
    // Remove the first occurrence of a field with this tag.
    // If there is not field with this tag do nothing.
    RemoveFirst,
    // Remove all occurrences of a field with this tag.
    // If there is not field with this tag do nothing.
    RemoveAll
}

#[derive(Default, Clone)]
pub struct FieldCollection {

    fields: Vec<Field>

}

impl FieldCollection {

    pub fn clear(&mut self)
    {
        self.fields.clear();
    }

    pub fn is_empty(&self) -> bool 
    {
        self.fields.is_empty()
    }

    pub fn push(&mut self, field: Field) {
        self.fields.push(field);
    }

    pub fn get(&self, tag: u32) -> Result<&Field, Error>
    {
        if let Some(field) = self.try_get(tag) {
            Ok(field)
        }
        else {
            Err(Error::MessageDoesNotContainFieldWithTag(tag))
        }
    }   

    pub fn try_get(&self, tag: u32) -> Option<&Field>
    {
        if let Some(field) = self.fields.iter().find(|field| field.tag == tag) {
            return Some(field);
        }

        None
    }

    // The set methods return true if one or more fields were added/updated, false if not.
    pub fn set(&mut self, field: &Field, operation: SetOperation) -> bool
    {
        if operation == SetOperation::Append {
            self.fields.push(field.clone());
            return true;
        }

        match self.fields.iter_mut().find(|existing| existing.tag == field.tag) {
            Some(existing) => {
                existing.value = field.value.clone();
                true
            }
            None => {
                if operation == SetOperation::ReplaceFirstOrAppend {
                    self.fields.push(field.clone());
                    true
                }
                else {
                    false
                }
            }
        }
    }

    // The remove methods return true if one or more fields were removed, false if not.
    pub fn remove(&mut self, tag: u32, operation: RemoveOperation) -> bool
    {
        match operation {
            RemoveOperation::RemoveFirst => {
                if let Some(index) = self.fields.iter().position(|field| field.tag == tag) {
                    self.fields.remove(index);
                    true
                }
                else {
                    false
                }
            }
            RemoveOperation::RemoveAll => {
                let any = self.fields.iter().any(|field| field.tag == tag);
                self.fields.retain(|field| field.tag != tag);
                any
            }
        }
    }

}

// TODO - study these traits.
impl<'a> IntoIterator for &'a FieldCollection {

    type Item = &'a Field;
    type IntoIter = std::slice::Iter<'a, Field>;
    
    fn into_iter(self) -> Self::IntoIter {
        self.fields.iter()
    }
}

impl<'a> IntoIterator for &'a mut FieldCollection {

    type Item = &'a mut Field;
    type IntoIter = std::slice::IterMut<'a, Field>;
    
    fn into_iter(self) -> Self::IntoIter {
        self.fields.iter_mut()
    }
}

impl IntoIterator for FieldCollection {

    type Item = Field;
    type IntoIter = std::vec::IntoIter<Field>;
    
    fn into_iter(self) -> Self::IntoIter {
        self.fields.into_iter()
    }
}

impl Deref for FieldCollection {
    type Target = [Field];
    
    fn deref(&self) -> &Self::Target {
        &self.fields
    }
}

impl DerefMut for FieldCollection {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.fields
    }
}

impl Index<usize> for FieldCollection {
    type Output = Field;
    
    fn index(&self, index: usize) -> &Self::Output {
        &self.fields[index]
    }
}


#[cfg(test)]
mod tests {

    use super::*;
 
    #[test]
    pub fn default_state()
    {
        let fields = FieldCollection::default();
        assert!(fields.is_empty());
    }

    #[test]
    pub fn clear()
    {
        let mut fields = FieldCollection::default();
        assert!(fields.is_empty());
        assert_eq!(fields.set(&Field::from_field_value(crate::FIX_4_2::OrdStatus::New()), SetOperation::Append), true);
        assert!(!fields.is_empty());
        fields.clear();
        assert!(fields.is_empty());
    }

    #[test]
    pub fn overwrite_non_existent_field()
    {
        let mut fields = FieldCollection::default();
        assert_eq!(fields.set(&Field::from_str(crate::FIX_5_0SP2::ExDestination::TAG, "ASX"), SetOperation::ReplaceFirst), false);
        assert!(fields.is_empty());
    }

    #[test]
    pub fn overwrite_existing_field() -> Result<(), crate::error::Error>
    {
        let mut fields = FieldCollection::default();
        assert_eq!(fields.set(&Field::from_field_value(crate::FIX_4_2::OrdStatus::New()), SetOperation::Append), true);
        assert_eq!(fields.get(crate::FIX_5_0SP2::OrdStatus::TAG)?.value, "0");
        assert_eq!(fields.set(&Field::from_field_value(crate::FIX_4_2::OrdStatus::PartiallyFilled()), SetOperation::ReplaceFirstOrAppend), true);
        assert_eq!(fields.get(crate::FIX_5_0SP2::OrdStatus::TAG)?.value, "1");
        Ok(())
    }

    #[test]
    pub fn add_non_existent_field()
    {
        let mut fields = FieldCollection::default();
        assert_eq!(fields.set(&Field::from_str(crate::FIX_5_0SP2::ExDestination::TAG, "ASX"), SetOperation::ReplaceFirstOrAppend), true);
        assert_eq!(fields.len(), 1);
        let field = &fields[0];
        assert_eq!(field.tag, crate::FIX_5_0SP2::ExDestination::TAG);
        assert_eq!(field.value, "ASX");
    }

    #[test]
    pub fn add_duplicate_field()
    {
        let mut fields = FieldCollection::default();
        assert_eq!(fields.set(&Field::from_str(crate::FIX_5_0SP2::ExDestination::TAG, "ASX"), SetOperation::ReplaceFirstOrAppend), true);
        assert_eq!(fields.set(&Field::from_str(crate::FIX_5_0SP2::ExDestination::TAG, "ASX"), SetOperation::Append), true);
        assert_eq!(fields.len(), 2);
        let field = &fields[0];
        assert_eq!(field.tag, crate::FIX_5_0SP2::ExDestination::TAG);
        assert_eq!(field.value, "ASX");
        let field = &fields[1];
        assert_eq!(field.tag, crate::FIX_5_0SP2::ExDestination::TAG);
        assert_eq!(field.value, "ASX");
    }

    #[test]
    pub fn remove_first_non_existent_field_from_empty_collection()
    {
        let mut fields = FieldCollection::default();
        assert_eq!(fields.remove(crate::FIX_5_0SP2::ExDestination::TAG, RemoveOperation::RemoveFirst), false);
    }

    #[test]
    pub fn remove_all_non_existent_field_from_empty_collection()
    {
        let mut fields = FieldCollection::default();
        assert_eq!(fields.remove(crate::FIX_5_0SP2::ExDestination::TAG, RemoveOperation::RemoveAll), false);
    }

    #[test]
    pub fn remove_first_existent_field_from_populated_collection()
    {
        let mut fields = FieldCollection::default();
        assert_eq!(fields.set(&Field::from_str(crate::FIX_5_0SP2::ExDestination::TAG, "ASX"), SetOperation::Append), true);
        assert_eq!(fields.set(&Field::from_str(crate::FIX_5_0SP2::ExDestination::TAG, "ASX"), SetOperation::Append), true);
        assert_eq!(fields.remove(crate::FIX_5_0SP2::ExDestination::TAG, RemoveOperation::RemoveFirst), true);
        assert_eq!(fields.len(), 1);
    }

    #[test]
    pub fn remove_all_existent_field_from_populated_collection()
    {
        let mut fields = FieldCollection::default();
        assert_eq!(fields.set(&Field::from_str(crate::FIX_5_0SP2::ExDestination::TAG, "ASX"), SetOperation::Append), true);
        assert_eq!(fields.set(&Field::from_str(crate::FIX_5_0SP2::ExDestination::TAG, "ASX"), SetOperation::Append), true);
        assert_eq!(fields.set(&Field::from_str(crate::FIX_5_0SP2::ExDestination::TAG, "ASX"), SetOperation::Append), true);
        assert_eq!(fields.remove(crate::FIX_5_0SP2::ExDestination::TAG, RemoveOperation::RemoveAll), true);
        assert!(fields.is_empty());
    }

    #[test]
    pub fn get_non_existent_field_from_empty_collection()
    {
        let fields = FieldCollection::default();
        assert_eq!(fields.get(crate::FIX_5_0SP2::TimeInForce::TAG), Err(Error::MessageDoesNotContainFieldWithTag(crate::FIX_5_0SP2::TimeInForce::TAG)));
    }

    #[test]
    pub fn get_non_existent_field_from_populated_collection()
    {
        let mut fields = FieldCollection::default();
        assert_eq!(fields.set(&Field::from_str(crate::FIX_5_0SP2::ExDestination::TAG, "ASX"), SetOperation::Append), true);
        assert_eq!(fields.get(crate::FIX_5_0SP2::TimeInForce::TAG), Err(Error::MessageDoesNotContainFieldWithTag(crate::FIX_5_0SP2::TimeInForce::TAG)));
    }

    #[test]
    pub fn get_existent_field() -> Result<(), crate::error::Error>
    {
        let mut fields = FieldCollection::default();
        assert_eq!(fields.set(&Field::from_str(crate::FIX_5_0SP2::ExDestination::TAG, "ASX"), SetOperation::Append), true);
        assert_eq!(fields.get(crate::FIX_5_0SP2::ExDestination::TAG)?.value, "ASX");
        Ok(())
    }

    #[test]
    pub fn try_get_field_from_empty_collection()
    {
        let fields = FieldCollection::default();
        assert_eq!(fields.try_get(crate::FIX_5_0SP2::TimeInForce::TAG), None);
    }

    #[test]
    pub fn try_get_non_existent_field_from_populated_collection()
    {
        let mut fields = FieldCollection::default();
        assert_eq!(fields.set(&Field::from_str(crate::FIX_5_0SP2::ExDestination::TAG, "ASX"), SetOperation::Append), true);
        assert_eq!(fields.try_get(crate::FIX_5_0SP2::TimeInForce::TAG), None);
    }

    #[test]
    pub fn try_get_existent_field()
    {
        let mut fields = FieldCollection::default();
        assert_eq!(fields.set(&Field::from_str(crate::FIX_5_0SP2::ExDestination::TAG, "ASX"), SetOperation::Append), true);
        match fields.try_get(crate::FIX_5_0SP2::ExDestination::TAG) {
            None => panic!("field collection does not contain expected field {}", crate::FIX_5_0SP2::ExDestination::TAG),
            Some(field) => assert_eq!(field.value, "ASX")
        }
    }

    #[test]
    pub fn try_get_existent_field_returns_first_instance_of_multiply_defined_field()
    {
        let mut fields = FieldCollection::default();
        assert_eq!(fields.set(&Field::from_str(crate::FIX_5_0SP2::ExDestination::TAG, "ASX"), SetOperation::Append), true);
        assert_eq!(fields.set(&Field::from_str(crate::FIX_5_0SP2::ExDestination::TAG, "TSX"), SetOperation::Append), true);
        match fields.try_get(crate::FIX_5_0SP2::ExDestination::TAG) {
            None => panic!("field collection does not contain expected field {}", crate::FIX_5_0SP2::ExDestination::TAG),
            Some(field) => assert_eq!(field.value, "ASX")
        }
    }

}