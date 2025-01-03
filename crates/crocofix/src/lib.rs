pub mod dictionary;

#[allow(non_snake_case)]

pub mod fix_4_2 {

pub struct Side {
}

impl Side {
    
    #[allow(non_snake_case)]
    pub fn Buy() -> &'static crate::dictionary::FieldValue {
        static VALUE: crate::dictionary::FieldValue = crate::dictionary::FieldValue { tag: 54 , name: "Buy", value: "1" };
        &VALUE
    }

    #[allow(non_snake_case)]
    pub fn Sell() -> &'static crate::dictionary::FieldValue {
        static VALUE: crate::dictionary::FieldValue = crate::dictionary::FieldValue { tag: 54, name: "Sell", value: "2" };
        &VALUE
    }

}

impl crate::dictionary::VersionField for Side {

    fn tag(&self) -> u32 { 54 }
    fn name(&self) -> &str { "Side" }
    fn data_type(&self) -> &str { "SideCodeSet" }
    fn description(&self) -> &str { "Side of order" }
    
    fn pedigree(&self) -> crate::dictionary::Pedigree {
        crate::dictionary::Pedigree {
            added: Some("FIX.2.7".to_string()),
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
            vec![
                Side::Buy(), 
                Side::Sell()
            ]
        })
    }

}

pub fn fields() -> &'static Vec<Box<dyn crate::dictionary::VersionField + Send + Sync>> {
    static FIELDS: std::sync::OnceLock<Vec<Box<dyn crate::dictionary::VersionField + Send + Sync>>> = std::sync::OnceLock::new();
    FIELDS.get_or_init(|| {
        vec![
            Box::new(crate::dictionary::InvalidField{}),
            Box::new(Side{})
        ]
    })
}

}



#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn value_definitions() {
        let buy = fix_4_2::Side::Buy();
        assert_eq!(buy.tag, 54);
        assert_eq!(buy.name, "Buy");
        assert_eq!(buy.value, "1");

        let sell = fix_4_2::Side::Sell();
        assert_eq!(sell.tag, 54);
        assert_eq!(sell.name, "Sell");
        assert_eq!(sell.value, "2")
    }

    #[test]
    fn version_invalid_field_definition() {
        let invalid = &fix_4_2::fields()[0];
        assert!(invalid.is_valid() == false);
        assert_eq!(invalid.tag(), 0);
        assert_eq!(invalid.name(), "");
        assert_eq!(invalid.description(), "");
        // pedigree
        assert_eq!(invalid.values().len(), 0);
    }

    #[test]
    fn version_valid_field_definition() {
        let valid = &fix_4_2::fields()[1];
        assert!(valid.is_valid() == true);
        assert_eq!(valid.tag(), 54);
        assert_eq!(valid.name(), "Side");
        assert_eq!(valid.description(), "Side of order");
        // pedigree
        assert!(valid.values().len() > 0);
    }

    #[test]
    fn lookup_field_name() {

        //     REQUIRE(crocofix::FIX_4_4::fields().name_of_field(100) == "ExDestination");
        //     REQUIRE(crocofix::FIX_4_4::fields().name_of_field(999999).empty());

    }

    #[test]
    fn lookup_field_value() {

        //     REQUIRE(crocofix::FIX_4_4::fields().name_of_value(18, "G") == "AllOrNone");
//     REQUIRE(crocofix::FIX_4_4::fields().name_of_value(999999, "1").empty());
//     REQUIRE(crocofix::FIX_4_4::fields().name_of_value(999999, "54").empty());

    }

    #[test]
    fn field_name() {

        //     REQUIRE(crocofix::FIX_5_0SP2::fields().name_of_value(18, "G") == "AllOrNone");
//     REQUIRE(crocofix::FIX_5_0SP2::fields().name_of_value(999999, "1").empty());
//     REQUIRE(crocofix::FIX_5_0SP2::fields().name_of_value(999999, "54").empty());

//     REQUIRE(crocofix::FIX_5_0SP2::fields()[1].name() == "Account");

    }

    #[test]
    fn tag_too_high_fails() {

        //     REQUIRE_THROWS_AS(crocofix::FIX_4_4::fields()[1000], std::out_of_range);

    }

    #[test]
    fn lookup_field_by_name() {

        //     REQUIRE(crocofix::FIX_5_0SP2::fields()["ExDestination"].tag() == 100);

    }

    #[test]
    fn lookup_invalid_name_fails() {

        //     REQUIRE_THROWS_AS(crocofix::FIX_5_0SP2::fields()["MadeUp"], std::out_of_range);

    }

}

