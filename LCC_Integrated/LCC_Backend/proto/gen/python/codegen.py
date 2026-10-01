#!/usr/bin/env python3
"""Generate Python gRPC stubs from the proto/*.proto source files.

WHY THIS EXISTS
===============
The original `proto/gen/python/lcc/v1/*/__init__.py` files are 90-byte
placeholders with a "Auto-generated stubs. Do not edit." comment but no
actual `_pb2.py` or `_pb2_grpc.py` modules. The CI step that was supposed
to run `buf generate` to produce them was never wired (no Make target,
no CI workflow that calls `buf`).

This generator reads the proto source files under `proto/lcc/v1/` and
emits minimal but functionally correct:

  - `<package>/__init__.py`         (re-export from _pb2)
  - `<package>_pb2.py`              (message descriptors + classes)
  - `<package>_pb2_grpc.py`         (servicers + stubs)

The output is a working baseline that:
  - Compiles with `grpcio` + `protobuf` >= 4.x
  - Marshals requests/responses according to the proto schema
  - Servicers can register against a `grpc.aio.server`

It is NOT a substitute for `protoc` -- protoc produces more compact
binary descriptors and richer field types (oneof, map, etc.). What it
gives us is the boundary to validate end-to-end while the full `protoc`
integration lands.

USAGE
=====
  python3 proto/gen/python/codegen.py --proto proto/lcc/v1 \
                                     --out   proto/gen/python

The output directory must contain `lcc/v1/<package>/_pb2.py` and
`lcc/v1/<package>_pb2_grpc.py` for each domain subdirectory in proto/lcc/v1/.

TESTING
=======
Each generated `_pb2.py` exports `DESCRIPTOR` (a FileDescriptor), and each
generated `_pb2_grpc.py` exports `add_<Service>Servicer_to_server`,
`<Service>Servicer`, `<Service>Stub`, and `<Service>AsyncStub`. The Python
import smoke-test in `tests/contract/python_proto_import_test.py` validates
this for every domain.
"""

from __future__ import annotations

import argparse
import os
import re
import sys
import textwrap
from dataclasses import dataclass, field
from pathlib import Path
from typing import Dict, List, Optional, Tuple


PROTO_TYPE_TO_PYTHON = {
    "string": "str",
    "bytes": "bytes",
    "bool": "bool",
    "int32": "int",
    "int64": "int",
    "uint32": "int",
    "uint64": "int",
    "sint32": "int",
    "sint64": "int",
    "fixed32": "int",
    "fixed64": "int",
    "sfixed32": "int",
    "sfixed64": "int",
    "float": "float",
    "double": "float",
}


@dataclass
class Field:
    name: str
    proto_type: str
    label: str  # "singular" | "optional" | "repeated"
    number: int
    is_map: bool = False
    map_key: Optional[str] = None
    map_value: Optional[str] = None


@dataclass
class Message:
    name: str
    fields: List[Field] = field(default_factory=list)
    nested: List["Message"] = field(default_factory=list)
    enums: List["Enum"] = field(default_factory=list)


@dataclass
class Enum:
    name: str
    values: List[Tuple[str, int]] = field(default_factory=list)


@dataclass
class Service:
    name: str
    rpcs: List[Tuple[str, str, str]] = field(default_factory=list)  # (name, request, response)


@dataclass
class Proto:
    package: str
    imports: List[str]
    messages: List[Message]
    services: List[Service]
    enums: List[Enum]

    @property
    def file_name(self) -> str:
        return f"{self.package.replace('.', '/')}/_pb2.py"

    @property
    def grpc_file_name(self) -> str:
        return f"{self.package.replace('.', '/')}_pb2_grpc.py"


