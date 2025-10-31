use crate::order_book::OrderBook;
use crate::order::Order;
use crate::field::Field;
use crate::dictionary::OrchestrationField;
use std::io::Write;
use comfy_table::*;
use comfy_table::presets::ASCII_BORDERS_ONLY_CONDENSED;


pub const DEFAULT_FIELDS: &[u32] = &[
    crate::FIX_5_0SP2::SenderCompID::TAG,
    crate::FIX_5_0SP2::TargetCompID::TAG,
    crate::FIX_5_0SP2::ClOrdID::TAG,
    crate::FIX_5_0SP2::OrigClOrdID::TAG,
    crate::FIX_5_0SP2::Symbol::TAG,
    crate::FIX_5_0SP2::OrdStatus::TAG,
    crate::FIX_5_0SP2::OrdType::TAG,
    crate::FIX_5_0SP2::TimeInForce::TAG,
    crate::FIX_5_0SP2::Side::TAG,
    crate::FIX_5_0SP2::OrderQty::TAG,
    crate::FIX_5_0SP2::Price::TAG,
    crate::FIX_5_0SP2::CumQty::TAG,
    crate::FIX_5_0SP2::AvgPx::TAG                                
];

#[derive(Default)]
pub struct OrderReport {

    fields: Vec<u32>
}

impl OrderReport
{
    pub fn with_fields(fields: Vec<u32>) -> Self
    {
        Self {
            fields: fields
        }
    }

    fn pending_value_for_field(order: &Order, field: &Field, definition: &Box<dyn OrchestrationField>) -> Option<String>
    {
        if Order::is_identity_field(field.tag) {
            return None;
        }

        let Some(pending) = order.pending_fields.try_get(field.tag) else {
            return None;
        };

        if pending.value == field.value {
            return None;
        }

        let text = match definition.name_of_value(pending.value.as_str()) {
            Some(name) => name.to_string(),
            None => pending.value.clone()
        };

        Some(format!("({})", text))
    }

    pub fn print<W: Write>(&mut self, writer: &mut W, order_book: &OrderBook) -> std::io::Result<()>
    {
        let mut table = Table::default();

        table.load_preset(ASCII_BORDERS_ONLY_CONDENSED)
            .set_style(TableComponent::HeaderLines, '-')
            .set_style(TableComponent::MiddleHeaderIntersections, '-')
        ;

        let headers: Vec<Cell> = self.fields.iter().map(|tag| {
            // TODO - field lookup needs to return an option 
            let field = &crate::FIX_5_0SP2::fields()[*tag as usize];
            Cell::new(field.name().to_string()).set_alignment(
                if field.is_numeric() {
                    CellAlignment::Right
                }
                else {
                    CellAlignment::Left
                }
            )
        }).collect();


        table.set_header(headers);
        
        order_book.orders
            .values()
            .for_each(|order| {
                let row: Vec<Cell> = self.fields.iter().map(|tag| {
                    if let Some(field) = order.fields.try_get(*tag) {

                        let definition = &crate::FIX_5_0SP2::fields()[*tag as usize];
                        
                        let value = match definition.name_of_value(field.value.as_str()) {
                            None => field.value.clone(),
                            Some(name) => name.to_string()
                        };

                        let text = match OrderReport::pending_value_for_field(&order, &field, &definition) {
                            Some(pending_value) => value + " " + pending_value.as_str(),
                            None => value
                        };

                        Cell::new(text).set_alignment(
                            if definition.is_numeric() {
                                CellAlignment::Right
                            }
                            else {
                                CellAlignment::Left
                            }
                        )
                    }        
                    else {
                        Cell::new("".to_string())
                    }
                }).collect();
                table.add_row(row);
            });
        
        let report = format!("{}\n\n", table);

        writer.write(report.as_bytes())?;

        Ok(())
    }
}