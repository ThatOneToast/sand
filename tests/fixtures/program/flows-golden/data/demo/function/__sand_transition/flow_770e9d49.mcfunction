execute unless score @s s8dec516490d2d08 matches -2147483648..2147483647 run scoreboard players set @s s8dec516490d2d08 0
function demo:__sand_transition/flow_770e9d49/select
execute if score @s s8dec516490d2d08 matches 1 run function demo:__sand_transition/flow_770e9d49/tick_0
execute unless score @s s8dec516490d2d08 matches 1 run scoreboard players set @s __sf_770e9d49t1 0
execute if score @s s8dec516490d2d08 matches 1 run scoreboard players add @s __sf_770e9d49t1 1
execute if score @s s8dec516490d2d08 matches 1 if score @s __sf_770e9d49t1 matches 2.. run function demo:__sand_transition/flow_770e9d49/tick_1