"""Tiny Python and blueprint frontends. Sand remains the only gameplay compiler."""

def counter(initial_value=0, bounds=None):
    reference = {"kind": "internal", "id": "demo:increment"}
    program = {
        "format": "sand.program", "format_version": 1,
        "target": {"minecraft": "26.2"},
        "pack": {"namespace": "demo", "description": "Player counter"},
        "modules": [{"id": "demo:counter_module", "states": [{
            "id": "demo:counter", "scope": "player", "revision": 1,
            "fields": [{"name": "value", "default": initial_value}]}],
            "functions": [
                {"id": "demo:tick", "context": "server", "body": [{"action": {
                    "op": "players", "body": [{"action": {"op": "call", "function": reference}}]}}]},
                {"id": "demo:increment", "context": "player", "body": [{"action": {
                    "op": "score_add", "score": {"state": "demo:counter", "field": "value"}, "value": 1}}]}],
            "tags": [{"tag": "minecraft:tick", "function": {"kind": "internal", "id": "demo:tick"}}]}]}
    if bounds is not None:
        program["modules"][0]["states"][0]["fields"][0]["bounds"] = list(bounds)
    return program


def global_counter():
    """A shared bounded counter, usable by server functions without players."""
    program = counter(initial_value=10, bounds=(0, 100))
    program["pack"]["description"] = "Global counter"
    program["requires"] = ["global_state", "state_defaults", "state_bounds"]
    module = program["modules"][0]
    module["id"] = "demo:global_module"
    module["states"][0]["scope"] = "global"
    module["tags"] = []
    score = {"state": "demo:counter", "field": "value"}
    def write(op, value):
        return {"action": {"op": op, "score": score, "value": value}}
    # Even add(0) creates an absent Minecraft score. This catches load actions
    # accidentally running before the declared global default is installed.
    module["load"] = [write("score_add", 0)]
    bodies = {
        "increment": [write("score_add", 1)],
        "choose": [{"action": {"op": "branch", "condition": {"score": score, "min": 10, "max": 10},
                    "then": [write("score_add", 5)], "otherwise": [write("score_set", 1)]}}],
        "fill": [write("score_set", 200)],
        "subtract": [write("score_add", -3)],
    }
    module["functions"] = [{"id": f"demo:{name}", "context": "server", "body": body}
                           for name, body in bodies.items()]
    return program


def encounter_flows():
    """Independent player encounters and a shared world phase; no command text."""
    def field(state, name):
        return {"state": "demo:" + state, "field": name}
    def write(state, name, amount):
        return {"action": {"op": "score_add", "score": field(state, name), "value": amount}}
    def hook(phase, state, name, cadence=1):
        return {"phase": phase, "state": state, "function": {"kind": "internal", "id": "demo:" + name}, "cadence": cadence}
    def function(name, context, *body):
        return {"id": "demo:" + name, "context": context, "body": list(body)}
    program = counter()
    program["pack"]["description"] = "Player encounters and shared world flow"
    program["requires"] = ["state_flows", "global_state", "state_defaults", "state_bounds"]
    program["modules"] = [{"id": "demo:encounters", "states": [
        {"id": "demo:player", "scope": "player", "revision": 1, "fields": [
            {"name": "phase", "default": 0}, {"name": "energy", "default": 100, "bounds": [0, 100]}, {"name": "events", "default": 0}]},
        {"id": "demo:world", "scope": "global", "revision": 1, "fields": [
            {"name": "phase", "default": 0}, {"name": "ticks", "default": 0}]}
    ], "functions": [
        function("start", "player", write("player", "energy", -30), write("player", "events", 10)),
        function("active", "player", write("player", "events", 1)),
        function("pulse", "player", write("player", "energy", -5)),
        function("rest", "player", write("player", "events", 100)),
        function("world_tick", "server", write("world", "ticks", 1)),
    ], "flows": [
        {"id": "demo:encounter", "state": field("player", "phase"), "transitions": [
            {"from": 0, "to": 1, "when": {"score": field("player", "energy"), "min": 30}, "priority": 100},
            {"from": 0, "to": 2, "priority": 0},
            {"from": 1, "to": 2, "when": {"score": field("player", "energy"), "max": 10}}
        ], "hooks": [hook("enter", 1, "start"), hook("tick", 1, "active"), hook("tick", 1, "pulse", 2), hook("enter", 2, "rest")]},
        {"id": "demo:round", "state": field("world", "phase"), "transitions": [{"from": 0, "to": 1}],
         "hooks": [hook("tick", 1, "world_tick", 2)]}
    ]}]
    return program


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
