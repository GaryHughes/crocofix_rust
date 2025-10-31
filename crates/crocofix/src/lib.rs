pub mod dictionary;
pub mod field;
pub mod message;
pub mod error;
pub mod order;
pub mod order_book;
pub mod order_report;
pub mod field_collection;

include!(concat!(env!("OUT_DIR"), "/FIX_4_2.rs"));
include!(concat!(env!("OUT_DIR"), "/FIX_4_4.rs"));
include!(concat!(env!("OUT_DIR"), "/FIX_5_0SP2.rs"));

#[cfg(test)]
mod tests {
    use super::*;

    use dictionary::Message;
    use dictionary::Orchestration;
    use dictionary::Presence;

    #[test]
    fn value_definitions_4_2() {
        let buy = FIX_4_2::Side::Buy();
        assert_eq!(buy.tag, 54);
        assert_eq!(buy.name, "Buy");
        assert_eq!(buy.value, "1");

        let sell = FIX_4_2::Side::Sell();
        assert_eq!(sell.tag, 54);
        assert_eq!(sell.name, "Sell");
        assert_eq!(sell.value, "2")
    }

    #[test]
    fn value_definitions_4_4() {
        let buy = FIX_4_4::Side::Buy();
        assert_eq!(buy.tag, 54);
        assert_eq!(buy.name, "Buy");
        assert_eq!(buy.value, "1");

        let sell = FIX_4_4::Side::Sell();
        assert_eq!(sell.tag, 54);
        assert_eq!(sell.name, "Sell");
        assert_eq!(sell.value, "2")
    }

    #[test]
    fn value_definitions_5_0_sp2() {
        let buy = FIX_5_0SP2::Side::Buy();
        assert_eq!(buy.tag, 54);
        assert_eq!(buy.name, "Buy");
        assert_eq!(buy.value, "1");

        let sell = FIX_5_0SP2::Side::Sell();
        assert_eq!(sell.tag, 54);
        assert_eq!(sell.name, "Sell");
        assert_eq!(sell.value, "2")
    }

    #[test]
    fn version_invalid_field_definition() {
        let invalid = &FIX_4_2::fields()[0];
        assert!(invalid.is_valid() == false);
        assert_eq!(invalid.tag(), 0);
        assert_eq!(invalid.name(), "");
        assert_eq!(invalid.synopsis(), "");
        // pedigree
        assert_eq!(invalid.values().len(), 0);
    }

    #[test]
    fn version_valid_field_definition() {
        let valid = &FIX_4_2::fields()[54];
        assert!(valid.is_valid() == true);
        assert_eq!(valid.tag(), 54);
        assert_eq!(valid.name(), "Side");
        assert_eq!(valid.synopsis(), "Side of order");
        // pedigree
        assert!(valid.values().len() > 0);
    }

    #[test]
    fn lookup_field_name() {
        assert_eq!(FIX_4_4::fields().name_of_field(100), Some("ExDestination"));
        assert_eq!(FIX_4_4::fields().name_of_field(999999), None);
    }

    #[test]
    fn lookup_field_value() {
        assert_eq!(FIX_4_4::fields().name_of_value(18, "G"), Some("AllOrNone"));
        assert_eq!(FIX_4_4::fields().name_of_value(999999, "1"), None);
        assert_eq!(FIX_4_4::fields().name_of_value(999999, "54"), None);

        assert_eq!(FIX_5_0SP2::fields().name_of_value(18, "G"), Some("AllOrNone"));
        assert_eq!(FIX_5_0SP2::fields().name_of_value(999999, "1"), None);
        assert_eq!(FIX_5_0SP2::fields().name_of_value(999999, "54"), None);
    }

    #[test]
    #[should_panic]
    fn tag_too_high_fails() {
        let _ = &FIX_4_4::fields()[1000];
    }

    // How do we idiomatically model overloaded index methods?

    // #[test]
    // fn lookup_field_by_name() {
    //     assert!(&FIX_5_0SP2::fields()["ExDestination"].tag() == 100);
    // }

    // #[test]
    // #[should_panic]
    // fn lookup_invalid_name_fails() {

    //     let _ = &FIX_5_0SP2::fields()["MadeUp"];
    // }

