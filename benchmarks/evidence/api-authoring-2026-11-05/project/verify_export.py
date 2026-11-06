from pathlib import Path
import json
root=Path(__file__).resolve().parent/'dist/vanilla_expedition'
base=root/'data/vanilla_expedition'
files=list(root.rglob('*'))
for p in files:
 if p.suffix=='.json' or p.name=='pack.mcmeta': json.loads(p.read_text())
expected=['advancement/frontier.json','recipe/field_compass.json','loot_table/expedition_cache.json','predicate/in_frontier.json','item_modifier/chartered.json','dialog/journal.json','tags/item/supplies.json','enchantment/pathfinder.json','dimension/sanctuary.json','dimension_type/sanctuary.json','worldgen/structure_set/expedition_ruins.json']
for path in expected: assert (base/path).is_file(), path
recipe=json.loads((base/'recipe/field_compass.json').read_text())
assert 'vanilla_expedition:field_compass' in json.dumps(recipe)
commands='\n'.join(p.read_text() for p in base.rglob('*.mcfunction'))
for fragment in ['schedule function vanilla_expedition:season_pulse 10s replace','bossbar set vanilla_expedition:season players @a','text_display','Pathfinder Blade','item modify entity @s weapon.mainhand vanilla_expedition:chartered','if predicate vanilla_expedition:in_frontier','function vanilla_expedition:veteran_reward']:
 assert fragment in commands, fragment
assert 'scoreboard players add @s ' in commands
assert 'setblock ~ ~ ~ minecraft:campfire' in commands
load=json.loads((root/'data/minecraft/tags/function/load.json').read_text())
assert load['values']
import re
missing=[]
for p in base.rglob('*.mcfunction'):
 for target in re.findall(r'(?:^| )function (vanilla_expedition:[a-z0-9_./-]+)',p.read_text(),re.M):
  target_path=base/'function'/(target.split(':',1)[1]+'.mcfunction')
  if not target_path.exists(): missing.append((str(p),target))
assert not missing, missing
print('PASS: all JSON parse, 11 feature resources, recipe identity, personal state, milestone reward, owned display, predicate HUD, equipment modifier, world interaction, schedule and bossbar wiring.')
print('Files:',sum(p.is_file() for p in files))
