#!/usr/bin/python

def curate_material(material):
    if material.name == "bar":
      material.update_annotation("usage", "service")
