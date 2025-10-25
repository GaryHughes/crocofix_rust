#!/usr/bin/python3

from sanitise import *

def generate_orchestration(file, orchestration, module):
    
    file.write("\npub struct Orchestration {\n")
    file.write("}\n\n")

    file.write("impl crate::dictionary::Orchestration for Orchestration {\n")
    file.write("    fn name(&self) -> &'static str {{ \"{}\" }}\n".format(module))
    file.write("    fn fields(&self) -> &'static crate::dictionary::OrchestrationFieldCollection {{ crate::{}::fields() }}\n".format(module))
    file.write("    fn messages(&self) -> &'static crate::dictionary::MessageCollection {{ crate::{}::messages() }}\n".format(module))
    file.write("}\n\n")
               
    file.write("pub fn orchestration() -> &'static crate::{}::Orchestration {{\n".format(module))
    file.write("    static ORCHESTRATION: std::sync::OnceLock<crate::{}::Orchestration> = std::sync::OnceLock::new();\n".format(module))
    file.write("    ORCHESTRATION.get_or_init(|| {{ crate::{}::Orchestration{{}} }})\n".format(module))
    file.write("}\n\n")