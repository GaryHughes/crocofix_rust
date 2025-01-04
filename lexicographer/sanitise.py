#!/usr/bin/python3

def sanitise(string):
    if string is None:
        return ''
    return string.replace('\n', '').replace('"', '\'').encode("ascii", errors="ignore").decode()