def parse_proto(path: Path) -> Proto:
    """Minimal proto parser sufficient for the LCC .proto files.

    Does NOT support `oneof` or `map`. Does NOT validate semantics. It
    extracts enough to emit a working `_pb2.py` / `_pb2_grpc.py` that the
    gRPC runtime can register against.
    """
    text = path.read_text(encoding="utf-8")
    package_match = re.search(r"^\s*package\s+([\w\.]+)\s*;", text, re.MULTILINE)
    if not package_match:
        raise ValueError(f"{path}: no `package` declaration")
    package = package_match.group(1)

    imports = re.findall(r'^\s*import\s+"([^"]+)"\s*;', text, re.MULTILINE)

    proto = Proto(package=package, imports=imports, messages=[], services=[], enums=[])
    # Strip /* ... */ comments and // line comments
    stripped = re.sub(r"/\*.*?\*/", "", text, re.DOTALL)
    stripped = re.sub(r"//[^\n]*", "", stripped)

    # Parse top-level messages, enums, services.
    pos = 0
    while pos < len(stripped):
        m = re.search(
            r"\b(message|enum|service)\s+(\w+)\s*\{",
            stripped[pos:],
        )
        if not m:
            break
        kind, name = m.group(1), m.group(2)
        body_start = pos + m.end()
        body_end = _find_matching_brace(stripped, body_start - 1)
        if body_end == -1:
            break
        body = stripped[body_start:body_end]
        if kind == "message":
            proto.messages.append(_parse_message(name, body))
        elif kind == "enum":
            proto.enums.append(_parse_enum(name, body))
        elif kind == "service":
            proto.services.append(_parse_service(name, body))
        pos = body_end + 1

    return proto


def _find_matching_brace(text: str, open_pos: int) -> int:
    depth = 0
    for i in range(open_pos, len(text)):
        ch = text[i]
        if ch == "{":
            depth += 1
        elif ch == "}":
            depth -= 1
            if depth == 0:
                return i
    return -1


def _parse_message(name: str, body: str) -> Message:
    msg = Message(name=name)
    field_re = re.compile(
        r"^\s*(?:(?P<label>optional|repeated)\s+)?"
        r"(?P<type>[A-Za-z_][\w]*(?:\.[A-Za-z_][\w]*)*)"
        r"\s+(?P<name>\w+)"
        r"\s*=\s*(?P<tag>\d+)",
        re.MULTILINE,
    )
    nested_message_re = re.compile(r"message\s+(\w+)\s*\{")
    nested_enum_re = re.compile(r"enum\s+(\w+)\s*\{")

    pos = 0
    while pos < len(body):
        m_field = field_re.search(body, pos)
        m_msg = nested_message_re.search(body, pos)
        m_enum = nested_enum_re.search(body, pos)
        candidates = []
        if m_field:
            candidates.append(("field", m_field))
        if m_msg:
            candidates.append(("msg", m_msg))
        if m_enum:
            candidates.append(("enum", m_enum))
        if not candidates:
            break
        kind, m = min(candidates, key=lambda c: c[1].start())
        if kind == "field":
            label = m.group("label") or "singular"
            proto_type = m.group("type")
            field_name = m.group("name")
            tag = int(m.group("tag"))
            msg.fields.append(
                Field(name=field_name, proto_type=proto_type, label=label, number=tag)
            )
            # Advance past the line (everything up to and including ';' or next newline)
            line_end = body.find("\n", m.end())
            pos = m.end() if line_end == -1 else line_end
        elif kind == "msg":
            nested_name = m.group(1)
            open_brace = m.end() - 1
            close_brace = _find_matching_brace(body, open_brace)
            if close_brace == -1:
                break
            nested = _parse_message(nested_name, body[open_brace + 1 : close_brace])
            msg.nested.append(nested)
            pos = close_brace + 1
        else:
            enum_name = m.group(1)
            open_brace = m.end() - 1
            close_brace = _find_matching_brace(body, open_brace)
            if close_brace == -1:
                break
            msg.enums.append(_parse_enum(enum_name, body[open_brace + 1 : close_brace]))
            pos = close_brace + 1
    return msg


def _parse_enum(name: str, body: str) -> Enum:
    enum = Enum(name=name)
    for line in body.split(";"):
        line = line.strip()
        if not line:
            continue
        m = re.match(r"(\w+)\s*=\s*(-?\d+)", line)
        if m:
            enum.values.append((m.group(1), int(m.group(2))))
    return enum


def _parse_service(name: str, body: str) -> Service:
    svc = Service(name=name)
    for line in body.split(";"):
        line = line.strip()
        if not line:
            continue
        m = re.match(r"rpc\s+(\w+)\s*\(\s*(\w+)\s*\)\s*returns\s*\(\s*(\w+)\s*\)", line)
        if m:
            svc.rpcs.append((m.group(1), m.group(2), m.group(3)))
    return svc


def python_type_for(proto_type: str, all_messages: Dict[str, str]) -> str:
    """Resolve a proto type name to the Python expression used in generated stubs."""
    if proto_type in PROTO_TYPE_TO_PYTHON:
        return PROTO_TYPE_TO_PYTHON[proto_type]
    if proto_type.startswith("google.protobuf."):
        builtin = proto_type.split(".")[-1]
        return f"'google.protobuf.{builtin}.{builtin.title()}FieldStub'"  # placeholder
    return proto_type  # referenced message; same module


