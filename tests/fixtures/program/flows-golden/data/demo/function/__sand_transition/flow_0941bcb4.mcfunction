execute unless score #sand_demo_world_78d17ea0 s709d2ddbe2c2931 matches -2147483648..2147483647 run scoreboard players set #sand_demo_world_78d17ea0 s709d2ddbe2c2931 0
function demo:__sand_transition/flow_0941bcb4/select
execute unless score #sand_demo_world_78d17ea0 s709d2ddbe2c2931 matches 1 run scoreboard players set #sand_demo_world_78d17ea0 __sf_0941bcb4t0 0
execute if score #sand_demo_world_78d17ea0 s709d2ddbe2c2931 matches 1 run scoreboard players add #sand_demo_world_78d17ea0 __sf_0941bcb4t0 1
execute if score #sand_demo_world_78d17ea0 s709d2ddbe2c2931 matches 1 if score #sand_demo_world_78d17ea0 __sf_0941bcb4t0 matches 2.. run function demo:__sand_transition/flow_0941bcb4/tick_0