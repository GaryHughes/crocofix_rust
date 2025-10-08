#!/usr/bin/python3

from sanitise import *

def generate_orchestration_fields(file, orchestration, module):
    sorted_fields = sorted(orchestration.fields_by_tag.values(), key=lambda x: int(x.id))

    for field in sorted_fields:

        file.write("pub struct {} {{\n".format(field.name))
        file.write("}\n\n")

        try:
            code_set = orchestration.code_sets[field.type]
            if len(code_set.codes) > 0:
                file.write("impl {} {{\n".format(field.name))
                file.write('\n')
                for code in code_set.codes:
                    file.write("    pub fn {}() -> &'static crate::dictionary::FieldValue {{\n".format(code.name))
                    file.write("        static VALUE: crate::dictionary::FieldValue = crate::dictionary::FieldValue {{ tag: {}, name: \"{}\", value: \"{}\" }};\n".format(field.id, code.name, code.value))
                    file.write("        &VALUE\n")
                    file.write("    }\n\n")
                file.write("}\n\n")
        except KeyError:
            # TODO - maybe check that its an expected built in type
            pass

   
        file.write("impl crate::dictionary::VersionField for {} {{\n\n".format(field.name))

        file.write("    fn tag(&self) -> u32 {{ {} }}\n".format(field.id))
        file.write("    fn name(&self) -> &'static str {{ \"{}\" }}\n".format(field.name))
        file.write("    fn data_type(&self) -> &'static str {{ \"{}\" }}\n".format(field.type))
        file.write("    fn synopsis(&self) -> &'static str {{ \"{}\" }}\n".format(sanitise(field.synopsis)))
        
        file.write("    fn pedigree(&self) -> crate::dictionary::Pedigree {\n")
        file.write("        crate::dictionary::Pedigree {\n")
        file.write("            added: {},\n".format(format_pedigree(field.pedigree.added)))
        file.write("            added_ep: {},\n".format(format_pedigree(field.pedigree.addedEP)))
        file.write("            updated: {},\n".format(format_pedigree(field.pedigree.updated)))
        file.write("            updated_ep: {},\n".format(format_pedigree(field.pedigree.updatedEP)))
        file.write("            deprecated: {},\n".format(format_pedigree(field.pedigree.deprecated)))
        file.write("            deprecated_ep: {}\n".format(format_pedigree(field.pedigree.deprecatedEP)))
        file.write("        }\n")
        file.write("    }\n")
        
        file.write("    fn values(&self) -> &'static Vec<&'static crate::dictionary::FieldValue> {\n")
        file.write("        static VALUES: std::sync::OnceLock<Vec<&'static crate::dictionary::FieldValue>> = std::sync::OnceLock::new();\n")
        file.write("        VALUES.get_or_init(|| {\n")
        
        code_set = None
        try:
            code_set = orchestration.code_sets[field.type]
        except KeyError:
            # TODO - maybe check that its an expected built in type
            pass

        if code_set is None or len(code_set.codes) == 0:
            file.write("            vec![]\n")
        else:
            file.write("            vec![\n")
            for code in code_set.codes:
                file.write("                {}::{}(),\n".format(field.name, code.name))
            file.write("            ]\n")
        
        file.write("        })\n")
        file.write("    }\n\n")


        file.write("}\n\n")


    file.write("pub fn fields() -> &'static crate::dictionary::VersionFieldCollection {\n")
    file.write("    static FIELDS: std::sync::OnceLock<crate::dictionary::VersionFieldCollection> = std::sync::OnceLock::new();\n")
    file.write("    FIELDS.get_or_init(|| {\n")
    file.write("        crate::dictionary::VersionFieldCollection::new(\n")
    file.write("            vec![\n                ")

    index = 0
    offset = -1
    for field in sorted_fields:
        while index < int(field.id):
            file.write("0,")
            index += 1
            offset += 1
            if index % 20 == 0:
                file.write('\n                ')
        file.write("{},".format(index - offset))
        index += 1
        if index % 20 == 0:
            file.write('\n                ')


    file.write("\n            ],\n")
    file.write("            vec![\n")
    file.write("                Box::new(crate::dictionary::InvalidField{}),\n")
  
    index = 1
    for field in sorted_fields:
        file.write('                Box::new(crate::{}::{}{{}}),\n'.format(module, field.name))
        # Investigate why this is here, prob just optimisation to avoid a massive list
        # if index > 3000:
        #     break
        index += 1
                   
    file.write("            ]\n")
    file.write("       )")
    file.write("   })\n")
    file.write("}\n")





