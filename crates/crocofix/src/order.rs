use crate::message::Message;
use crate::field::Field;
use crate::field_collection::{FieldCollection, SetOperation};
use crate::error::Error;

#[derive(Default, Clone)]
pub struct Order {
    pub messages: Vec<Message>,
    pub fields: FieldCollection,
    pub pending_fields: FieldCollection,
    pub key: String,
    // Store the fields that comprise the order book id directly.
    pub begin_string: String,
    pub sender_comp_id: String,
    pub target_comp_id: String,
    pub cl_ord_id: Field,
    // This isn't a part of the id but it's important for tracking cancel replace chains so track it directly.
    pub orig_cl_ord_id: Option<Field>,
    // When we send an order cancel or order cancel replace request we cache the current status here
    // and set OrdStatus to Pending???. If we get a successful reply we blat this value, if we get
    // rejected we replace OrdStatus with this value.
    // TODO - can we make this Option<&Field>? we need lifetimes
    previous_ord_status: Option<Field>,
    // This is for replaced orders. 
    // 1. When we see the OrderCancelReplaceRequest we have ClOrdId=1 and OrigClOrdID=2  
    // 2. When we get the Pending ExecutionReport   we have ClOrdId=1 and OrigClOrdID=2
    // 3. When we get the Replaced ExecutionReport  we have ClOrdId=1 and OrigClOrdID=1
    // 4. At this point we want to set the previous order to Replaced and we want to clone it
    //    and give the new order ClOrdID=2. We walk back through the order list to find this
    //    ClOrdId in mNewClOrdId.
    // TODO - can we make this Option<&Field>? we need lifetimes
    new_cl_ord_id: Option<Field>
}

impl Order {
    
    pub fn new(message: &Message) -> Result<Self, Error> 
    {
        let mut order = Order { 
            key: Order::key_for_message(message, false)?, 
            fields: message.fields.clone(),
            begin_string: message.fields.get(crate::FIX_5_0SP2::BeginString::TAG)?.value.clone(),
            sender_comp_id: message.fields.get(crate::FIX_5_0SP2::SenderCompID::TAG)?.value.clone(),
            target_comp_id: message.fields.get(crate::FIX_5_0SP2::TargetCompID::TAG)?.value.clone(),
            cl_ord_id: message.fields.get(crate::FIX_5_0SP2::ClOrdID::TAG)?.clone(),
            orig_cl_ord_id: message.fields.try_get(crate::FIX_5_0SP2::OrigClOrdID::TAG).and_then(|field| Some(field.clone())),
            ..Default::default() 
        };
        order.messages.push(message.clone());
        Ok(order)
    }

    fn create_key(sender_comp_id: &str, target_comp_id: &str, cl_ord_id: &str) -> String
    {
        format!("{}-{}-{}", sender_comp_id, target_comp_id, cl_ord_id)
    }

    pub fn key_for_message(message: &Message, reverse_comp_ids: bool) -> Result<String, Error>
    {
        let sender_comp_id = message.fields.get(crate::FIX_5_0SP2::SenderCompID::TAG)?.value.as_str();
        let target_comp_id = message.fields.get(crate::FIX_5_0SP2::TargetCompID::TAG)?.value.as_str();

        let cl_ord_id = match message.fields.try_get(crate::FIX_5_0SP2::OrigClOrdID::TAG) {
            Some(field) => field.value.as_str(),
            None => {
                message.fields.get(crate::FIX_5_0SP2::ClOrdID::TAG)?.value.as_str()
            }
        };

        if reverse_comp_ids {
            Ok(Order::create_key(target_comp_id, sender_comp_id, cl_ord_id))
        }
        else {
            Ok(Order::create_key(sender_comp_id, target_comp_id, cl_ord_id))
        }
    }

    const IDENTITY_FIELDS: [u32; 5] = [
        crate::FIX_5_0SP2::BeginString::TAG,
        crate::FIX_5_0SP2::SenderCompID::TAG,
        crate::FIX_5_0SP2::TargetCompID::TAG,
        crate::FIX_5_0SP2::ClOrdID::TAG,
        crate::FIX_5_0SP2::OrigClOrdID::TAG
    ];

    pub fn is_identity_field(tag: u32) -> bool
    {
        Order::IDENTITY_FIELDS.contains(&tag)
    }
        
