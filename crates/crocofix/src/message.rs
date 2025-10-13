use crate::field::Field;
use crate::error::Error;
// use crate::FIX_5_0SP2::orchestration as FIX_5_0SP2;
// use crate::dictionary::Orchestration;
use crate::dictionary::VersionField;
use std::fmt;
use bitflags::bitflags;

const VALUE_SEPARATOR: u8 = b'=';
const FIELD_SEPARATOR: u8 = 0x01;

#[derive(Debug, PartialEq)]
pub struct DecodeResult
{
    pub consumed: usize,
    pub complete: bool
}

// It is useful to be able to disable the setting of various standard fields when encoding for testing purposes.
bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
    pub struct EncodeOptions: u8 {
        const SetChecksum     = 0b0001;
        const SetBodyLength   = 0b0010;
        const SetBeginString  = 0b0100;
        const SetMsgSeqNum    = 0b1000;
        const Standard        = Self::SetChecksum.bits() | Self::SetBodyLength.bits() | Self::SetBeginString.bits() | Self::SetMsgSeqNum.bits();
    }
}

#[derive(Default)]
pub struct Message {

    // TODO - Vec is a placeholder for a specialised collection
    pub fields: Vec<Field>,
    decode_checksum: u32,
    decode_checksum_valid: bool

}

impl Message {

    // Decode FIX tag/value pairs and store them in this message. This does no validation of
    // the field content and does not validate the BodyLength or CheckSum. This supports
    // decoding fragmentary messages. This method is restartable, it can be called multiple
    // times with separate pieces of the same message until completion. This method does not
    // track completeness on subsequent calls so you can call it again after it has returned
    // complete=true and it will decode and store any fields it reads which may result in an
    // invalid message.
    pub fn decode(&mut self, buffer: &[u8]) -> Result<DecodeResult, Error>
    {
        let mut current_index = 0;
        let mut checksum_index = 0;
        let mut complete = false;

        while current_index < buffer.len() {

            let equals_index = match buffer[current_index..].iter().position(|&byte| byte == VALUE_SEPARATOR) {
                Some(position) => current_index + position,
                None => break
            };

            let tag_bytes = &buffer[current_index..equals_index];

            let tag: u32 = std::str::from_utf8(tag_bytes)
                .map_err(|error| Error::InvalidUtf8(error))
                .and_then(|string| string.parse().map_err(|error| Error::TagParseFailed))?;

            
            let separator_index = match buffer[equals_index + 1..].iter().position(|&byte| byte == FIELD_SEPARATOR) {
                Some(position) => equals_index + 1 + position,
                None => break,
            };

            let value_index = equals_index + 1;
            let value_len = separator_index - value_index;
            let value_bytes = &buffer[value_index..value_index + value_len];
            let value = std::str::from_utf8(value_bytes).map_err(|error| Error::InvalidUtf8(error))?;

            self.fields.push(Field { tag, value: value.to_string() });

            current_index = separator_index + 1;

            if tag == crate::FIX_5_0SP2::CheckSum::TAG {
                complete = true;
                break;
            }

            checksum_index = current_index;
        }

        /*
        let mut current_idx = 0;
        let mut checksum_idx = 0;
        let buffer_len = buffer.len();
        let mut complete = false;

        while current_idx < buffer_len {
            // Find the '=' separator
            let equals_pos = match buffer[current_idx..].iter().position(|&b| b == VALUE_SEPARATOR) {
                Some(pos) => current_idx + pos,
                None => break,
            };

            // Parse the tag
            let tag_bytes = &buffer[current_idx..equals_pos];
            let tag_str = std::str::from_utf8(tag_bytes)
                .map_err(|_| "Invalid UTF-8 in tag".to_string())?;
            let tag = i32::from_str(tag_str)
                .map_err(|_| format!("{} is not a valid field tag", tag_str))?;

            // Check if this is a data field
            if self.is_data_field(tag) {
                if self.fields.is_empty() {
                    return Err(format!(
                        "parsed a data field with tag={} that was not preceeded by a length field",
                        tag
                    ));
                }

                // Get length from previous field
                let length = self.fields.last()
                    .and_then(|f| usize::from_str(f.value()).ok())
                    .ok_or_else(|| format!(
                        "parsed a data field with tag={} but the preceeding field value was not a valid numeric length",
                        tag
                    ))?;

                let field_end_idx = equals_pos + length + 1;
                if field_end_idx >= buffer_len {
                    break;
                }

                // Extract value
                let value_start = equals_pos + 1;
                let value = &buffer[value_start..value_start + length];
                
                // Check for field separator
                if buffer[field_end_idx] != FIELD_SEPARATOR {
                    return Err(format!(
                        "parsed a data field with tag={} but the field did not have a trailing field separator",
                        tag
                    ));
                }

                let value_str = std::str::from_utf8(value)
                    .map_err(|_| "Invalid UTF-8 in data field value".to_string())?;
                self.fields.push(Field {
                    tag,
                    value: value_str.to_string(),
                });

                // +1 for field separator, +1 to move to next tag
                current_idx = field_end_idx + 1;
            } else {
                // Find the field separator
                let delimiter_pos = match buffer[equals_pos + 1..].iter().position(|&b| b == FIELD_SEPARATOR) {
                    Some(pos) => equals_pos + 1 + pos,
                    None => break,
                };

                // Extract value
                let value_start = equals_pos + 1;
                let value_len = delimiter_pos - value_start;
                let value = &buffer[value_start..value_start + value_len];
                let value_str = std::str::from_utf8(value)
                    .map_err(|_| "Invalid UTF-8 in field value".to_string())?;
                
                self.fields.push(Field {
                    tag,
                    value: value_str.to_string(),
                });

                // +1 to move past delimiter
                current_idx = delimiter_pos + 1;
            }

            // Check if we've reached the checksum field
            if tag == CHECKSUM_TAG {
                complete = true;
                break;
            }

            // Update checksum index
            checksum_idx = current_idx;
        }
        */

        self.decode_checksum += buffer[..checksum_index].iter().map(|&byte| byte as u32).sum::<u32>();
    
        if complete {
            self.decode_checksum %= 256;
            self.decode_checksum_valid = true;
        }

        Ok(DecodeResult { consumed: current_index, complete })
    }

