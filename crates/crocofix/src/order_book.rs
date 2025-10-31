use indexmap::IndexMap;
use crate::message::Message;
use crate::order::Order;
use crate::error::Error;

#[derive(Default)]
pub struct OrderBook {
    // TODO - perhaps remove Clone from Order and Box
    pub orders: IndexMap<String, Order>
}

impl OrderBook {

    pub fn process(&mut self, message: &Message) -> Result<(), Error>
    {
        let Some(msg_type) = message.msg_type() else {
            return Err(Error::MessageDoesNotContainMsgType);
        };

        // TODO - make these comparisons more natural
        // TODO - can we change the dictionary so these are constants which will allow a match expression?
        if msg_type == crate::FIX_5_0SP2::MsgType::NewOrderSingle().value { 
            return self.process_order_single(message);
        }

        if msg_type == crate::FIX_5_0SP2::MsgType::ExecutionReport().value {
            return self.process_execution_report(message);
        }

        if msg_type == crate::FIX_5_0SP2::MsgType::OrderCancelRequest().value {
            return self.process_order_cancel_request(message);
        }

        if msg_type == crate::FIX_5_0SP2::MsgType::OrderCancelReplaceRequest().value {
            return self.process_order_cancel_replace_request(message);
        }

        if msg_type == crate::FIX_5_0SP2::MsgType::OrderCancelReject().value {
            return self.process_order_cancel_reject(message);
        }

        Err(Error::UnsupportedMsgType(msg_type.to_string()))
    }

    pub fn clear(&mut self)
    {
        self.orders.clear();
    }

    fn process_order_single(&mut self, order_single: &Message) -> Result<(), Error>
    {
        let order = Order::new(order_single)?;
        // TODO - understand and improve this
        let key = (&order).key.clone();
        if self.orders.contains_key(&key) {
            return Err(Error::OrderBookAlreadyContainsOrderWithKey(key));
        }
        self.orders.insert(key.clone(), order.clone());
        Ok(())
    }

    fn process_execution_report(&mut self, execution_report: &Message) -> Result<(), Error>
    {
        let key = Order::key_for_message(&execution_report, true)?;
        
        if !self.orders.contains_key(&key) {
            return Err(Error::OrderBookDoesNotContainOrderWithKey(key));
        }

        if let Some(exec_type) = execution_report.fields.try_get(crate::FIX_5_0SP2::ExecType::TAG) {
            if exec_type == crate::FIX_5_0SP2::ExecType::Replaced() {
                let Some(order) = self.orders.get_mut(&key) else {
                    return Err(Error::OrderBookDoesNotContainOrderWithKey(key));
                };
                let replacement = order.replace(execution_report)?;
                // TODO - understand and improve this
                let replacement_key = (&replacement).key.clone();
                if self.orders.contains_key(&replacement_key) {
                    return Err(Error::OrderBookAlreadyContainsOrderWithKey(replacement_key));
                }
                self.orders.insert(replacement_key.clone(), replacement.clone());
                return Ok(());
            }
        }

        let Some(order) = self.orders.get_mut(&key) else {
            return Err(Error::OrderBookDoesNotContainOrderWithKey(key));
        };

        order.update(&execution_report)
    }

    fn process_order_cancel_request(&mut self, order_cancel_request: &Message) -> Result<(), Error>
    {
        let key = Order::key_for_message(&order_cancel_request, false)?;
        
        let Some(order) = self.orders.get_mut(&key) else {
            return Err(Error::OrderBookDoesNotContainOrderWithKey(key));
        };
        
        order.update(&order_cancel_request)
    }

    fn process_order_cancel_replace_request(&mut self, order_cancel_replace_request: &Message) -> Result<(), Error>
    {
        let key = Order::key_for_message(&order_cancel_replace_request, false)?;

        let Some(order) = self.orders.get_mut(&key) else {
            return Err(Error::OrderBookDoesNotContainOrderWithKey(key));
        };

        order.update(&order_cancel_replace_request)
    }