    fn update_pending_fields(&mut self, fields: &FieldCollection)
    {
        for field in fields {
            if !Order::is_identity_field(field.tag) {
                self.pending_fields.set(field, SetOperation::ReplaceFirstOrAppend);
            }
        }
    }

    fn update_fields(&mut self, fields: &FieldCollection)
    {
        for field in fields {
            if !Order::is_identity_field(field.tag) {
                self.fields.set(field, SetOperation::ReplaceFirstOrAppend);
            }
        }
    }

    pub fn update(&mut self, message: &Message) -> Result<(), Error>
    {
        self.messages.push(message.clone());

        let Some(msg_type) = message.msg_type() else {
            return Err(crate::error::Error::MessageDoesNotContainMsgType);    
        };

        if msg_type == crate::FIX_5_0SP2::MsgType::OrderCancelReplaceRequest().value {
            self.previous_ord_status = self.fields.try_get(crate::FIX_5_0SP2::OrdStatus::TAG).and_then(|field| Some(field.clone()));
            self.new_cl_ord_id = Some(message.fields.get(crate::FIX_5_0SP2::ClOrdID::TAG)?.clone());
            self.update_pending_fields(&message.fields);
            self.fields.set(&Field::from_field_value(crate::FIX_5_0SP2::OrdStatus::PendingReplace()), crate::field_collection::SetOperation::ReplaceFirst);
            return Ok(());
        }

        if msg_type == crate::FIX_5_0SP2::MsgType::OrderCancelRequest().value {
            self.previous_ord_status = self.fields.try_get(crate::FIX_5_0SP2::OrdStatus::TAG).and_then(|field| Some(field.clone()));
            self.update_pending_fields(&message.fields);
            self.fields.set(&Field::from_field_value(&crate::FIX_5_0SP2::OrdStatus::PendingCancel()), crate::field_collection::SetOperation::ReplaceFirst);
            return Ok(());
        }

        self.update_fields(&message.fields);
         
        Ok(())
    }

    pub fn rollback(&mut self)
    {
        self.pending_fields.clear();

        if let Some(previous_ord_status) = &self.previous_ord_status {
            self.fields.set(&previous_ord_status, SetOperation::ReplaceFirst);
            self.previous_ord_status.take();
        }
    }

    pub fn commit(&mut self)
    {
        // TODO - figure out the borrow issue here
        let pending_fields = self.pending_fields.clone();
        self.update_fields(&pending_fields);
        self.pending_fields.clear();
    }

    pub fn replace(&mut self, execution_report: &Message) -> Result<Order, Error>
    {
        let mut replacement = self.clone(); 
        replacement.update(&execution_report)?;
        replacement.commit();
        self.rollback();

        // TODO - document and add explicit tests for this
        if let Some(new_cl_ord_id) = &self.new_cl_ord_id {
            replacement.fields.set(new_cl_ord_id, SetOperation::ReplaceFirstOrAppend);
            replacement.fields.set(&Field::from_str(crate::FIX_5_0SP2::OrigClOrdID::TAG, replacement.cl_ord_id.value.as_str()), SetOperation::ReplaceFirstOrAppend);
            replacement.orig_cl_ord_id = Some(replacement.cl_ord_id.clone());
            replacement.cl_ord_id = new_cl_ord_id.clone();
        }
        else {
            if let Some(cl_ord_id) = execution_report.fields.try_get(crate::FIX_5_0SP2::ClOrdID::TAG) {
                replacement.fields.set(cl_ord_id, SetOperation::ReplaceFirstOrAppend);                
                replacement.fields.set(&Field::from_str(crate::FIX_5_0SP2::OrigClOrdID::TAG, replacement.cl_ord_id.value.as_str()), SetOperation::ReplaceFirstOrAppend);
                replacement.orig_cl_ord_id = Some(replacement.cl_ord_id.clone());
                replacement.cl_ord_id = self.cl_ord_id.clone();
            }
        }

        replacement.key = Order::create_key(replacement.sender_comp_id.as_str(), replacement.target_comp_id.as_str(), replacement.cl_ord_id.value.as_str());
        replacement.fields.set(&Field::from_field_value(crate::FIX_5_0SP2::OrdStatus::New()), SetOperation::ReplaceFirstOrAppend);
        self.fields.set(&Field::from_field_value(crate::FIX_5_0SP2::OrdStatus::Replaced()), SetOperation::ReplaceFirstOrAppend);

        self.messages.push(execution_report.clone());

        Ok(replacement)
    }

}

