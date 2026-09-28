"""Tiny Python and blueprint frontends. Sand remains the only gameplay compiler."""

def counter():
    reference = {"kind": "internal", "id": "demo:increment"}
    return {
        "format": "sand.program", "format_version": 1,
        "target": {"minecraft": "26.2"},
        "pack": {"namespace": "demo", "description": "Player counter"},
        "modules": [{"id": "demo:counter_module", "states": [{
            "id": "demo:counter", "scope": "player", "revision": 1,
            "fields": [{"name": "value", "default": 0}]}],
            "functions": [
                {"id": "demo:tick", "context": "server", "body": [{"action": {
                    "op": "players", "body": [{"action": {"op": "call", "function": reference}}]}}]},
                {"id": "demo:increment", "context": "player", "body": [{"action": {
                    "op": "score_add", "score": {"state": "demo:counter", "field": "value"}, "value": 1}}]}],
            "tags": [{"tag": "minecraft:tick", "function": {"kind": "internal", "id": "demo:tick"}}]}]}


def blueprint(graph):
    """A deliberately small execution graph: tick -> players -> increment.

    One ordered outgoing execution edge per node, with node/port origins retained.
    Layout is editor-only and never enters the semantic program.
    """
    nodes = {node["id"]: node for node in graph["nodes"]}
    if len(nodes) != len(graph["nodes"]):
        raise ValueError("duplicate node identity")
    edges = {}
    for edge in graph["edges"]:
        source, target = edge["from"], edge["to"]
        if source not in nodes or target not in nodes or source in edges:
            raise ValueError("ambiguous or unresolved execution edge")
        edges[source] = target
    roots = [key for key, node in nodes.items() if node["kind"] == "tick"]
    if len(roots) != 1:
        raise ValueError("one tick entry is required")
    order, current = [], roots[0]
    while current is not None:
        if current in order:
            raise ValueError("unsupported execution cycle")
        order.append(current)
        current = edges.get(current)
    if len(order) != len(nodes) or [nodes[key]["kind"] for key in order] != ["tick", "players", "increment"]:
        raise ValueError("unsupported blueprint execution graph")
    program = counter()
    functions = program["modules"][0]["functions"]
    functions[0]["body"][0]["origin"] = {"node": order[1], "port": "exec"}
    functions[0]["body"][0]["action"]["body"][0]["origin"] = {"node": order[2], "port": "exec"}
    functions[1]["body"][0]["origin"] = {"node": order[2], "port": "value"}
    return program


if __name__ == "__main__":
    import json
    import sys
    json.dump(counter(), sys.stdout, indent=2)
    sys.stdout.write("\n")
