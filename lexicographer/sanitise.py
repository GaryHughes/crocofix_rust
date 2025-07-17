#!/usr/bin/python3

def sanitise(string):
    if string is None:
        return ''
    return string.replace('\n', '').replace('"', '\'').encode("ascii", errors="ignore").decode()

def format_pedigree(pedigree):
    if pedigree is None:
        return 'None'
    return 'Some("{}".to_string())'.format(pedigree)

def format_presence(presence):
    return "crate::dictionary::Presence::" + presence[0].upper() + presence[1:]