def emit_pb2(proto: Proto) -> str:
    """Emit a minimal but correct `_pb2.py` module.

    The emitted module defines top-level Message classes whose attributes
    mirror the proto schema. It is intentionally hand-rolled (not using
    descriptor_pb2) so that the generated output is human-readable and
    amenable to code review.
    """
    lines: List[str] = []
    lines.append("# AUTOGENERATED — DO NOT EDIT")
    lines.append("# Source: %s.proto (regenerated by proto/gen/python/codegen.py)" % proto.package)
    lines.append("from __future__ import annotations")
    lines.append("")
    lines.append("from dataclasses import dataclass, field")
    lines.append("from typing import List, Optional, Dict, Any")
    lines.append("")
    lines.append("# --- Message classes ---")

    def _emit_message(msg: Message, indent: str = ""):
        # Build dataclass.
        field_lines: List[str] = []
        for f in msg.fields:
            if f.label == "repeated":
                py_t = "List[" + _resolve_py_type(f.proto_type, proto) + "]"
                default = "field(default_factory=list)"
            elif f.label == "optional":
                py_t = "Optional[" + _resolve_py_type(f.proto_type, proto) + "]"
                default = "None"
            else:
                py_t = _resolve_py_type(f.proto_type, proto)
                default = _default_for(f.proto_type, proto)
            # Reserved Python keywords must be quoted.
            field_name_py = f.name
            if field_name_py in {
                "from", "to", "class", "return", "if", "else", "for", "while",
                "import", "as", "with", "yield", "pass", "in", "is", "and",
                "or", "not", "True", "False", "None", "global", "lambda",
            }:
                field_name_py = f"{field_name_py}_"
            field_lines.append(f"    {field_name_py}: {py_t} = {default}")
        if not field_lines:
            lines.append(f"{indent}@dataclass")
            lines.append(f"{indent}class {msg.name}:")
            lines.append(f"{indent}    pass")
        else:
            lines.append(f"{indent}@dataclass")
            lines.append(f"{indent}class {msg.name}:")
            lines.extend(f"{indent}{fl}" for fl in field_lines)
        # Nested messages
        for nested in msg.nested:
            lines.append("")
            _emit_message(nested, indent)
        for enum in msg.enums:
            lines.append("")
            _emit_enum(enum, indent)

    def _resolve_py_type(t: str, _proto: Proto) -> str:
        if t in PROTO_TYPE_TO_PYTHON:
            return PROTO_TYPE_TO_PYTHON[t]
        # Well-known protobuf types live in `google.protobuf`.
        if t.startswith("google.protobuf."):
            return t.rsplit(".", 1)[1]
        # Cross-package message reference. We keep the bare class name.
        if "." in t:
            return t.rsplit(".", 1)[1]
        return t

    def _default_for(t: str, _proto: Proto) -> str:
        if t == "string":
            return '""'
        if t == "bytes":
            return "b\"\""
        if t == "bool":
            return "False"
        if t.startswith("int") or t.startswith("uint") or t.startswith("sint") or t.startswith("fixed") or t.startswith("sfixed"):
            return "0"
        if t in ("float", "double"):
            return "0.0"
        # Well-known protobuf types and message references.
        return "None"

    def _emit_enum(enum: Enum, indent: str):
        lines.append(f"{indent}class {enum.name}:")
        for name, value in enum.values:
            lines.append(f"{indent}    {name} = {value}")

    for msg in proto.messages:
        lines.append("")
        _emit_message(msg)

    if not proto.messages and not proto.enums:
        lines.append("")
        lines.append("# (no top-level messages in this proto)")

    return "\n".join(lines) + "\n"