    fn process_order_cancel_reject(&mut self, order_cancel_reject: &Message) -> Result<(), Error>
    {
        let key = Order::key_for_message(&order_cancel_reject, true)?;
        
        let Some(order) = self.orders.get_mut(&key) else {
            return Err(Error::OrderBookDoesNotContainOrderWithKey(key));
        };

        order.rollback();

        Ok(())
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
    pub fn default_book() 
    {
        let book = OrderBook::default();
        assert!(book.orders.is_empty());
    }

    #[test]
    pub fn unknown_execution_report() -> Result<(), crate::error::Error>
    {
        let mut book = OrderBook::default();
        let message = decode_message("8=FIX.4.4\u{0001}9=164\u{0001}35=8\u{0001}49=ACCEPTOR\u{0001}56=INITIATOR\u{0001}34=232\u{0001}52=20190929-04:51:06.981\u{0001}39=0\u{0001}11=51\u{0001}37=INITIATOR-ACCEPTOR-51\u{0001}17=2\u{0001}150=0\u{0001}151=10000\u{0001}55=WTF\u{0001}54=1\u{0001}38=10000\u{0001}32=0\u{0001}31=0\u{0001}14=0\u{0001}6=0\u{0001}40=1\u{0001}10=115\u{0001}")?;
        assert_eq!(book.process(&message), Err(crate::error::Error::OrderBookDoesNotContainOrderWithKey("INITIATOR-ACCEPTOR-51".to_string())));
        Ok(())
    }

    #[test]
    pub fn order_cancel_replace_request_for_unknown_order() -> Result<(), crate::error::Error>
    {
        let mut book = OrderBook::default();
        let message = decode_message("8=FIX.4.4\u{0001}9=178\u{0001}35=G\u{0001}49=INITIATOR\u{0001}56=ACCEPTOR\u{0001}34=2536\u{0001}52=20191117-01:01:47.010\u{0001}37=INITIATOR-ACCEPTOR-56\u{0001}41=56\u{0001}11=57\u{0001}70=55\u{0001}100=AUTO\u{0001}55=WTF\u{0001}54=1\u{0001}60=20191117-01:01:38.158\u{0001}38=100000\u{0001}40=2\u{0001}44=11\u{0001}59=0\u{0001}10=035\u{0001}")?;
        assert_eq!(book.process(&message), Err(crate::error::Error::OrderBookDoesNotContainOrderWithKey("INITIATOR-ACCEPTOR-56".to_string())));
        Ok(())
    }

    #[test]
    pub fn order_cancel_request_for_unknown_order() -> Result<(), crate::error::Error>
    {
        let mut book = OrderBook::default();
        let message = decode_message("8=FIX.4.4\u{0001}9=147\u{0001}35=F\u{0001}49=INITIATOR\u{0001}56=ACCEPTOR\u{0001}34=2544\u{0001}52=20191117-01:09:11.302\u{0001}41=58\u{0001}37=INITIATOR-ACCEPTOR-58\u{0001}11=59\u{0001}55=WTF\u{0001}54=1\u{0001}60=20191117-01:09:09.139\u{0001}38=100000\u{0001}10=092\u{0001}")?;
        assert_eq!(book.process(&message), Err(crate::error::Error::OrderBookDoesNotContainOrderWithKey("INITIATOR-ACCEPTOR-58".to_string())));
        Ok(())
    }

    #[test]
    pub fn order_cancel_reject_for_unknown_order() -> Result<(), crate::error::Error>
    {
        let mut book = OrderBook::default();
        let message = decode_message("8=FIX.4.4\u{0001}9=127\u{0001}35=9\u{0001}49=ACCEPTOR\u{0001}56=INITIATOR\u{0001}34=511\u{0001}52=20191117-01:11:06.578\u{0001}37=INITIATOR-ACCEPTOR-58\u{0001}39=8\u{0001}41=58\u{0001}434=1\u{0001}11=59\u{0001}58=Unknown order\u{0001}10=208\u{0001}")?;
        assert_eq!(book.process(&message), Err(crate::error::Error::OrderBookDoesNotContainOrderWithKey("INITIATOR-ACCEPTOR-58".to_string())));
        Ok(())
    }

    #[test]
    pub fn new_order_single_acknowledged() -> Result<(), crate::error::Error>
    {
        let messages = [
             "8=FIX.4.4\u{0001}9=140\u{0001}35=D\u{0001}49=INITIATOR\u{0001}56=ACCEPTOR\u{0001}34=2283\u{0001}52=20190929-04:51:06.973\u{0001}11=51\u{0001}70=50\u{0001}100=AUTO\u{0001}55=WTF\u{0001}54=1\u{0001}60=20190929-04:35:33.562\u{0001}38=10000\u{0001}40=1\u{0001}59=1\u{0001}10=127\u{0001}",
             "8=FIX.4.4\u{0001}9=164\u{0001}35=8\u{0001}49=ACCEPTOR\u{0001}56=INITIATOR\u{0001}34=232\u{0001}52=20190929-04:51:06.981\u{0001}39=0\u{0001}11=51\u{0001}37=INITIATOR-ACCEPTOR-51\u{0001}17=2\u{0001}150=0\u{0001}151=10000\u{0001}55=WTF\u{0001}54=1\u{0001}38=10000\u{0001}32=0\u{0001}31=0\u{0001}14=0\u{0001}6=0\u{0001}40=1\u{0001}10=115\u{0001}"
        ];
        let mut book = OrderBook::default();
        for text in messages {
            let message = decode_message(text)?;
            book.process(&message)?;
        }
        assert_eq!(book.orders.len(), 1);
        Ok(())
    }

    #[test]
    pub fn clear() -> Result<(), crate::error::Error>
    {
        let messages = [
            "8=FIX.4.4\u{0001}9=140\u{0001}35=D\u{0001}49=INITIATOR\u{0001}56=ACCEPTOR\u{0001}34=2283\u{0001}52=20190929-04:51:06.973\u{0001}11=51\u{0001}70=50\u{0001}100=AUTO\u{0001}55=WTF\u{0001}54=1\u{0001}60=20190929-04:35:33.562\u{0001}38=10000\u{0001}40=1\u{0001}59=1\u{0001}10=127\u{0001}",
            "8=FIX.4.4\u{0001}9=164\u{0001}35=8\u{0001}49=ACCEPTOR\u{0001}56=INITIATOR\u{0001}34=232\u{0001}52=20190929-04:51:06.981\u{0001}39=0\u{0001}11=51\u{0001}37=INITIATOR-ACCEPTOR-51\u{0001}17=2\u{0001}150=0\u{0001}151=10000\u{0001}55=WTF\u{0001}54=1\u{0001}38=10000\u{0001}32=0\u{0001}31=0\u{0001}14=0\u{0001}6=0\u{0001}40=1\u{0001}10=115\u{0001}"
        ];
        let mut book = OrderBook::default();
        for text in messages {
            let message = decode_message(text)?;
            book.process(&message)?;
        }
        assert_eq!(book.orders.len(), 1);
        book.clear();
        assert!(book.orders.is_empty());
        Ok(())
    }

    #[test]
    pub fn order_cancel_request_for_known_order_accepted() -> Result<(), crate::error::Error>
    {
        let messages = [
            "8=FIXT.1.1\u{0001}9=148\u{0001}35=D\u{0001}49=INITIATOR\u{0001}56=ACCEPTOR\u{0001}34=24\u{0001}52=20200119-04:43:20.679\u{0001}11=8\u{0001}70=6\u{0001}100=AUTO\u{0001}55=WTF.AX\u{0001}54=1\u{0001}60=20200119-04:43:18.221\u{0001}38=20000\u{0001}40=2\u{0001}44=11.56\u{0001}59=1\u{0001}10=081\u{0001}",
            "8=FIXT.1.1\u{0001}9=173\u{0001}35=8\u{0001}49=ACCEPTOR\u{0001}56=INITIATOR\u{0001}34=31\u{0001}52=20200119-04:43:35.419\u{0001}39=0\u{0001}11=8\u{0001}37=INITIATOR-ACCEPTOR-8\u{0001}17=1\u{0001}150=0\u{0001}151=20000\u{0001}55=WTF.AX\u{0001}54=1\u{0001}38=20000\u{0001}44=11.56\u{0001}32=0\u{0001}31=0\u{0001}14=0\u{0001}6=0\u{0001}40=2\u{0001}10=138\u{0001}",
            "8=FIXT.1.1\u{0001}9=153\u{0001}35=F\u{0001}49=INITIATOR\u{0001}56=ACCEPTOR\u{0001}34=26\u{0001}52=20200119-04:43:43.562\u{0001}41=8\u{0001}37=INITIATOR-ACCEPTOR-8\u{0001}11=9\u{0001}55=WTF.AX\u{0001}54=1\u{0001}60=20200119-04:43:42.213\u{0001}38=20000\u{0001}100=AUTO\u{0001}10=056\u{0001}",
            "8=FIXT.1.1\u{0001}9=178\u{0001}35=8\u{0001}49=ACCEPTOR\u{0001}56=INITIATOR\u{0001}34=32\u{0001}52=20200119-04:43:43.570\u{0001}39=6\u{0001}11=9\u{0001}37=INITIATOR-ACCEPTOR-8\u{0001}17=2\u{0001}150=6\u{0001}151=20000\u{0001}41=8\u{0001}55=WTF.AX\u{0001}54=1\u{0001}38=20000\u{0001}44=11.56\u{0001}32=0\u{0001}31=0\u{0001}14=0\u{0001}6=0\u{0001}40=2\u{0001}10=118\u{0001}",
            "8=FIXT.1.1\u{0001}9=174\u{0001}35=8\u{0001}49=ACCEPTOR\u{0001}56=INITIATOR\u{0001}34=33\u{0001}52=20200119-04:43:51.214\u{0001}39=4\u{0001}11=9\u{0001}37=INITIATOR-ACCEPTOR-8\u{0001}17=3\u{0001}150=4\u{0001}151=0\u{0001}41=8\u{0001}55=WTF.AX\u{0001}54=1\u{0001}38=20000\u{0001}44=11.56\u{0001}32=0\u{0001}31=0\u{0001}14=0\u{0001}6=0\u{0001}40=2\u{0001}10=168\u{0001}"
        ];
        let mut book = OrderBook::default();
        for (index, text) in messages.iter().enumerate() {
            let message = decode_message(text)?;
            book.process(&message)?;
            match index {
                0 => assert_eq!(book.orders.len(), 1),
                1 => {
                    assert_eq!(book.orders.len(), 1);
                    let order = book.orders.values().next().ok_or(Error::OrderIndexOutOfRange(0))?;
                    assert_eq!(order.fields.get(crate::FIX_5_0SP2::OrdStatus::TAG)?, crate::FIX_5_0SP2::OrdStatus::New());
                }
                2 => {
                    assert_eq!(book.orders.len(), 1);
                    let order = book.orders.values().next().ok_or(Error::OrderIndexOutOfRange(0))?;
                    assert_eq!(order.fields.get(crate::FIX_5_0SP2::OrdStatus::TAG)?, crate::FIX_5_0SP2::OrdStatus::PendingCancel());
                }
                3 => {
                    assert_eq!(book.orders.len(), 1);
                    let order = book.orders.values().next().ok_or(Error::OrderIndexOutOfRange(0))?;
                    assert_eq!(order.fields.get(crate::FIX_5_0SP2::OrdStatus::TAG)?, crate::FIX_5_0SP2::OrdStatus::PendingCancel());
                }
                4 => {
                    assert_eq!(book.orders.len(), 1);
                    let order = book.orders.values().next().ok_or(Error::OrderIndexOutOfRange(0))?;
                    assert_eq!(order.fields.get(crate::FIX_5_0SP2::OrdStatus::TAG)?, crate::FIX_5_0SP2::OrdStatus::Canceled());
                }
                _ => {
                    panic!("unexpected number of lines in input: {}", index);
                }
            }
        }
        Ok(())
    }

    #[test]
    pub fn order_cancel_request_for_known_order_rejected() -> Result<(), crate::error::Error>
    {
        let messages = [
            "8=FIXT.1.1\u{0001}9=149\u{0001}35=D\u{0001}49=INITIATOR\u{0001}56=ACCEPTOR\u{0001}34=31\u{0001}52=20200119-04:45:43.004\u{0001}11=10\u{0001}70=7\u{0001}100=AUTO\u{0001}55=WTF.AX\u{0001}54=1\u{0001}60=20200119-04:45:40.842\u{0001}38=20000\u{0001}40=2\u{0001}44=11.56\u{0001}59=1\u{0001}10=117\u{0001}",
            "8=FIXT.1.1\u{0001}9=175\u{0001}35=8\u{0001}49=ACCEPTOR\u{0001}56=INITIATOR\u{0001}34=38\u{0001}52=20200119-04:45:46.392\u{0001}39=0\u{0001}11=10\u{0001}37=INITIATOR-ACCEPTOR-10\u{0001}17=4\u{0001}150=0\u{0001}151=20000\u{0001}55=WTF.AX\u{0001}54=1\u{0001}38=20000\u{0001}44=11.56\u{0001}32=0\u{0001}31=0\u{0001}14=0\u{0001}6=0\u{0001}40=2\u{0001}10=236\u{0001}",
            "8=FIXT.1.1\u{0001}9=156\u{0001}35=F\u{0001}49=INITIATOR\u{0001}56=ACCEPTOR\u{0001}34=32\u{0001}52=20200119-04:45:53.320\u{0001}41=10\u{0001}37=INITIATOR-ACCEPTOR-10\u{0001}11=11\u{0001}55=WTF.AX\u{0001}54=1\u{0001}60=20200119-04:45:51.569\u{0001}38=20000\u{0001}100=AUTO\u{0001}10=190\u{0001}",
            "8=FIXT.1.1\u{0001}9=181\u{0001}35=8\u{0001}49=ACCEPTOR\u{0001}56=INITIATOR\u{0001}34=39\u{0001}52=20200119-04:45:53.331\u{0001}39=6\u{0001}11=11\u{0001}37=INITIATOR-ACCEPTOR-10\u{0001}17=5\u{0001}150=6\u{0001}151=20000\u{0001}41=10\u{0001}55=WTF.AX\u{0001}54=1\u{0001}38=20000\u{0001}44=11.56\u{0001}32=0\u{0001}31=0\u{0001}14=0\u{0001}6=0\u{0001}40=2\u{0001}10=243\u{0001}",
            "8=FIXT.1.1\u{0001}9=128\u{0001}35=9\u{0001}49=ACCEPTOR\u{0001}56=INITIATOR\u{0001}34=41\u{0001}52=20200119-04:46:09.609\u{0001}37=INITIATOR-ACCEPTOR-10\u{0001}39=8\u{0001}41=10\u{0001}434=1\u{0001}11=11\u{0001}58=Not telling you\u{0001}10=092\u{0001}"
        ];
        let mut book = OrderBook::default();
        for (index, text) in messages.iter().enumerate() {
            let message = decode_message(text)?;
            book.process(&message)?;
            match index {
                0 => assert_eq!(book.orders.len(), 1),
                1 => {
                    assert_eq!(book.orders.len(), 1);
                    let order = book.orders.values().next().ok_or(Error::OrderIndexOutOfRange(0))?;
                    assert_eq!(order.fields.get(crate::FIX_5_0SP2::OrdStatus::TAG)?, crate::FIX_5_0SP2::OrdStatus::New());
                }
                2 => {
                    assert_eq!(book.orders.len(), 1);
                    let order = book.orders.values().next().ok_or(Error::OrderIndexOutOfRange(0))?;
                    assert_eq!(order.fields.get(crate::FIX_5_0SP2::OrdStatus::TAG)?, crate::FIX_5_0SP2::OrdStatus::PendingCancel());
                }
                3 => {
                    assert_eq!(book.orders.len(), 1);
                    let order = book.orders.values().next().ok_or(Error::OrderIndexOutOfRange(0))?;
                    assert_eq!(order.fields.get(crate::FIX_5_0SP2::OrdStatus::TAG)?, crate::FIX_5_0SP2::OrdStatus::PendingCancel());
                }
                4 => {
                    assert_eq!(book.orders.len(), 1);
                    let order = book.orders.values().next().ok_or(Error::OrderIndexOutOfRange(0))?;
                    assert_eq!(order.fields.get(crate::FIX_5_0SP2::OrdStatus::TAG)?, crate::FIX_5_0SP2::OrdStatus::New());
                }
                _ => {
                    panic!("unexpected number of lines in input: {}", index);
                }
            }
        }
        Ok(())
    }
   
    #[test]
    pub fn order_cancel_replace_request_for_known_order_accepted() -> Result<(), crate::error::Error>
    {
        let messages = [
            "8=FIXT.1.1\u{0001}9=148\u{0001}35=D\u{0001}49=INITIATOR\u{0001}56=ACCEPTOR\u{0001}34=10\u{0001}52=20200119-02:35:09.990\u{0001}11=1\u{0001}70=1\u{0001}100=AUTO\u{0001}55=WTF.AX\u{0001}54=1\u{0001}60=20200119-02:30:33.801\u{0001}38=20000\u{0001}40=2\u{0001}44=11.56\u{0001}59=1\u{0001}10=061\u{0001}",
            "8=FIXT.1.1\u{0001}9=173\u{0001}35=8\u{0001}49=ACCEPTOR\u{0001}56=INITIATOR\u{0001}34=10\u{0001}52=20200119-02:35:12.810\u{0001}39=0\u{0001}11=1\u{0001}37=INITIATOR-ACCEPTOR-1\u{0001}17=1\u{0001}150=0\u{0001}151=20000\u{0001}55=WTF.AX\u{0001}54=1\u{0001}38=20000\u{0001}44=11.56\u{0001}32=0\u{0001}31=0\u{0001}14=0\u{0001}6=0\u{0001}40=2\u{0001}10=110\u{0001}",
            "8=FIXT.1.1\u{0001}9=178\u{0001}35=G\u{0001}49=INITIATOR\u{0001}56=ACCEPTOR\u{0001}34=11\u{0001}52=20200119-02:35:32.416\u{0001}37=INITIATOR-ACCEPTOR-1\u{0001}41=1\u{0001}11=2\u{0001}70=1\u{0001}100=AUTO\u{0001}55=WTF.AX\u{0001}54=1\u{0001}60=20200119-02:35:17.910\u{0001}38=40000\u{0001}40=2\u{0001}44=11.565\u{0001}59=1\u{0001}10=132\u{0001}",
            "8=FIXT.1.1\u{0001}9=178\u{0001}35=8\u{0001}49=ACCEPTOR\u{0001}56=INITIATOR\u{0001}34=11\u{0001}52=20200119-02:35:32.434\u{0001}39=E\u{0001}11=2\u{0001}37=INITIATOR-ACCEPTOR-1\u{0001}17=2\u{0001}150=E\u{0001}151=20000\u{0001}41=1\u{0001}55=WTF.AX\u{0001}54=1\u{0001}38=20000\u{0001}44=11.56\u{0001}32=0\u{0001}31=0\u{0001}14=0\u{0001}6=0\u{0001}40=2\u{0001}10=120\u{0001}",
            "8=FIXT.1.1\u{0001}9=175\u{0001}35=8\u{0001}49=ACCEPTOR\u{0001}56=INITIATOR\u{0001}34=12\u{0001}52=20200119-02:35:34.878\u{0001}39=5\u{0001}11=1\u{0001}37=INITIATOR-ACCEPTOR-2\u{0001}17=3\u{0001}150=5\u{0001}151=0\u{0001}41=1\u{0001}55=WTF.AX\u{0001}54=1\u{0001}38=40000\u{0001}44=11.565\u{0001}32=0\u{0001}31=0\u{0001}14=0\u{0001}6=0\u{0001}40=2\u{0001}10=218\u{0001}"
        ];
        let mut book = OrderBook::default();
        for (index, text) in messages.iter().enumerate() {
            let message = decode_message(text)?;
            book.process(&message)?;
            match index {
                0 => assert_eq!(book.orders.len(), 1),
                1 => {
                    assert_eq!(book.orders.len(), 1);
                    let order = book.orders.values().next().ok_or(Error::OrderIndexOutOfRange(0))?;
                    assert_eq!(order.fields.get(crate::FIX_5_0SP2::OrdStatus::TAG)?, crate::FIX_5_0SP2::OrdStatus::New());
                    assert_eq!(order.fields.get(crate::FIX_5_0SP2::ClOrdID::TAG)?.value, "1");
                    assert_eq!(order.fields.try_get(crate::FIX_5_0SP2::OrigClOrdID::TAG), None);
                }
                2 => {
                    assert_eq!(book.orders.len(), 1);
                    let order = book.orders.values().next().ok_or(Error::OrderIndexOutOfRange(0))?;
                    assert_eq!(order.fields.get(crate::FIX_5_0SP2::OrdStatus::TAG)?, crate::FIX_5_0SP2::OrdStatus::PendingReplace());
                    assert_eq!(order.fields.get(crate::FIX_5_0SP2::ClOrdID::TAG)?.value, "1");
                    assert_eq!(order.fields.try_get(crate::FIX_5_0SP2::OrigClOrdID::TAG), None);
                    assert_eq!(order.fields.get(crate::FIX_5_0SP2::OrderQty::TAG)?.value, "20000");
                    assert_eq!(order.pending_fields.get(crate::FIX_5_0SP2::OrderQty::TAG)?.value, "40000");
                }
                3 => {
                    assert_eq!(book.orders.len(), 1);
                    let order = book.orders.values().next().ok_or(Error::OrderIndexOutOfRange(0))?;
                    assert_eq!(order.fields.get(crate::FIX_5_0SP2::OrdStatus::TAG)?, crate::FIX_5_0SP2::OrdStatus::PendingReplace());
                    assert_eq!(order.fields.get(crate::FIX_5_0SP2::ClOrdID::TAG)?.value, "1");
                    assert_eq!(order.fields.try_get(crate::FIX_5_0SP2::OrigClOrdID::TAG), None);
                    assert_eq!(order.fields.get(crate::FIX_5_0SP2::OrderQty::TAG)?.value, "20000");
                    assert_eq!(order.pending_fields.get(crate::FIX_5_0SP2::OrderQty::TAG)?.value, "40000");
                }
                4 => {
                    assert_eq!(book.orders.len(), 2);
                    let original = book.orders.values().nth(0).ok_or(Error::OrderIndexOutOfRange(0))?;
                    let replacement = book.orders.values().nth(1).ok_or(Error::OrderIndexOutOfRange(1))?;
                    assert_eq!(original.fields.get(crate::FIX_5_0SP2::OrdStatus::TAG)?, crate::FIX_5_0SP2::OrdStatus::Replaced());
                    assert_eq!(original.fields.get(crate::FIX_5_0SP2::ClOrdID::TAG)?.value, "1");
                    assert_eq!(original.fields.get(crate::FIX_5_0SP2::OrderQty::TAG)?.value, "20000");
                    assert_eq!(replacement.fields.get(crate::FIX_5_0SP2::OrdStatus::TAG)?, crate::FIX_5_0SP2::OrdStatus::New());
                    assert_eq!(replacement.fields.get(crate::FIX_5_0SP2::ClOrdID::TAG)?.value, "2");
                    assert_eq!(replacement.fields.get(crate::FIX_5_0SP2::OrigClOrdID::TAG)?.value, "1");
                    assert_eq!(replacement.fields.get(crate::FIX_5_0SP2::OrderQty::TAG)?.value, "40000");
                }
                _ => {
                    panic!("unexpected number of lines in input: {}", index);
                }
            }
        }
        Ok(())
    }
   
    #[test]
    pub fn order_cancel_replace_request_for_known_order_rejected() -> Result<(), crate::error::Error>
    {
        let messages = [
            "8=FIXT.1.1\u{0001}9=150\u{0001}35=D\u{0001}49=INITIATOR\u{0001}56=ACCEPTOR\u{0001}34=48\u{0001}52=20200119-04:52:10.221\u{0001}11=14\u{0001}70=10\u{0001}100=AUTO\u{0001}55=WTF.AX\u{0001}54=1\u{0001}60=20200119-04:52:08.245\u{0001}38=20000\u{0001}40=2\u{0001}44=11.56\u{0001}59=1\u{0001}10=155\u{0001}",
            "8=FIXT.1.1\u{0001}9=175\u{0001}35=8\u{0001}49=ACCEPTOR\u{0001}56=INITIATOR\u{0001}34=57\u{0001}52=20200119-04:52:13.304\u{0001}39=0\u{0001}11=14\u{0001}37=INITIATOR-ACCEPTOR-14\u{0001}17=9\u{0001}150=0\u{0001}151=20000\u{0001}55=WTF.AX\u{0001}54=1\u{0001}38=20000\u{0001}44=11.56\u{0001}32=0\u{0001}31=0\u{0001}14=0\u{0001}6=0\u{0001}40=2\u{0001}10=235\u{0001}",
            "8=FIXT.1.1\u{0001}9=189\u{0001}35=G\u{0001}49=INITIATOR\u{0001}56=ACCEPTOR\u{0001}34=50\u{0001}52=20200119-04:52:29.864\u{0001}37=INITIATOR-ACCEPTOR-14\u{0001}41=14\u{0001}11=15\u{0001}70=10\u{0001}100=AUTO\u{0001}55=WTF.AX\u{0001}54=1\u{0001}60=20200119-04:52:19.840\u{0001}38=35000\u{0001}40=2\u{0001}44=12.10\u{0001}59=1\u{0001}58=Blah\u{0001}10=080\u{0001}",
            "8=FIXT.1.1\u{0001}9=182\u{0001}35=8\u{0001}49=ACCEPTOR\u{0001}56=INITIATOR\u{0001}34=59\u{0001}52=20200119-04:52:29.874\u{0001}39=E\u{0001}11=15\u{0001}37=INITIATOR-ACCEPTOR-14\u{0001}17=10\u{0001}150=E\u{0001}151=20000\u{0001}41=14\u{0001}55=WTF.AX\u{0001}54=1\u{0001}38=20000\u{0001}44=11.56\u{0001}32=0\u{0001}31=0\u{0001}14=0\u{0001}6=0\u{0001}40=2\u{0001}10=089\u{0001}",
            "8=FIXT.1.1\u{0001}9=124\u{0001}35=9\u{0001}49=ACCEPTOR\u{0001}56=INITIATOR\u{0001}34=60\u{0001}52=20200119-04:52:40.220\u{0001}37=INITIATOR-ACCEPTOR-14\u{0001}39=8\u{0001}41=14\u{0001}434=2\u{0001}11=15\u{0001}58=Not telling\u{0001}10=214\u{0001}"  
        ];
        let mut book = OrderBook::default();
        for (index, text) in messages.iter().enumerate() {
            let message = decode_message(text)?;
            book.process(&message)?;
            match index {
                0 => assert_eq!(book.orders.len(), 1),
                1 => {
                    assert_eq!(book.orders.len(), 1);
                    let order = book.orders.values().next().ok_or(Error::OrderIndexOutOfRange(0))?;
                    assert_eq!(order.fields.get(crate::FIX_5_0SP2::OrdStatus::TAG)?, crate::FIX_5_0SP2::OrdStatus::New());
                }
                2 => {
                    assert_eq!(book.orders.len(), 1);
                    let order = book.orders.values().next().ok_or(Error::OrderIndexOutOfRange(0))?;
                    assert_eq!(order.fields.get(crate::FIX_5_0SP2::OrdStatus::TAG)?, crate::FIX_5_0SP2::OrdStatus::PendingReplace());
                }
                3 => {
                    assert_eq!(book.orders.len(), 1);
                    let order = book.orders.values().next().ok_or(Error::OrderIndexOutOfRange(0))?;
                    assert_eq!(order.fields.get(crate::FIX_5_0SP2::OrdStatus::TAG)?, crate::FIX_5_0SP2::OrdStatus::PendingReplace());
                }
                4 => {
                    assert_eq!(book.orders.len(), 1);
                    let order = book.orders.values().next().ok_or(Error::OrderIndexOutOfRange(0))?;
                    assert_eq!(order.fields.get(crate::FIX_5_0SP2::OrdStatus::TAG)?, crate::FIX_5_0SP2::OrdStatus::New());
                }
                _ => {
                    panic!("unexpected number of lines in input: {}", index);
                }
            }
        }
        Ok(())
    }
   
    #[test]
    pub fn message_with_no_msg_type_ignored() -> Result<(), crate::error::Error>
    {
        let text = "8=FIX.4.4\u{0001}9=149\u{0001}49=INITIATOR\u{0001}56=ACCEPTOR\u{0001}34=2752\u{0001}52=20200114-08:13:20.041\u{0001}11=61\u{0001}70=60\u{0001}100=AUTO\u{0001}55=BHP.AX\u{0001}54=1\u{0001}60=20200114-08:12:59.397\u{0001}38=10000\u{0001}40=2\u{0001}44=20\u{0001}59=1\u{0001}10=021\u{0001}";
        let message = decode_message(text)?;
        let mut book = OrderBook::default();
        assert_eq!(book.process(&message), Err(crate::error::Error::MessageDoesNotContainMsgType));
        Ok(())
    }

    #[test]
    pub fn message_with_unsupported_msg_type_ignored() -> Result<(), crate::error::Error>
    {
        let text = "8=FIX.4.4\u{0001}9=149\u{0001}35=S\u{0001}49=INITIATOR\u{0001}56=ACCEPTOR\u{0001}34=2752\u{0001}52=20200114-08:13:20.041\u{0001}11=61\u{0001}70=60\u{0001}100=AUTO\u{0001}55=BHP.AX\u{0001}54=1\u{0001}60=20200114-08:12:59.397\u{0001}38=10000\u{0001}40=2\u{0001}44=20\u{0001}59=1\u{0001}10=021\u{0001}";
        let message = decode_message(text)?;
        let mut book = OrderBook::default();
        match book.process(&message) {
            Ok(()) => panic!(""),
            Err(error) => {
                // REQUIRE(reason == "unsupported MsgType = S");
                // REQUIRE_FALSE(processed);
            }
        };
        Ok(())
    }

    #[test]
    pub fn new_order_single_filled() -> Result<(), crate::error::Error>
    {
        let messages = [
            "8=FIX.4.4\u{0001}9=149\u{0001}35=D\u{0001}49=INITIATOR\u{0001}56=ACCEPTOR\u{0001}34=2752\u{0001}52=20200114-08:13:20.041\u{0001}11=61\u{0001}70=60\u{0001}100=AUTO\u{0001}55=BHP.AX\u{0001}54=1\u{0001}60=20200114-08:12:59.397\u{0001}38=10000\u{0001}40=2\u{0001}44=20\u{0001}59=1\u{0001}10=021\u{0001}",
            "8=FIX.4.4\u{0001}9=173\u{0001}35=8\u{0001}49=ACCEPTOR\u{0001}56=INITIATOR\u{0001}34=718\u{0001}52=20200114-08:13:20.072\u{0001}39=0\u{0001}11=61\u{0001}37=INITIATOR-ACCEPTOR-61\u{0001}17=1\u{0001}150=0\u{0001}151=10000\u{0001}55=BHP.AX\u{0001}54=1\u{0001}38=10000\u{0001}44=20\u{0001}32=0\u{0001}31=0\u{0001}14=0\u{0001}6=0\u{0001}40=2\u{0001}10=021\u{0001}",
            "8=FIX.4.4\u{0001}9=187\u{0001}35=8\u{0001}49=ACCEPTOR\u{0001}56=INITIATOR\u{0001}34=719\u{0001}52=20200114-08:13:20.072\u{0001}39=1\u{0001}11=61\u{0001}37=INITIATOR-ACCEPTOR-61\u{0001}17=2\u{0001}150=1\u{0001}151=893\u{0001}55=BHP.AX\u{0001}54=1\u{0001}38=10000\u{0001}44=20\u{0001}32=9107\u{0001}31=20\u{0001}14=9107\u{0001}6=20\u{0001}30=AUTO\u{0001}40=2\u{0001}10=081\u{0001}",
            "8=FIX.4.4\u{0001}9=185\u{0001}35=8\u{0001}49=ACCEPTOR\u{0001}56=INITIATOR\u{0001}34=720\u{0001}52=20200114-08:13:20.072\u{0001}39=2\u{0001}11=61\u{0001}37=INITIATOR-ACCEPTOR-61\u{0001}17=3\u{0001}150=2\u{0001}151=0\u{0001}55=BHP.AX\u{0001}54=1\u{0001}38=10000\u{0001}44=20\u{0001}32=893\u{0001}31=20\u{0001}14=10000\u{0001}6=20\u{0001}30=AUTO\u{0001}40=2\u{0001}10=201\u{0001}"
        ];
        let mut book = OrderBook::default();
        for (index, text) in messages.iter().enumerate() {
            let message = decode_message(text)?;
            book.process(&message)?;
            match index {
                0 => assert_eq!(book.orders.len(), 1),
                1 => {
                    assert_eq!(book.orders.len(), 1);
                    let order = book.orders.values().next().ok_or(Error::OrderIndexOutOfRange(0))?;
                    assert_eq!(order.fields.get(crate::FIX_5_0SP2::OrdStatus::TAG)?, crate::FIX_5_0SP2::OrdStatus::New());
                    // TODO - numeric comparisons?
                    assert_eq!(order.fields.get(crate::FIX_5_0SP2::CumQty::TAG)?.value, "0");
                    assert_eq!(order.fields.get(crate::FIX_5_0SP2::AvgPx::TAG)?.value, "0");
                }
                2 => {
                    assert_eq!(book.orders.len(), 1);
                    let order = book.orders.values().next().ok_or(Error::OrderIndexOutOfRange(0))?;
                    assert_eq!(order.fields.get(crate::FIX_5_0SP2::OrdStatus::TAG)?, crate::FIX_5_0SP2::OrdStatus::PartiallyFilled());
                    // TODO - numeric comparisons?
                    assert_eq!(order.fields.get(crate::FIX_5_0SP2::CumQty::TAG)?.value, "9107");
                    assert_eq!(order.fields.get(crate::FIX_5_0SP2::AvgPx::TAG)?.value, "20");
                }
                3 => {
                    assert_eq!(book.orders.len(), 1);
                    let order = book.orders.values().next().ok_or(Error::OrderIndexOutOfRange(0))?;
                    assert_eq!(order.fields.get(crate::FIX_5_0SP2::OrdStatus::TAG)?, crate::FIX_5_0SP2::OrdStatus::Filled());
                    // TODO - numeric comparisons?
                    assert_eq!(order.fields.get(crate::FIX_5_0SP2::CumQty::TAG)?.value, "10000");
                    assert_eq!(order.fields.get(crate::FIX_5_0SP2::AvgPx::TAG)?.value, "20");
                }
                _ => {
                    panic!("unexpected number of lines in input: {}", index);
                }
            }
        }
        Ok(())
    }
    
}