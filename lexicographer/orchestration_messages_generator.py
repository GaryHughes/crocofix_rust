#!/usr/bin/python3

from sanitise import *

def generate_orchestration_messages(file, orchestration, module):
    
    file.write("\npub mod message {\n\n")
    
    for message in orchestration.messages.values():

        file.write("pub struct {} {{\n".format(message.name))
        file.write("}\n\n")

        file.write("impl crate::dictionary::Message for {} {{\n".format(message.name))
        file.write("    fn name(&self) -> &str {{ \"{}\" }}\n".format(message.name))
        file.write("    fn msg_type(&self) -> &str {{ \"{}\" }}\n".format(message.msg_type))
        file.write("    fn category(&self) -> &str {{ \"{}\" }}\n".format(message.category))
        file.write("    fn synopsis(&self) -> &str {{ \"{}\" }}\n".format(sanitise(message.synopsis)))
        file.write("    fn pedigree(&self) -> crate::dictionary::Pedigree {\n")
        file.write("        crate::dictionary::Pedigree {\n")
        file.write("            added: {},\n".format(format_pedigree(message.pedigree.added)))
        file.write("            added_ep: {},\n".format(format_pedigree(message.pedigree.addedEP)))
        file.write("            updated: {},\n".format(format_pedigree(message.pedigree.updated)))
        file.write("            updated_ep: {},\n".format(format_pedigree(message.pedigree.updatedEP)))
        file.write("            deprecated: {},\n".format(format_pedigree(message.pedigree.deprecated)))
        file.write("            deprecated_ep: {}\n".format(format_pedigree(message.pedigree.deprecatedEP)))
        file.write("        }\n")
        file.write("    }\n")

        file.write("    fn fields(&self) -> &'static Vec<Box<crate::dictionary::MessageField>> {\n")
        file.write("        static VALUES: std::sync::OnceLock<Vec<Box<crate::dictionary::MessageField>>> = std::sync::OnceLock::new();\n")
        file.write("        VALUES.get_or_init(|| {\n")
        file.write("            vec![\n")

        for field in orchestration.message_fields(message):
            file.write("                Box::new(crate::dictionary::MessageField::new(Box::new(crate::{}::{}{{}}), {}, {})),\n".format(module, field.field.name, format_presence(field.presence), field.depth))

        file.write("            ]\n")
        file.write("        })\n")
        file.write("    }\n")

        file.write("}\n\n")
    
    file.write("}\n\n") # pub mod message

    file.write("pub fn messages() -> &'static crate::dictionary::VersionMessageCollection {\n")
    file.write("    static FIELDS: std::sync::OnceLock<crate::dictionary::VersionMessageCollection> = std::sync::OnceLock::new();\n")
    file.write("    FIELDS.get_or_init(|| {\n")
    file.write("        crate::dictionary::VersionMessageCollection::new(\n")
    file.write("            vec![\n")

    for message in orchestration.messages.values():
        file.write('                Box::new(crate::{}::message::{}{{}}),\n'.format(module, message.name))
                   
    file.write("            ]\n")
    file.write("       )\n")
    file.write("   })\n")
    file.write("}\n")

    
