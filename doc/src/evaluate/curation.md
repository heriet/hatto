# curation

The role of curation is to correct license information. License information collected by license information collection tools is rarely inaccurate and may need to be corrected manually.

The curation is intended to be written by the project owner. The project owner and the person that determining the organization's licensing policy may be different.

The curation file must implements `def curate_material(material)` written by python.

## curate_material

The `def curate_material(material)` is curate material. In this context, curete means to modify the material and license information.

For example, suppose the license of foo package detected MIT by license collection tool. However, there was an error in the collection process and the true license is Apache 2.0.

You can correct this error using `curate_material`.

```python
#!/usr/bin/python

def curate_material(material):
    if material.name == "foo":
      material.licenses = ["Apache-2.0"]
```

In another case, suppose you had many package that dual licensed MIT and Apache 2.0 in your project.

For reasons of your organizational policy, you may want to specify which license to use.

```python
#!/usr/bin/python

def curate_material(material):
    if set(material.licenses) == set(["MIT", "Apache-2.0"]):
      material.licenses = ["MIT"]
```


## Material

### Instance Variables

|name|type|explain|
|---|---|---|
|name|string|name of material|
|version|string|version of material|
|licenses|list|list of license name(string). In most cases, license name is expected to specify SPDX license identifier |
|annotations|dict|dict of annotation key(string) to value(string)|

### Methods

#### Material.update_annotation(key, value)

The `update_annotation` is update annotation by key-value pair.

`WARNING:` Currently, it is possible to update annotations dict, but cannot update by annotations dict key. Therefore, this method is temporary provided. This method may be removed in the future.

```python
#!/usr/bin/python

def curate_material(material):
    material.annotations["foo"] = "bar" # not updated
    material.annotations |= {"hoge": "fuga"} # python 3.9 later
    material.update_annotation("x", "y")

    print(material.annotations) # {"hoge": "fuga", "x": "y"}
```

