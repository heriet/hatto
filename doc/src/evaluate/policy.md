# policy

The policy is intended to be written by someone on your team or organization who is considering available licenses.

The policy file must implements `def evaluate(material, result)` written by python.

## evaluate

The `def evaluate(material, result)` is evaluate policy. If the evaluation fails, you call `result.add_error(message)` in this function.

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

This policy allows `Apache-2.0`, `BSD-3-Clause`, `MIT` and `Unlicense`.

The evaluate argment `material` is curated [Material](curation.md#material) by curation, and `result` is [EvaluateResult](#evaluateresult).

### evaluate using annotations

The Material set some kind of annotation, you can evaluate the policy more flexibly. For example, there may be cases where you want to ignore a license that is normally deny due to some special circumstances.

```python
def evaluate(material, result):
    if "ignore" in material.annotations:
        return

    # your policy check
```

In the above, when `ignore` is set on annotation, the evaluation is always ignored.

The rules for granting annotations are free, so you will need to decide the person who created the policy. For example, the above rule of ignore by `ignore` annotation should be in your policy document.

As another example, let's take a policy evaluate of the AGPL 3.0 license.

Suppose your team or organization may use AGPL 3.0 software in a service and you want to check whether the composed software is source distributed according to the AGPL 3.0 license.

For example, you can write the following.

```python
def evaluate(material, result):
    for license in material.licenses:
        if license.startswith("AGPL"):
            usage = material.annotations.get("usage", "unknown")

            if usage == "unknown":
                result.add_error(f"you must set usage for {license} software")
            elif usage == "service" and "project-source-distributed" not in material.annotations:
                result.add_error(f"you must project-source-distribute on {license} software")
```

`WARNING:` that the above policy does not check for compliance with all AGPL 3.0 license terms by legal perspective. For example, AGPL 3.0 also requires source code distribution when distributing a composed software even if it is not for use in a service.

The method of compliance with the terms of a particular license must be determined by your team or organization.

Even in the above example, your team or organization should decide under what conditions `usage` annotation and `project-source-distribute` will be granted and with what values.

## EvaluateResult

`EvaluateResult` is result of evaluate. If `EvaluateResult` contains any errors, hatto evaluate is failed. Conversely, `EvaluateResult` not contains errors, hatto evaluate is success.

It can also contain warnings. If the `EvaluateResult` contains any warnings, hatto evaluate is not failed.

### Methods

#### EvaluateResult.add_errors(message)

The `add_errors` is add error to result.

```python
def evaluate(material, result):
    result.add_error("Any licences is not allowed")
```

```sh
$ hatto evaluate --policy=policy.py example.tsv
NG foo 1.0.0 licenses:["MIT"] annotations:{}
  ERROR Any licences is not allowed
Failure: evaluate failed
```

#### EvaluateResult.add_warnings(message)

The `add_warnings` is add warning to result.

```python
def evaluate(material, result):
    result.add_warning("Any licences is warning")
```

```sh
$ hatto evaluate --policy=policy.py example.tsv
OK foo 1.0.0 licenses:["MIT"] annotations:{}
    WARNING Any licences is warning
```

## default policy

If the `--policy` option is not set, the following default policy is executed.

The default policy allows at least what is commonly used in some Permissive License. This policy may change in future versions. So, we recommend that it be not use as possible.

The policy for your team or organization should be written by your team or organization.

```python
#!/usr/bin/python

allowed_licenses = [
    "Apache-2.0",
    "MIT",
    "BSD-3-Clause",
    "Unlicense",
]

def evaluate(material, result):
    for license in material.licenses:
        if license not in allowed_licenses:
           result.add_error(f"{license} is not allowed")
```

## tests

The policy is python code. So, it can be test in common way of python tests.

**example `test_polocy.py`**

```python
from policy import evaluate

import pytest


class Material:
    def __init__(self, name="", version="", licenses=[], annotations={}):
        self.name = name
        self.version = version
        self.licenses = licenses
        self.annotations = annotations

    def update_annotation(self, key, value):
        self.annotations[key] = value


class EvaluateResult:
    def __init__(self):
        self.errors = []
        self.warnings = []

    def add_error(self, message):
        self.errors.append(message)

    def add_warning(self, message):
        self.warnings.append(message)


@pytest.mark.parameterize("license_name", ["Apache-2.0", "BSD-3-Clause", "MIT", "Unlicense"])
def test_evaluate_allow(license_name):
    result = EvaluateResult()
    material = Material("foo", "v0.1.0", [license_name])
    evaluate()

    assert len(result.errors) == 0

@pytest.mark.parameterize("license_name", ["UNKNOWN"])
def test_evaluate_deny(license_name):
    result = EvaluateResult()
    material = Material("foo", "v0.1.0", [license_name])
    evaluate()

    assert len(result.errors) == 1
```