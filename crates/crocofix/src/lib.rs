use dictionary::Message;

pub mod dictionary;

include!(concat!(env!("OUT_DIR"), "/FIX_4_2.rs"));
include!(concat!(env!("OUT_DIR"), "/FIX_4_4.rs"));
include!(concat!(env!("OUT_DIR"), "/FIX_5_0SP2.rs"));

#[cfg(test)]
mod tests {
    use super::*;

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
        assert_eq!(invalid.description(), "");
        // pedigree
        assert_eq!(invalid.values().len(), 0);
    }

    #[test]
    fn version_valid_field_definition() {
        let valid = &FIX_4_2::fields()[54];
        assert!(valid.is_valid() == true);
        assert_eq!(valid.tag(), 54);
        assert_eq!(valid.name(), "Side");
        assert_eq!(valid.description(), "Side of order");
        // pedigree
        assert!(valid.values().len() > 0);
    }

    #[test]
    fn lookup_field_name() {
        assert!(&FIX_4_4::fields().name_of_field(100) == "ExDestination");
        assert!(&FIX_4_4::fields().name_of_field(999999).is_empty());
    }

    #[test]
    fn lookup_field_value() {
        assert!(&FIX_4_4::fields().name_of_value(18, "G") == "AllOrNone");
        assert!(&FIX_4_4::fields().name_of_value(999999, "1").is_empty());
        assert!(&FIX_4_4::fields().name_of_value(999999, "54").is_empty());

        assert!(&FIX_5_0SP2::fields().name_of_value(18, "G") == "AllOrNone");
        assert!(&FIX_5_0SP2::fields().name_of_value(999999, "1").is_empty());
        assert!(&FIX_5_0SP2::fields().name_of_value(999999, "54").is_empty());
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
        assert!(order_single.name() == "NewOrderSingle");
        assert!(order_single.msg_type() == "D");
        assert!(order_single.category() == "SingleGeneralOrderHandling");
        assert!(order_single.synopsis() == "The new order message type is used by institutions wishing to electronically submit securities and forex orders to a broker for execution.");
        let pedigree = order_single.pedigree();
        assert!(pedigree.added == Some("FIX.2.7".to_string()));
        assert!(pedigree.added_ep == None);
        assert!(pedigree.updated == None);
        assert!(pedigree.updated_ep == None);
        assert!(pedigree.deprecated == None);
        assert!(pedigree.deprecated_ep == None);
    }

    #[test]
    fn orchestration_message_definitions() {
        // auto orchestration = crocofix::FIX_4_4::orchestration();

        // REQUIRE(orchestration.messages().size() == 93);

        // auto heartbeat = orchestration.messages()[0];

        // REQUIRE(heartbeat.name() == "Heartbeat");
        // REQUIRE(heartbeat.msg_type() == "0");
        // REQUIRE(heartbeat.category() == "Session");
        // REQUIRE(heartbeat.pedigree().added() == "FIX.2.7");        
        // REQUIRE(heartbeat.synopsis() == "The Heartbeat monitors the status of the communication link and identifies when the last of a string of messages was not received.");
    }

    #[test]
    fn orchestration_msg_type_lookup_with_valid_msg_type() {
        // auto orchestration = crocofix::FIX_4_4::orchestration();
        // const auto& executionReport = orchestration.messages()["8"];
        // REQUIRE(executionReport.name() == "ExecutionReport");
    }

    #[test]
    #[should_panic]
    fn orchestration_msg_type_lookup_with_invalid_msg_type() {
        let _ = &FIX_4_4::messages()[200];
    }

    #[test]
    fn orchestration_message_fields() {
        // auto orchestration = crocofix::FIX_4_4::orchestration();
        // auto heartbeat = orchestration.messages()[0];
        // REQUIRE(heartbeat.fields().size() == 34);
    }

    #[test]
    fn orchestration_lookup_message_name() {
        // auto orchestration = crocofix::FIX_4_4::orchestration();
        // REQUIRE(orchestration.messages().name_of_message("A") == "Logon");
        // REQUIRE(orchestration.messages().name_of_message("ZZZZ").empty());
    }

    #[test]
    fn orchestration_version_field_definitions() {
        // auto orchestration = crocofix::FIX_4_4::orchestration();
        // REQUIRE(orchestration.fields()[0].is_valid() == false);
        // REQUIRE(orchestration.fields().size() == 913);
        // REQUIRE(orchestration.fields()[54].is_valid() == true);
        // REQUIRE(orchestration.fields()[54].tag() == 54);
    }

    #[test]
    fn orchestration_lookup_field_name() {
        // auto orchestration = crocofix::FIX_4_4::orchestration();
        // REQUIRE(orchestration.fields().name_of_field(100) == "ExDestination");
        // REQUIRE(orchestration.fields().name_of_field(999999).empty());
    }

    #[test]
    fn orchestration_lookup_field_value() {
        // auto orchestration = crocofix::FIX_4_4::orchestration();
        // REQUIRE(orchestration.fields().name_of_value(18, "G") == "AllOrNone");
        // REQUIRE(orchestration.fields().name_of_value(999999, "1").empty());
        // REQUIRE(orchestration.fields().name_of_value(999999, "54").empty());
    }

}

