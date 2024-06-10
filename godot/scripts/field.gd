extends Node3D
class_name Field


@export var camera: Camera3D

var _tile_field_desc: TileFieldDesc
var _block_field_desc: BlockFieldDesc
var _entity_field_desc: EntityFieldDesc
var _behavior_plugin_desc: BehaviorPluginDesc

var _tile_field: TileField
var _block_field: BlockField
var _entity_field: EntityField
var _behavior_plugin: BehaviorPlugin


func _ready():
	_tile_field_desc = TileFieldDesc.new_from(
		2048,
		8,
		[
			TileFieldDescEntry.new_from(preload("res://images/surface_dirt.png")),
			TileFieldDescEntry.new_from(preload("res://images/surface_grass.png")),
			TileFieldDescEntry.new_from(preload("res://images/surface_gravel.png")),
			TileFieldDescEntry.new_from(preload("res://images/surface_sand.png")),
			TileFieldDescEntry.new_from(preload("res://images/surface_stone.png")),
		] as Array[TileFieldDescEntry],
		preload("res://field.gdshader"),
	)
	_tile_field = TileField.new_from(_tile_field_desc, get_world_3d())

	_block_field_desc = BlockFieldDesc.new_from(
		2048,
		8,
		[
			BlockFieldDescEntry.new_from(
				Vector2i(4, 2),
				preload("res://images/birch_tree.png"),
				true,
				Vector2(4.0, 6.0), Vector2(0.0, 0.0),
				Vector2(1.0, 0.5), Vector2(1.5, 0.0),
			),
			BlockFieldDescEntry.new_from(
				Vector2i(1, 1),
				preload("res://images/dandelion.png"),
				false,
				Vector2(1.0, 1.0), Vector2(0.0, 0.0),
				Vector2(0.0, 0.0), Vector2(0.0, 0.0),
			),
			BlockFieldDescEntry.new_from(
				Vector2i(4, 2),
				preload("res://images/dying_tree.png"),
				true,
				Vector2(4.0, 6.0), Vector2(0.0, 0.0),
				Vector2(1.0, 0.5), Vector2(1.5, 0.0),
			),
			BlockFieldDescEntry.new_from(
				Vector2i(1, 1),
				preload("res://images/fallen_branch.png"),
				false,
				Vector2(1.0, 1.0), Vector2(0.0, 0.0),
				Vector2(0.0, 0.0), Vector2(0.0, 0.0),
			),
			BlockFieldDescEntry.new_from(
				Vector2i(1, 1),
				preload("res://images/fallen_leaves.png"),
				false,
				Vector2(1.0, 1.0), Vector2(0.0, 0.0),
				Vector2(0.0, 0.0), Vector2(0.0, 0.0),
			),
			BlockFieldDescEntry.new_from(
				Vector2i(1, 1),
				preload("res://images/mix_grass.png"),
				false,
				Vector2(1.0, 1.0), Vector2(0.0, 0.0),
				Vector2(0.0, 0.0), Vector2(0.0, 0.0),
			),
			BlockFieldDescEntry.new_from(
				Vector2i(1, 1),
				preload("res://images/mix_pebbles.png"),
				false,
				Vector2(1.0, 1.0), Vector2(0.0, 0.0),
				Vector2(0.0, 0.0), Vector2(0.0, 0.0),
			),
			BlockFieldDescEntry.new_from(
				Vector2i(2, 1),
				preload("res://images/mix_rock.png"),
				true,
				Vector2(2.0, 2.0), Vector2(0.0, 0.0),
				Vector2(2.0, 1.0), Vector2(0.0, 0.0),
			),
			BlockFieldDescEntry.new_from(
				Vector2i(4, 2),
				preload("res://images/oak_tree.png"),
				false,
				Vector2(4.0, 6.0), Vector2(0.0, 0.0),
				Vector2(1.0, 0.5), Vector2(1.5, 0.0),
			),
		] as Array[BlockFieldDescEntry],
		preload("res://field.gdshader"),
	)
	_block_field = BlockField.new_from(_block_field_desc, get_world_3d())

	_entity_field_desc = EntityFieldDesc.new_from(
		2048,
		8,
		[
			EntityFieldDescEntry.new_from(
				preload("res://images/frame1x2.png"),
				true,
				Vector2(1.0, 2.0), Vector2(-0.5, 0.0),
				Vector2(0.8, 0.8), Vector2(-0.4, 0.1),
			),
			EntityFieldDescEntry.new_from(
				preload("res://images/frame1x1.png"),
				true,
				Vector2(1.0, 1.0), Vector2(-0.5, 0.0),
				Vector2(0.8, 0.8), Vector2(-0.4, 0.1),
			),
		] as Array[EntityFieldDescEntry],
		preload("res://field.gdshader"),
	)
	_entity_field = EntityField.new_from(_entity_field_desc, get_world_3d())

	var tile_factories = [
		BehaviorFactory.new_unit(),
		BehaviorFactory.new_unit(),
	] as Array[BehaviorFactory]
	var block_factories = [
		BehaviorFactory.new_unit(),
		BehaviorFactory.new_unit(),
		BehaviorFactory.new_unit(),
		BehaviorFactory.new_unit(),
		BehaviorFactory.new_unit(),
		BehaviorFactory.new_unit(),
		BehaviorFactory.new_unit(),
		BehaviorFactory.new_unit(),
	] as Array[BehaviorFactory]
	var entity_factories = [
		BehaviorFactory.new_generator(),
		BehaviorFactory.new_random_walk(0.5, 5.0, 5.0, 10.0, 1.0),
	] as Array[BehaviorFactory]
	_behavior_plugin_desc = BehaviorPluginDesc.new_from(
		tile_factories,
		block_factories,
		entity_factories,
	)
	_behavior_plugin = BehaviorPlugin.new_from(
		_behavior_plugin_desc,
		_tile_field,
		_block_field,
		_entity_field,
	)

	for y in range(-64, 65):
		for x in range(-64, 65):
			_behavior_plugin.place_tile(Tile.new_from(randi_range(0, 1), Vector2i(x, y)))

	for i in range(4096):
		var x = randi_range(-64, 65)
		var y = randf_range(-64, 65)
		_behavior_plugin.place_block(Block.new_from(randi_range(0, 8), Vector2i(x, y)))

	for i in range(64):
		var x = randf_range(-64.0, 64.0)
		var y = randf_range(-64.0, 64.0)
		_behavior_plugin.place_entity(Entity.new_from(1, Vector2(x, y)))

	for y in range(-4, 5):
		for x in range(-4, 5):
			_tile_field.insert_view(Vector2i(x, y))
			_block_field.insert_view(Vector2i(x, y))
			_entity_field.insert_view(Vector2i(x, y))


func _process(delta):
	_behavior_plugin.update(delta)

	_tile_field.update_view()
	_block_field.update_view()
	_entity_field.update_view()