    // Encode this FIX message into the supplied buffer. This method calculates 
    // and rewrites the BodyLength and CheckSum by default, these fields must already be present, they
    // will not be added. It does no validation of the message content/structure. 
    // Returns 0 if the buffer is not big enough.
    pub fn encode(&self, buffer: &mut Vec<u8>, options: EncodeOptions) -> Result<usize, Error>
    {
        Ok(0)
    }

    pub fn is_admin(&self) -> bool {
        false
    }

}

impl fmt::Display for Message {
    
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result
    {
        write!(formatter, "FIX MESSAGE")?;
        Ok(())
    }

}


#[cfg(test)]
mod tests {

    use super::*;
 
    #[test]
    fn decode_a_complete_message() -> Result<(), crate::error::Error>
    {
        let text = "8=FIX.4.4\u{0001}9=149\u{0001}35=D\u{0001}49=INITIATOR\u{0001}56=ACCEPTOR\u{0001}34=2752\u{0001}52=20200114-08:13:20.041\u{0001}11=61\u{0001}70=60\u{0001}100=AUTO\u{0001}55=BHP.AX\u{0001}54=1\u{0001}60=20200114-08:12:59.397\u{0001}38=10000\u{0001}40=2\u{0001}44=20\u{0001}59=1\u{0001}10=021\u{0001}";
        let mut message = Message::default();
        let result = message.decode(text.as_bytes())?;
        assert!(result.complete);
        assert_eq!(result.consumed, text.len());
        assert_eq!(message.fields.len(), 18);
        Ok(())
    }

