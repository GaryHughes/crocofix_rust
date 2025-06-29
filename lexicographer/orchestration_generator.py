#!/usr/bin/python3

from sanitise import *

def generate_orchestration(file, orchestration, module):
    pass




# namespace crocofix::FIX_4_2
# {

# class orchestration : public crocofix::dictionary::orchestration
# {
# public:
    
#     orchestration()
#     : crocofix::dictionary::orchestration(crocofix::FIX_4_2::messages(), crocofix::FIX_4_2::fields())
#     {
#     }

# };

# }


# namespace crocofix::dictionary
# {

# class orchestration
# {
# public:

#     orchestration(const message_collection& messages,
#                   const orchestration_field_collection& fields)
#     :   m_messages(messages),
#         m_fields(fields)
#     {
#     }

#     const message_collection& messages() const noexcept { return m_messages; }
#     const orchestration_field_collection& fields() const noexcept { return m_fields; }

#     bool is_field_defined(size_t tag) const
#     {
#         return fields()[tag].is_valid();
#     }

# private:

#     const message_collection& m_messages;
#     const orchestration_field_collection& m_fields;

# };

# }