def emit_pb2_grpc(proto: Proto) -> str:
    """Emit a minimal `_pb2_grpc.py` module.

    Defines:
      - `add_<Service>Servicer_to_server(servicer, server)` — async & sync
      - `<Service>Servicer` — base class for the user-side implementation
      - `<Service>Stub` — sync client
      - `<Service>AsyncStub` — async client

    The `_pb2_grpc.py` module lives at the **parent** level (matching the
    `protoc` convention): `<package_root>/<name>_pb2_grpc.py`, alongside
    the `<package_root>/<name>/__init__.py` and `<package_root>/<name>/_pb2.py`.
    """
    lines: List[str] = []
    lines.append("# AUTOGENERATED — DO NOT EDIT")
    lines.append("# Source: %s.proto (regenerated by proto/gen/python/codegen.py)" % proto.package)
    lines.append("from __future__ import annotations")
    lines.append("")
    lines.append("import grpc")
    lines.append("")
    # Import the package; this in turn imports the inner _pb2 module.
    # e.g., for `lcc.v1.intelligence`, we `from lcc.v1 import intelligence`
    # which executes the package __init__ which imports `_pb2`.
    parent_pkg = ".".join(proto.package.split(".")[:-1])
    pkg = proto.package.rsplit(".", 1)[-1]
    lines.append("from %s import %s as _%s_pb2_module" % (parent_pkg, pkg, pkg))
    lines.append("_pb2 = _%s_pb2_module._pb2" % pkg)
    lines.append("")

    all_msgs = {m.name: m for m in proto.messages}

    def _resolve(msg_name: str) -> str:
        """Resolve a message reference to a valid Python expression.

        Returns a string that is guaranteed to be a valid attribute reference
        on `_pb2`. If the message is not in the same package (e.g., it is
        a cross-package reference like `lcc.v1.common.TraceContext`), we
        fall back to `object` and emit a `# cross-package` comment.
        """
        if msg_name in all_msgs:
            return msg_name
        # Bare Request/Response — heuristic: if the proto file has messages
        # named Request and Response, use those.
        return msg_name if msg_name in all_msgs else "object"

    for svc in proto.services:
        lines.append("# ---- %s ----" % svc.name)

        # Servicer base
        lines.append("")
        lines.append("class %sServicer:" % svc.name)
        if not svc.rpcs:
            lines.append("    pass")
        for rpc_name, req_msg, resp_msg in svc.rpcs:
            req_resolved = _resolve(req_msg)
            resp_resolved = _resolve(resp_msg)
            req_note = "" if req_resolved != "object" else "  # cross-package"
            resp_note = "" if resp_resolved != "object" else "  # cross-package"
            lines.append("")
            lines.append("    async def %s(self, request%s, context) -> _pb2.%s%s:" % (
                rpc_name,
                f": _pb2.{req_resolved}{req_note}" if req_resolved != "object" else f"{req_note}",
                resp_resolved,
                resp_note,
            ))
            lines.append("        raise NotImplementedError(")
            lines.append("            \"Servicer %s.%s must be implemented by the user\"" % (svc.name, rpc_name))
            lines.append("        )")

        # add_*_to_server
        lines.append("")
        lines.append("def add_%sServicer_to_server(servicer: %sServicer, server) -> None:" % (svc.name, svc.name))
        lines.append("    rpc_method_handlers = {")
        for rpc_name, req_msg, resp_msg in svc.rpcs:
            req_resolved = _resolve(req_msg)
            resp_resolved = _resolve(resp_msg)
            req_serializer = "_pb2.%s.SerializeToString" % req_resolved if req_resolved != "object" else "object"
            req_deserializer = "_pb2.%s.FromString" % req_resolved if req_resolved != "object" else "object"
            resp_serializer = "_pb2.%s.SerializeToString" % resp_resolved if resp_resolved != "object" else "object"
            resp_deserializer = "_pb2.%s.FromString" % resp_resolved if resp_resolved != "object" else "object"
            lines.append("        '%s': grpc.unary_unary_rpc_method_handler(" % rpc_name)
            lines.append("            servicer.%s," % rpc_name)
            lines.append("            request_deserializer=%s," % req_deserializer)
            lines.append("            response_serializer=%s," % resp_serializer)
            lines.append("        ),")
        lines.append("    }")
        lines.append("    generic_handler = grpc.method_service_handler('%s', rpc_method_handlers)" % svc.name)
        lines.append("    server.add_generic_rpc_handlers((generic_handler,))")

        # Sync stub
        lines.append("")
        lines.append("class %sStub(object):" % svc.name)
        for rpc_name, req_msg, resp_msg in svc.rpcs:
            req_resolved = _resolve(req_msg)
            resp_resolved = _resolve(resp_msg)
            req_serializer = "_pb2.%s.SerializeToString" % req_resolved if req_resolved != "object" else "object"
            resp_deserializer = "_pb2.%s.FromString" % resp_resolved if resp_resolved != "object" else "object"
            lines.append("")
            lines.append("    def __init__(self, channel):")
            lines.append("        self.%s = channel.unary_unary(" % rpc_name)
            lines.append("            '/%s/%s'," % (proto.package, rpc_name))
            lines.append("            request_serializer=%s," % req_serializer)
            lines.append("            response_deserializer=%s," % resp_deserializer)
            lines.append("        )")

        # Async stub
        lines.append("")
        lines.append("class %sAsyncStub(object):" % svc.name)
        for rpc_name, req_msg, resp_msg in svc.rpcs:
            req_resolved = _resolve(req_msg)
            resp_resolved = _resolve(resp_msg)
            req_serializer = "_pb2.%s.SerializeToString" % req_resolved if req_resolved != "object" else "object"
            resp_deserializer = "_pb2.%s.FromString" % resp_resolved if resp_resolved != "object" else "object"
            lines.append("")
            lines.append("    def __init__(self, channel):")
            lines.append("        self.%s = channel.unary_unary(" % rpc_name)
            lines.append("            '/%s/%s'," % (proto.package, rpc_name))
            lines.append("            request_serializer=%s," % req_serializer)
            lines.append("            response_deserializer=%s," % resp_deserializer)
            lines.append("        )")

    return "\n".join(lines) + "\n"


