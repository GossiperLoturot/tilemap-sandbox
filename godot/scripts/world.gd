extends Node3D


@export var ui: Control
var _forwarder_rect: Rect2
var _gen_rect: Rect2
var _view_rect: Rect2


func _enter_tree() -> void:
	Root.open(retrieve_func)


func _process(delta: float) -> void:
	# logic
	Root.forwarde_rect(_forwarder_rect, delta)
	Root.generate_rect(_gen_rect)
	Root.forward_time(delta)
	# rendering
	Root.update_view(_view_rect)


func _exit_tree() -> void:
	Root.close()


func _on_forwarder_rect_changed(rect: Rect2) -> void:
	_forwarder_rect = rect


func _on_gen_rect_changed(rect: Rect2) -> void:
	_gen_rect = rect


func _on_view_rect_changed(rect: Rect2) -> void:
	_view_rect = rect


func retrieve_func(name: String):
	print(name)

	var retrieve_table = {
		"shader_field": preload("res://shaders/field.gdshader"),
		"shader_field_shadow": preload("res://shaders/field_shadow.gdshader"),
		"shader_selection": preload("res://shaders/selector.gdshader"),
		"viewport": self.get_viewport(),
		"image_tile_dirt": preload("res://images/surface_dirt.webp"),
		"image_tile_grass": preload("res://images/surface_grass.webp"),
		"image_block_dandelion": preload("res://images/dandelion.webp"),
		"image_block_fallenleaves": preload("res://images/fallen_leaves.webp"),
		"image_block_mixgrass": preload("res://images/mix_grass.webp"),
		"image_block_mixpebbles": preload("res://images/mix_pebbles.webp"),
		"image_entity_player_idle0": preload("res://images/player_idle_0.webp"),
		"image_entity_player_idle1": preload("res://images/player_idle_1.webp"),
		"image_entity_player_walk0": preload("res://images/player_walk_0.webp"),
		"image_entity_player_walk1": preload("res://images/player_walk_1.webp"),
		"image_entity_pig_idle0": preload("res://images/pig_idle_0.webp"),
		"image_entity_pig_idle1": preload("res://images/pig_idle_1.webp"),
		"image_entity_pig_walk0": preload("res://images/pig_walk_0.webp"),
		"image_entity_pig_walk1": preload("res://images/pig_walk_1.webp"),
		"image_entity_cow_idle0": preload("res://images/cow_idle_0.webp"),
		"image_entity_cow_idle1": preload("res://images/cow_idle_1.webp"),
		"image_entity_cow_walk0": preload("res://images/cow_walk_0.webp"),
		"image_entity_cow_walk1": preload("res://images/cow_walk_1.webp"),
		"image_entity_sheep_idle0": preload("res://images/sheep_idle_0.webp"),
		"image_entity_sheep_idle1": preload("res://images/sheep_idle_1.webp"),
		"image_entity_sheep_walk0": preload("res://images/sheep_walk_0.webp"),
		"image_entity_sheep_walk1": preload("res://images/sheep_walk_1.webp"),
		"image_entity_chicken_idle": preload("res://images/chicken_idle.webp"),
		"image_entity_chicken_walk": preload("res://images/chicken_walk.webp"),
		"image_entity_bird_idle": preload("res://images/bird_idle.webp"),
		"image_entity_bird_walk": preload("res://images/bird_walk.webp"),
		"image_item_package": preload("res://images/package.webp"),
		"callable_inventory_player": open_inventory_player,
	}
	return retrieve_table[name]


func open_inventory_player(inventory_key: int):
	var instance = preload("res://scenes/inventory_player.tscn").instantiate()
	self.add_child(instance)
	instance.change_inventory(ui, inventory_key)