    #[test]
    fn decode_a_complete_message_in_two_pieces_aligned_on_a_field_boundary() -> Result<(), crate::error::Error>
    {
        let one = "8=FIX.4.4\u{0001}9=149\u{0001}35=D\u{0001}49=INITIATOR\u{0001}56=ACCEPTOR\u{0001}34=2752\u{0001}52=20200114-08:13:20.041\u{0001}11=61\u{0001}70=60\u{0001}100=AUTO\u{0001}55=BHP.AX\u{0001}";
        let two = "54=1\u{0001}60=20200114-08:12:59.397\u{0001}38=10000\u{0001}40=2\u{0001}44=20\u{0001}59=1\u{0001}10=021\u{0001}";
        let mut message = Message::default();
        let one_result = message.decode(one.as_bytes())?;
        assert!(!one_result.complete);
        assert_eq!(one_result.consumed, one.len());
        let two_result = message.decode(two.as_bytes())?;
        assert!(two_result.complete);
        assert_eq!(two_result.consumed, two.len());
        assert_eq!(message.fields.len(), 18);
        Ok(())
    }

    #[test]
    fn decode_a_complete_message_in_two_pieces_not_aligned_on_a_field_boundary() -> Result<(), crate::error::Error>
    {
        let one = "8=FIX.4.4\u{0001}9=149\u{0001}35=D\u{0001}49=INITIATOR\u{0001}56=ACCEPTOR\u{0001}34=2752\u{0001}52=20200114-08:13:20.041\u{0001}11=61\u{0001}70=60\u{0001}100=AUTO\u{0001}55=B";
        let two = "55=BHP.AX\u{0001}54=1\u{0001}60=20200114-08:12:59.397\u{0001}38=10000\u{0001}40=2\u{0001}44=20\u{0001}59=1\u{0001}10=021\u{0001}";
        let mut message = Message::default();
        let one_result = message.decode(one.as_bytes())?;
        assert!(!one_result.complete);
        assert_eq!(one_result.consumed, one.len() - "55=B".len());
        let two_result = message.decode(two.as_bytes())?;
        assert!(two_result.complete);
        assert_eq!(two_result.consumed, two.len());
        assert_eq!(message.fields.len(), 18);
        Ok(())
    }
  
    #[test]
    fn invalid_tag() -> Result<(), crate::error::Error>
    {
        /*
        crocofix::message message;
        REQUIRE_THROWS_AS(message.decode("A=FIX.4.4"), std::out_of_range);
        */
        Ok(())
    }

    #[test]
    fn msg_type_lookup_fails_for_a_message_with_no_msg_type() -> Result<(), crate::error::Error>
    {
        /*
        const std::string text = "8=FIX.4.4\u{0001}9=149\u{0001}49=INITIATOR\u{0001}56=ACCEPTOR\u{0001}34=2752\u{0001}52=20200114-08:13:20.041\u{0001}11=61\u{0001}70=60\u{0001}100=AUTO\u{0001}55=BHP.AX\u{0001}54=1\u{0001}60=20200114-08:12:59.397\u{0001}38=10000\u{0001}40=2\u{0001}44=20\u{0001}59=1\u{0001}10=021\u{0001}";
        crocofix::message message;
        message.decode(text);
        REQUIRE_THROWS(message.MsgType());
        */
        Ok(())
    }

    #[test]
    fn msg_type_lookup() -> Result<(), crate::error::Error>
    {
        /*
        const std::string text = "8=FIX.4.4\u{0001}9=149\u{0001}35=D\u{0001}49=INITIATOR\u{0001}56=ACCEPTOR\u{0001}34=2752\u{0001}52=20200114-08:13:20.041\u{0001}11=61\u{0001}70=60\u{0001}100=AUTO\u{0001}55=BHP.AX\u{0001}54=1\u{0001}60=20200114-08:12:59.397\u{0001}38=10000\u{0001}40=2\u{0001}44=20\u{0001}59=1\u{0001}10=021\u{0001}";
        crocofix::message message;
        message.decode(text);
        REQUIRE(message.MsgType() == "D");
        */
        Ok(())
    }