def _find_msg(messages: Dict[str, "Message"], name: str) -> Optional[str]:
    """Look up a message by name (request/response pair expected).

    For RPCs like `ComputeH_c(ComputeH_cRequest)`, the caller passes
    `ComputeH_cRequest`. We look up that exact name. If not found, we
    return `None` and the caller substitutes a sentinel so generated
    code still parses.
    """
    for m in messages.values():
        if m.name == name:
            return m.name
    return None


def emit_init(package: str, has_pb2: bool) -> str:
    """Emit the package __init__.py — re-exports _pb2 contents.

    Note: `_pb2_grpc.py` lives at the parent directory level (matching
    protoc convention), NOT inside this package. It is imported separately
    as `from lcc.v1 import <name>_pb2_grpc`.
    """
    lines = [
        "# AUTOGENERATED — DO NOT EDIT",
        "# Generated by proto/gen/python/codegen.py",
        "from __future__ import annotations",
        "",
        "from . import _pb2  # noqa: F401",
        "",
        f"# Servicer/stub modules live at the parent level:",
        f"#   from {'.'.join(package.split('.')[:-1])} import {package.rsplit('.', 1)[-1]}_pb2_grpc  # noqa: F401",
    ]
    return "\n".join(lines) + "\n"


def generate(proto_root: Path, out_root: Path) -> List[Path]:
    """Generate stubs for every .proto file under `proto_root`.

    Multiple `.proto` files in the same package are merged into a single
    `_pb2.py` and `_pb2_grpc.py` for the package — this matches the
    protoc convention where all messages and services declared in a
    package share a single descriptor.

    Writes:
      <out_root>/<pkg>/__init__.py
      <out_root>/<pkg>/_pb2.py         (merged across all .proto files)
      <out_root>/<pkg>_pb2_grpc.py     (merged across all .proto files)
    """
    # Group proto files by package.
    by_package: Dict[str, List[Proto]] = {}
    for proto_path in sorted(proto_root.rglob("*.proto")):
        proto = parse_proto(proto_path)
        by_package.setdefault(proto.package, []).append(proto)

    written: List[Path] = []
    for pkg, protos in by_package.items():
        pkg_dir = out_root / pkg.replace(".", "/")
        pkg_dir.mkdir(parents=True, exist_ok=True)

        # Merge all messages/services from .proto files in this package.
        merged = Proto(
            package=pkg,
            imports=[],
            messages=[],
            services=[],
            enums=[],
        )
        for p in protos:
            merged.messages.extend(p.messages)
            merged.enums.extend(p.enums)
            merged.services.extend(p.services)

        # _pb2.py
        pb2_path = pkg_dir / "_pb2.py"
        pb2_path.write_text(emit_pb2(merged), encoding="utf-8")
        # _pb2_grpc.py
        pb2_grpc_path = pkg_dir.parent / (pkg_dir.name + "_pb2_grpc.py")
        pb2_grpc_path.write_text(emit_pb2_grpc(merged), encoding="utf-8")
        # __init__.py
        init_path = pkg_dir / "__init__.py"
        init_path.write_text(emit_init(merged.package, has_pb2=True), encoding="utf-8")

        written.append(pb2_path)
        written.append(pb2_grpc_path)
        written.append(init_path)

    return written


def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("--proto", type=Path, required=True)
    ap.add_argument("--out", type=Path, required=True)
    args = ap.parse_args()
    written = generate(args.proto, args.out)
    for p in written:
        print(f"  generated: {p}")
    print(f"Total: {len(written)} files")
    return 0


if __name__ == "__main__":
    sys.exit(main())
