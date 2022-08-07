# evaluate

`hatto evaluate` is evaluate license policy.

```sh
$ hatto evaluate --help
hatto-evaluate
evaluate policy

USAGE:
    hatto evaluate [OPTIONS] <FILE>

ARGS:
    <FILE>

OPTIONS:
    -c, --curation <FILE>
    -h, --help                         Print help information
    -o, --output <OUTPUT_FORMAT>       [default: human] [possible values: human, json]
    -p, --policy <FILE>
    -t, --source-type <SOURCE_TYPE>    [possible values: tsv, spdx-tag, spdx-json, spdx-yaml,
                                       cyclone-dx-json, cyclone-dx-xml]
```

The evaluate ARGS file is SBOM or tsv. SBOM supports `SPDX` or `CycloneDX`.

Yet another hatto supports tsv. This tsv file must contain header.

**example `example.tsv`**

```tsv
name	version	licenses	annotations
foo	1.0.1	MIT,Apache-2.0	usage=service
bar	1.1.2	UNKNOWN	
```

These files can generate with any license collection tool. If the license collection tool does not support SBOM, you shoud convert to tsv or SBOM.

And you can configure `--policy` and `--curation`.

The `--policy` file defines license policy that written in python. The policy file must implements `def evaluate(material, result)`.

**example `polocy.py`**

```python
#!/usr/bin/python

allowed_licenses = [
    "Apache-2.0",
    "BSD-3-Clause",
    "MIT",
    "Unlicense",
]

def evaluate(material, result):
    for license in material.licenses:
        if license not in allowed_licenses:
           result.add_error(f"{license} is not allowed")
```

```sh
$ hatto evaluate --policy policy.py example.tsv
OK foo 1.0.1 licenses:["MIT", "Apache-2.0"] annotations:{"usage": "service"}
NG bar 1.1.2 licenses:["UNKNOWN"] annotations:{}
  ERROR UNKNOWN is not allowed
Failure: evaluate failed
```

`UNKNOWN` is not allowed on `policy.py`. Therefore `hatto evaluate` is failed.

You may know `bar` true license is `BSD-3-Clause`. In such a case you can patch license information by `--curation` file. The curation file must implement `def curate_material(material)`.

**example `curation.py`**

```python
#!/usr/bin/python

def curate_material(material):
    if material.name == "bar":
      material.licenses = ["BSD-3-Clause"]
```

```sh
$ hatto evaluate --policy policy.py --curation curation.py example.tsv
OK foo 1.0.1 licenses:["MIT", "Apache-2.0"] annotations:{"usage": "service"}
OK bar 1.1.2 licenses:["BSD-3-Clause"] annotations:{}
```

These allow hatto to perform flexible license policy evaluation on your teams or organizations.