    #[test]
    fn is_admin_is_false_for_a_non_admin_message() -> Result<(), crate::error::Error>
    {
        // let text = "8=FIX.4.4\u{0001}9=149\u{0001}35=D\u{0001}49=INITIATOR\u{0001}56=ACCEPTOR\u{0001}34=2752\u{0001}52=20200114-08:13:20.041\u{0001}11=61\u{0001}70=60\u{0001}100=AUTO\u{0001}55=BHP.AX\u{0001}54=1\u{0001}60=20200114-08:12:59.397\u{0001}38=10000\u{0001}40=2\u{0001}44=20\u{0001}59=1\u{0001}10=021\u{0001}";
        // let mut message = Message::default();
        // message.decode(text.as_bytes())?;
        // assert!(!message.is_admin());        
        Ok(())
    }

    #[test]
    fn is_admin_is_true_for_an_admin_message() -> Result<(), crate::error::Error>
    {
        // let text = "8=FIX.4.4\u{0001}9=149\u{0001}35=A\u{0001}49=INITIATOR\u{0001}56=ACCEPTOR\u{0001}34=2752\u{0001}52=20200114-08:13:20.041\u{0001}11=61\u{0001}70=60\u{0001}100=AUTO\u{0001}55=BHP.AX\u{0001}54=1\u{0001}60=20200114-08:12:59.397\u{0001}38=10000\u{0001}40=2\u{0001}44=20\u{0001}59=1\u{0001}10=021\u{0001}";
        // let mut message = Message::default();
        // message.decode(text.as_bytes())?;
        // assert!(message.is_admin());        
        Ok(())
    }

    #[test]
    fn encode_a_message() -> Result<(), crate::error::Error>
    {
        // let expected = "8=FIX.4.4\u{0001}9=149\u{0001}35=D\u{0001}49=INITIATOR\u{0001}56=ACCEPTOR\u{0001}34=2752\u{0001}52=20200114-08:13:20.041\u{0001}11=61\u{0001}70=60\u{0001}100=AUTO\u{0001}55=BHP.AX\u{0001}54=1\u{0001}60=20200114-08:12:59.397\u{0001}38=10000\u{0001}40=2\u{0001}44=20\u{0001}59=1\u{0001}10=021\u{0001}";
        // let mut message = Message::default();
        // message.decode(expected.as_bytes())?;

        // let mut actual = Vec::new();
        // let result = message.encode(&mut actual, EncodeOptions::Standard)?;
        // assert!(result > 0);
        // assert_eq!(actual, expected.as_bytes());
        
        Ok(())
    }

    #[test]
    fn encode_does_not_add_checksum_if_it_is_not_present() -> Result<(), crate::error::Error>
    {
        // let expected = "8=FIX.4.4\u{0001}9=149\u{0001}35=D\u{0001}49=INITIATOR\u{0001}56=ACCEPTOR\u{0001}34=2752\u{0001}52=20200114-08:13:20.041\u{0001}11=61\u{0001}70=60\u{0001}100=AUTO\u{0001}55=BHP.AX\u{0001}54=1\u{0001}60=20200114-08:12:59.397\u{0001}38=10000\u{0001}40=2\u{0001}44=20\u{0001}59=1\u{0001}";
        // let mut message = Message::default();
        // message.decode(expected.as_bytes())?;

        // let mut actual = Vec::new();
        // let result = message.encode(&mut actual, EncodeOptions::Standard)?;
        // assert!(result > 0);
        // assert_eq!(actual, expected.as_bytes());
       
        Ok(())
    }

