#!/usr/bin/python

def evaluate(material, result):
    if material.name == "foo":
      result.add_error("MyError: my error")

