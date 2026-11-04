"""Version-specific passive-client packet IDs extracted from official server jars.

Regenerate JSON with extract_protocol_metadata.py; never derive a newer protocol
by shifting older IDs. Active scripted controls retain their separate 776 guard.
"""
import json
from pathlib import Path

PROFILES = {}
for version in ("26.2", "26.3"):
    metadata = json.loads((Path(__file__).with_name("protocol_metadata") / f"{version}.json").read_text())
    PROFILES[metadata["version"]["protocol_version"]] = metadata


def profile_for_protocol(protocol):
    try:
        return PROFILES[protocol]
    except KeyError as error:
        raise ValueError("passive audit client supports Minecraft 26.2 / protocol 776 and 26.3 / protocol 777 only") from error


def profile_for_version(version):
    for profile in PROFILES.values():
        if profile["version"]["id"] == version:
            return profile
    raise ValueError("passive player harness supports Minecraft 26.2 and 26.3 only")