    #[test]
    fn encode_does_not_add_body_length_if_it_is_not_present() -> Result<(), crate::error::Error>
    {
        // let expected = "8=FIX.4.4\u{0001}35=D\u{0001}49=INITIATOR\u{0001}56=ACCEPTOR\u{0001}34=2752\u{0001}52=20200114-08:13:20.041\u{0001}11=61\u{0001}70=60\u{0001}100=AUTO\u{0001}55=BHP.AX\u{0001}54=1\u{0001}60=20200114-08:12:59.397\u{0001}38=10000\u{0001}40=2\u{0001}44=20\u{0001}59=1\u{0001}10=021\u{0001}";
        // let mut message = Message::default();
        // message.decode(expected.as_bytes())?;

        // let mut actual = Vec::new();
        // let result = message.encode(&mut actual, EncodeOptions::Standard - EncodeOptions::SetChecksum)?;
        // assert!(result > 0);
        // assert_eq!(actual, expected.as_bytes());

        Ok(())
    }

    #[test]
    fn format_checksum_greater_than_3_digits_fails() -> Result<(), crate::error::Error>
    {
        // REQUIRE_THROWS(message::format_checksum(9999));
        Ok(())
    }

    #[test]
    fn format_checksum_pads_values_with_less_than_3_digits() -> Result<(), crate::error::Error>
    {
        /*
        REQUIRE(message::format_checksum(999) == "999");
        REQUIRE(message::format_checksum(99) == "099");
        REQUIRE(message::format_checksum(9) == "009");
        REQUIRE(message::format_checksum(0) == "000");
        REQUIRE(message::format_checksum(90) == "090");
        REQUIRE(message::format_checksum(900) == "900");
        */
        Ok(())
    }

    #[test]
    fn decode_a_message_with_a_data_field_that_has_no_preceeding_size_field() -> Result<(), crate::error::Error>
    {
        // let text = "8=FIX.4.4\u{0001}9=149\u{0001}35=D\u{0001}89=123\u{0001}49=INITIATOR\u{0001}56=ACCEPTOR\u{0001}34=2752\u{0001}52=20200114-08:13:20.041\u{0001}11=61\u{0001}70=60\u{0001}100=AUTO\u{0001}55=BHP.AX\u{0001}54=1\u{0001}60=20200114-08:12:59.397\u{0001}38=10000\u{0001}40=2\u{0001}44=20\u{0001}59=1\u{0001}10=021\u{0001}";
        // let mut message = Message::default();
        // assert_eq!(message.decode(text.as_bytes()), Err(crate::error::Error::DataFieldWithNoPrecedingSizeField));
        Ok(())
    }

    #[test]
    fn decode_a_message_with_a_data_field_with_a_non_numeric_previous_field_value() -> Result<(), crate::error::Error>
    {
        // let text = "8=FIX.4.4\u{0001}9=149\u{0001}35=D\u{0001}89=123\u{0001}49=INITIATOR\u{0001}56=ACCEPTOR\u{0001}34=2752\u{0001}52=20200114-08:13:20.041\u{0001}11=61\u{0001}70=60\u{0001}100=AUTO\u{0001}55=BHP.AX\u{0001}54=1\u{0001}60=20200114-08:12:59.397\u{0001}38=10000\u{0001}40=2\u{0001}44=20\u{0001}59=1\u{0001}10=021\u{0001}";
        // let mut message = Message::default();
        // assert_eq!(message.decode(text.as_bytes()), Err(crate::error::Error::DataFieldWithNonNumericPreviousField));
        Ok(())
    }

    #[test]
    fn decode_a_message_with_a_data_field_that_does_not_have_a_trailing_field_separator() -> Result<(), crate::error::Error>
    {
        // let text = "8=FIX.4.4\u{0001}9=149\u{0001}35=D\u{0001}93=3\u{0001}89=AAA49=INITIATOR\u{0001}56=ACCEPTOR\u{0001}34=2752\u{0001}52=20200114-08:13:20.041\u{0001}11=61\u{0001}70=60\u{0001}100=AUTO\u{0001}55=BHP.AX\u{0001}54=1\u{0001}60=20200114-08:12:59.397\u{0001}38=10000\u{0001}40=2\u{0001}44=20\u{0001}59=1\u{0001}10=021\u{0001}";
        // let mut message = Message::default();
        // assert_eq!(message.decode(text.as_bytes()), Err(crate::error::Error::DataFieldWithNoTrailingSeparator));
        Ok(())
    }