#[cfg(test)]
mod tests {

    use super::*;
 
    fn decode_message(text: &str) -> Result<Message, crate::error::Error>
    {
        let mut message = Message::default();
        let result = message.decode(text.as_bytes())?;
        assert!(result.complete);
        assert_eq!(result.consumed, text.len());
        Ok(message)
    }

    #[test]
    pub fn create_key()
    {
        assert_eq!("INITIATOR-ACCEPTOR-123", Order::create_key("INITIATOR", "ACCEPTOR", "123"))
    }

    #[test]

    pub fn key_for_message() -> Result<(), crate::error::Error>
    {
        let order_single = "8=FIX.4.4\u{0001}9=149\u{0001}35=D\u{0001}49=INITIATOR\u{0001}56=ACCEPTOR\u{0001}34=2752\u{0001}52=20200114-08:13:20.041\u{0001}11=61\u{0001}70=60\u{0001}100=AUTO\u{0001}55=BHP.AX\u{0001}54=1\u{0001}60=20200114-08:12:59.397\u{0001}38=10000\u{0001}40=2\u{0001}44=20\u{0001}59=1\u{0001}10=021\u{0001}";
        let message = decode_message(order_single)?;
        assert_eq!("INITIATOR-ACCEPTOR-61", Order::key_for_message(&message, false)?);
        assert_eq!("ACCEPTOR-INITIATOR-61", Order::key_for_message(&message, true)?);
        Ok(())
    }

    #[test]
    pub fn new_order_single() -> Result<(), crate::error::Error>
    {
        let order_single = "8=FIX.4.4\u{0001}9=149\u{0001}35=D\u{0001}49=INITIATOR\u{0001}56=ACCEPTOR\u{0001}34=2752\u{0001}52=20200114-08:13:20.041\u{0001}11=61\u{0001}70=60\u{0001}100=AUTO\u{0001}55=BHP.AX\u{0001}54=1\u{0001}60=20200114-08:12:59.397\u{0001}38=10000\u{0001}40=2\u{0001}44=20\u{0001}59=1\u{0001}10=021\u{0001}";

        let message = decode_message(order_single)?;
        let order = Order::new(&message)?;
        
        assert_eq!(order.fields.get(crate::FIX_5_0SP2::Side::TAG)?, crate::FIX_5_0SP2::Side::Buy()); 
        assert_eq!(order.fields.get(crate::FIX_5_0SP2::OrdType::TAG)?, crate::FIX_5_0SP2::OrdType::Limit()); 
        assert_eq!(order.fields.get(crate::FIX_5_0SP2::TimeInForce::TAG)?, crate::FIX_5_0SP2::TimeInForce::GoodTillCancel()); 
        assert_eq!(order.fields.get(crate::FIX_5_0SP2::OrderQty::TAG)?.value, "10000");
        assert_eq!(order.fields.get(crate::FIX_5_0SP2::Price::TAG)?.value, "20");
  
        Ok(())
    }

