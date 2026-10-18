scoreboard objectives add s02252f5fe4679ec dummy
scoreboard objectives add s048b722efdc0917 dummy
scoreboard objectives add sffe99bc0114fb8b dummy
execute unless score #sand_demo_counter_946714f8 sffe99bc0114fb8b matches -2147483648.. run scoreboard players set #sand_demo_counter_946714f8 sffe99bc0114fb8b 10
execute unless score #sand_demo_counter_946714f8 s048b722efdc0917 matches 1.. run scoreboard players set #sand_demo_counter_946714f8 s048b722efdc0917 1
scoreboard players add #sand_demo_counter_946714f8 sffe99bc0114fb8b 0
execute if score #sand_demo_counter_946714f8 sffe99bc0114fb8b matches ..-1 run scoreboard players set #sand_demo_counter_946714f8 sffe99bc0114fb8b 0
execute if score #sand_demo_counter_946714f8 sffe99bc0114fb8b matches 101.. run scoreboard players set #sand_demo_counter_946714f8 sffe99bc0114fb8b 100