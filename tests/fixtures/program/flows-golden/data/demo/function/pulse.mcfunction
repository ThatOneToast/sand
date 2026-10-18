function demo:__sand_lifecycle_init
scoreboard players remove @s s108eb94c11c6345 5
execute if score @s s108eb94c11c6345 matches ..-1 run scoreboard players set @s s108eb94c11c6345 0
execute if score @s s108eb94c11c6345 matches 101.. run scoreboard players set @s s108eb94c11c6345 100