    #[test]
    pub fn new_order_single_and_execution_reports() -> Result<(), crate::error::Error>
    {
        let order_single = "8=FIX.4.4\u{0001}9=149\u{0001}35=D\u{0001}49=INITIATOR\u{0001}56=ACCEPTOR\u{0001}34=2752\u{0001}52=20200114-08:13:20.041\u{0001}11=61\u{0001}70=60\u{0001}100=AUTO\u{0001}55=BHP.AX\u{0001}54=1\u{0001}60=20200114-08:12:59.397\u{0001}38=10000\u{0001}40=2\u{0001}44=20\u{0001}59=1\u{0001}10=021\u{0001}";
        let report_new = "8=FIX.4.4\u{0001}9=173\u{0001}35=8\u{0001}49=ACCEPTOR\u{0001}56=INITIATOR\u{0001}34=718\u{0001}52=20200114-08:13:20.072\u{0001}39=0\u{0001}11=61\u{0001}37=INITIATOR-ACCEPTOR-61\u{0001}17=1\u{0001}150=0\u{0001}151=10000\u{0001}55=BHP.AX\u{0001}54=1\u{0001}38=10000\u{0001}44=20\u{0001}32=0\u{0001}31=0\u{0001}14=0\u{0001}6=0\u{0001}40=2\u{0001}10=021\u{0001}";
        let report_partial = "8=FIX.4.4\u{0001}9=187\u{0001}35=8\u{0001}49=ACCEPTOR\u{0001}56=INITIATOR\u{0001}34=719\u{0001}52=20200114-08:13:20.072\u{0001}39=1\u{0001}11=61\u{0001}37=INITIATOR-ACCEPTOR-61\u{0001}17=2\u{0001}150=1\u{0001}151=893\u{0001}55=BHP.AX\u{0001}54=1\u{0001}38=10000\u{0001}44=20\u{0001}32=9107\u{0001}31=20\u{0001}14=9107\u{0001}6=20\u{0001}30=AUTO\u{0001}40=2\u{0001}10=081\u{0001}";
        let report_filled = "8=FIX.4.4\u{0001}9=185\u{0001}35=8\u{0001}49=ACCEPTOR\u{0001}56=INITIATOR\u{0001}34=720\u{0001}52=20200114-08:13:20.072\u{0001}39=2\u{0001}11=61\u{0001}37=INITIATOR-ACCEPTOR-61\u{0001}17=3\u{0001}150=2\u{0001}151=0\u{0001}55=BHP.AX\u{0001}54=1\u{0001}38=10000\u{0001}44=20\u{0001}32=893\u{0001}31=20\u{0001}14=10000\u{0001}6=20\u{0001}30=AUTO\u{0001}40=2\u{0001}10=201\u{0001}";
    
        let mut order = Order::new(&decode_message(order_single)?)?;

        assert_eq!(order.fields.get(crate::FIX_5_0SP2::Side::TAG)?, crate::FIX_5_0SP2::Side::Buy());
        assert_eq!(order.fields.get(crate::FIX_5_0SP2::OrderQty::TAG)?.value, "10000");
        assert_eq!(order.fields.get(crate::FIX_5_0SP2::Price::TAG)?.value, "20");

        order.update(&decode_message(report_new)?)?;

        assert_eq!(order.fields.get(crate::FIX_5_0SP2::OrdStatus::TAG)?, crate::FIX_5_0SP2::OrdStatus::New());
        assert_eq!(order.fields.get(crate::FIX_5_0SP2::CumQty::TAG)?.value, "0");
        assert_eq!(order.fields.get(crate::FIX_5_0SP2::AvgPx::TAG)?.value, "0");
 
        order.update(&decode_message(report_partial)?)?;

        assert_eq!(order.fields.get(crate::FIX_5_0SP2::OrdStatus::TAG)?, crate::FIX_5_0SP2::OrdStatus::PartiallyFilled());
        assert_eq!(order.fields.get(crate::FIX_5_0SP2::CumQty::TAG)?.value, "9107");
        assert_eq!(order.fields.get(crate::FIX_5_0SP2::AvgPx::TAG)?.value, "20");

        order.update(&decode_message(report_filled)?)?;

        assert_eq!(order.fields.get(crate::FIX_5_0SP2::OrdStatus::TAG)?, crate::FIX_5_0SP2::OrdStatus::Filled());
        assert_eq!(order.fields.get(crate::FIX_5_0SP2::CumQty::TAG)?.value, "10000");
        assert_eq!(order.fields.get(crate::FIX_5_0SP2::AvgPx::TAG)?.value, "20");

        Ok(())
    }