    #[test]
    fn message_definition() {
        let order_single = FIX_4_4::message::NewOrderSingle{};
        assert_eq!(order_single.name(), "NewOrderSingle");
        assert_eq!(order_single.msg_type(), "D");
        assert_eq!(order_single.category(), "SingleGeneralOrderHandling");
        assert_eq!(order_single.synopsis(), "The new order message type is used by institutions wishing to electronically submit securities and forex orders to a broker for execution.");
        let pedigree = order_single.pedigree();
        assert_eq!(pedigree.added, Some("FIX.2.7"));
        assert_eq!(pedigree.added_ep, None);
        assert_eq!(pedigree.updated, None);
        assert_eq!(pedigree.updated_ep, None);
        assert_eq!(pedigree.deprecated, None);
        assert_eq!(pedigree.deprecated_ep, None);
    }

    #[test]
    fn orchestration_message_definitions() {
        let orchestration = FIX_4_4::orchestration();
        assert_eq!(orchestration.messages().len(), 93);
        let heartbeat = &orchestration.messages()[0];
        assert_eq!(heartbeat.name(), "Heartbeat");
        assert_eq!(heartbeat.msg_type(), "0");
        assert_eq!(heartbeat.category(), "Session");
        assert_eq!(heartbeat.pedigree().added, Some("FIX.2.7"));
        assert_eq!(heartbeat.synopsis(), "The Heartbeat monitors the status of the communication link and identifies when the last of a string of messages was not received.");
    }

    #[test]
    #[should_panic]
    fn orchestration_msg_type_lookup_with_invalid_msg_type() {
        let _ = &FIX_4_4::messages()[200];
    }

    #[test]
    fn orchestration_message_fields() {
        let orchestration = FIX_4_4::orchestration();
        let heartbeat = &orchestration.messages()[0];
        assert_eq!(heartbeat.fields().len(), 34);
        let begin_string = &heartbeat.fields()[0];
        assert_eq!(begin_string.tag(), 8);
        assert_eq!(begin_string.name(), "BeginString".to_string());
        assert_eq!(begin_string.data_type(), "String".to_string());
        assert_eq!(begin_string.synopsis(), "Identifies beginning of new message and protocol version. ALWAYS FIRST FIELD IN MESSAGE. (Always unencrypted)".to_string());
        assert_eq!(begin_string.depth(), 0);
        assert!(begin_string.presence() == Presence::Required);
        let pedigree = begin_string.pedigree();
        assert_eq!(pedigree.added, Some("FIX.2.7"));
        assert_eq!(pedigree.added_ep, None);
        assert_eq!(pedigree.updated, None);
        assert_eq!(pedigree.updated_ep, None);
        assert_eq!(pedigree.deprecated, None);
        assert_eq!(pedigree.deprecated_ep, None);
    }

    #[test]
    fn orchestration_lookup_message_name() {
        let orchestration = FIX_4_4::orchestration();
        let messages = orchestration.messages();
        assert_eq!(messages.name_of_message("A"), Some("Logon"));
        assert_eq!(messages.name_of_message("ZZZZ"), None);
    }

    #[test]
    fn orchestration_version_field_definitions() {
        let orchestration = FIX_4_4::orchestration();
        let fields = orchestration.fields();
        assert!(fields[0].is_valid() == false);
        assert_eq!(fields.len(), 913);
        assert!(fields[54].is_valid());
        assert_eq!(fields[54].tag(), 54);
    }

    #[test]
    fn orchestration_lookup_field_name() {
        let orchestration = FIX_4_4::orchestration();
        let fields = orchestration.fields();
        assert_eq!(fields.name_of_field(100), Some("ExDestination"));
        assert_eq!(fields.name_of_field(999999), None);
    }

    #[test]
    fn orchestration_lookup_field_value() {
        let orchestration = FIX_4_4::orchestration();
        let fields = orchestration.fields();
        assert_eq!(fields.name_of_value(18, "G"), Some("AllOrNone"));
        assert_eq!(fields.name_of_value(999999, "1"), None);
        assert_eq!(fields.name_of_value(999999, "54"), None);
    }

    #[test]
    fn version_field_debug() {
        let orchestration = FIX_4_4::orchestration();
        let field = &orchestration.fields()[1];
        let debug = format!("{:?}", field);
        assert_eq!("OrchestrationField { name: \"Account\", tag: 1 }", debug);
    }

    #[test]
    fn is_tag_valid() {
        let orchestration = FIX_5_0SP2::orchestration();
        assert!(orchestration.fields().is_tag_valid(0) == false);
        assert!(orchestration.fields().is_tag_valid(1));
    }

}

