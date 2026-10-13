"""Loopback-only configuration for disposable legacy RCON audit servers."""
from pathlib import Path


def write_properties(server_dir: Path, server_port: int, rcon_port: int, password: str) -> None:
    """Write vanilla's required local credential inside the private temp directory.

    Vanilla uses server-ip for both game and RCON listeners. A localhost client
    alone does not constrain those listeners: an omitted address binds all
    interfaces. The scripts generate a fresh password for each process.
    """
    path = server_dir / "server.properties"
    path.write_text("\n".join([
        "level-name=world",
        "server-ip=127.0.0.1",
        "online-mode=false",
        "max-players=4",
        "enable-rcon=true",
        f"rcon.password={password}",
        f"rcon.port={rcon_port}",
        f"server-port={server_port}",
        "spawn-npcs=false",
        "spawn-animals=false",
        "spawn-monsters=false",
        "generate-structures=false",
        "view-distance=4",
        "simulation-distance=4",
        "sync-chunk-writes=false",
        "",
    ]), encoding="utf-8")
    path.chmod(0o600)