    #[test]
    pub fn update_order_cancel_replace_request() -> Result<(), crate::error::Error>
    {
        let order_single = "8=FIXT.1.1\u{0001}9=147\u{0001}35=D\u{0001}49=INITIATOR\u{0001}56=ACCEPTOR\u{0001}34=2\u{0001}52=20200215-05:53:02.300\u{0001}11=7\u{0001}70=7\u{0001}100=AUTO\u{0001}55=WTF.AX\u{0001}54=1\u{0001}60=20200215-05:52:59.271\u{0001}38=20000\u{0001}40=2\u{0001}44=11.56\u{0001}59=1\u{0001}10=016\u{0001}";
        let report_new = "8=FIXT.1.1\u{0001}9=172\u{0001}35=8\u{0001}49=ACCEPTOR\u{0001}56=INITIATOR\u{0001}34=2\u{0001}52=20200215-05:53:02.473\u{0001}39=0\u{0001}11=7\u{0001}37=INITIATOR-ACCEPTOR-7\u{0001}17=1\u{0001}150=0\u{0001}151=20000\u{0001}55=WTF.AX\u{0001}54=1\u{0001}38=20000\u{0001}44=11.56\u{0001}32=0\u{0001}31=0\u{0001}14=0\u{0001}6=0\u{0001}40=2\u{0001}10=078\u{0001}";
        let order_cancel_replace_request = "8=FIXT.1.1\u{0001}9=184\u{0001}35=G\u{0001}49=INITIATOR\u{0001}56=ACCEPTOR\u{0001}34=3\u{0001}52=20200215-05:53:22.465\u{0001}37=INITIATOR-ACCEPTOR-7\u{0001}41=7\u{0001}11=8\u{0001}70=7\u{0001}100=AUTO\u{0001}55=WTF.AX\u{0001}54=1\u{0001}60=20200215-05:53:08.895\u{0001}38=40000\u{0001}40=2\u{0001}44=11.58\u{0001}59=1\u{0001}58=Blah\u{0001}10=104\u{0001}";
        let report_pending_replace = "8=FIXT.1.1\u{0001}9=177\u{0001}35=8\u{0001}49=ACCEPTOR\u{0001}56=INITIATOR\u{0001}34=3\u{0001}52=20200215-05:53:22.481\u{0001}39=E\u{0001}11=8\u{0001}37=INITIATOR-ACCEPTOR-7\u{0001}17=2\u{0001}150=E\u{0001}151=20000\u{0001}41=7\u{0001}55=WTF.AX\u{0001}54=1\u{0001}38=20000\u{0001}44=11.56\u{0001}32=0\u{0001}31=0\u{0001}14=0\u{0001}6=0\u{0001}40=2\u{0001}10=091\u{0001}";
        let report_replaced = "8=FIXT.1.1\u{0001}9=173\u{0001}35=8\u{0001}49=ACCEPTOR\u{0001}56=INITIATOR\u{0001}34=4\u{0001}52=20200215-05:53:22.495\u{0001}39=5\u{0001}11=7\u{0001}37=INITIATOR-ACCEPTOR-8\u{0001}17=3\u{0001}150=5\u{0001}151=0\u{0001}41=7\u{0001}55=WTF.AX\u{0001}54=1\u{0001}38=40000\u{0001}44=11.58\u{0001}32=0\u{0001}31=0\u{0001}14=0\u{0001}6=0\u{0001}40=2\u{0001}10=128\u{0001}";

        let mut order = Order::new(&decode_message(order_single)?)?;

        assert_eq!(order.fields.get(crate::FIX_5_0SP2::Side::TAG)?, crate::FIX_5_0SP2::Side::Buy());
        assert_eq!(order.fields.get(crate::FIX_5_0SP2::OrderQty::TAG)?.value, "20000");
        assert_eq!(order.fields.get(crate::FIX_5_0SP2::Price::TAG)?.value, "11.56");

        order.update(&decode_message(report_new)?)?;

        assert_eq!(order.fields.get(crate::FIX_5_0SP2::OrdStatus::TAG)?, crate::FIX_5_0SP2::OrdStatus::New());

        order.update(&decode_message(order_cancel_replace_request)?)?;

        assert_eq!(order.fields.get(crate::FIX_5_0SP2::OrdStatus::TAG)?, crate::FIX_5_0SP2::OrdStatus::PendingReplace());

        order.update(&decode_message(report_pending_replace)?)?;

        assert_eq!(order.fields.get(crate::FIX_5_0SP2::OrdStatus::TAG)?, crate::FIX_5_0SP2::OrdStatus::PendingReplace());

        order.update(&decode_message(report_replaced)?)?;

        assert_eq!(order.fields.get(crate::FIX_5_0SP2::OrdStatus::TAG)?, crate::FIX_5_0SP2::OrdStatus::Replaced());

        Ok(())
    }

    #[test]
    pub fn update_order_cancel_request()
    {

    }

    #[test]
    pub fn commit()
    {

    }

    #[test]
    pub fn rollback()
    {

    }

    #[test]
    pub fn replace()
    {

    }


}