    #[test]
    fn decode_a_message_containing_a_data_field() -> Result<(), crate::error::Error>
    {
        // let signature = "ABCDEF\u{0001}ABCDEFABC\u{0001}DEF";
        // let text = "8=FIX.4.4\u{0001}9=167\u{0001}35=D\u{0001}49=INITIATOR\u{0001}56=ACCEPTOR\u{0001}34=2752\u{0001}52=20200114-08:13:20.041\u{0001}11=61\u{0001}70=60\u{0001}100=AUTO\u{0001}55=BHP.AX\u{0001}54=1\u{0001}60=20200114-08:12:59.397\u{0001}38=10000\u{0001}40=2\u{0001}44=20\u{0001}59=1\u{0001}93=20\u{0001}89=ABCDEF\u{0001}ABCDEFABC\u{0001}DEF\u{0001}10=220\u{0001}";
        // let mut message = Message::default();
        // let result = message.decode(text.as_bytes())?;
        // assert!(result.complete);
        // assert_eq!(message.fields.len(), 20);
        // assert_eq!(message.fields[18].value.len(), signature.len());
        // assert_eq!(signature, message.fields[18].value);
        Ok(())
    }

    #[test]
    fn decode_a_message_containing_a_data_field_in_two_pieces() -> Result<(), crate::error::Error>
    {
        // let signature = "ABCDEF\u{0001}ABCDEFABC\u{0001}DEF";
        // let one = "8=FIX.4.4\u{0001}9=167\u{0001}35=D\u{0001}49=INITIATOR\u{0001}56=ACCEPTOR\u{0001}34=2752\u{0001}52=20200114-08:13:20.041\u{0001}11=61\u{0001}70=60\u{0001}100=AUTO\u{0001}55=BHP.AX\u{0001}54=1\u{0001}60=20200114-08:12:59.397\u{0001}38=10000\u{0001}40=2\u{0001}44=20\u{0001}59=1\u{0001}93=20\u{0001}89=ABCDEF\u{0001}ABCDE";
        // let two = "89=ABCDEF\u{0001}ABCDEFABC\u{0001}DEF\u{0001}10=220\u{0001}";
        // let mut message = Message::default();
        // let one_result = message.decode(one.as_bytes())?;
        // assert!(!one_result.complete);
        // assert_eq!(one_result.consumed, one.len() - "89=ABCDEF\u{0001}ABCDE".len());
        // assert_eq!(message.fields.len(), 18);
        // let two_result = message.decode(two.as_bytes())?;
        // assert!(two_result.complete);
        // assert_eq!(message.fields.len(), 20);
        // assert_eq!(message.fields[18].value.len(), signature.len());
        // assert_eq!(signature, message.fields[18].value);
        Ok(())
    }

    #[test]
    fn encode_a_message_containing_a_data_field() -> Result<(), crate::error::Error>
    {
        // let expected = "8=FIX.4.4\u{0001}9=30\u{0001}93=20\u{0001}89=ABCDEF\u{0001}ABCDEFABC\u{0001}DEF\u{0001}10=119\u{0001}";

        // let mut message = Message::default();
        
        // message.fields().emplace_back(8, "FIX.4.4");
        // message.fields().emplace_back(9, "40");
        // message.fields().emplace_back(93, "20");
        // message.fields().emplace_back(89, "ABCDEF\u{0001}ABCDEFABC\u{0001}DEF");
        // message.fields().emplace_back(10, "220");

        // let mut actual = Vec::new();
        // let result = message.encode(&mut actual, EncodeOptions::Standard)?;
        // assert!(result > 0);
        // assert_eq!(actual, expected.as_bytes());
        
        Ok(())
    }
 
}