extends Node3D
class_name Field


var _tile_field_desc: TileFieldDesc
var _block_field_desc: BlockFieldDesc
var _entity_field_desc: EntityFieldDesc

var _tile_field: TileField
var _block_field: BlockField
var _entity_field: EntityField
var _node_store: NodeStore
var _world: WorldContext


func _ready():
	_tile_field_desc = TileFieldDesc.new_from(
		2048,
		8,
		[
			TileFieldDescEntry.new_from(
				preload("res://images/surface_dirt.webp"),
				false,
			),
			TileFieldDescEntry.new_from(
				preload("res://images/surface_grass.webp"),
				false,
			),
			TileFieldDescEntry.new_from(
				preload("res://images/surface_gravel.webp"),
				false,
			),
			TileFieldDescEntry.new_from(
				preload("res://images/surface_sand.webp"),
				false,
			),
			TileFieldDescEntry.new_from(
				preload("res://images/surface_stone.webp"),
				false,
			),
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
				preload("res://images/birch_tree.webp"),
				true,
				Vector2(4.0, 6.0), Vector2(0.0, 0.0),
				Vector2(1.0, 0.5), Vector2(1.5, 0.0),
			),
			BlockFieldDescEntry.new_from(
				Vector2i(1, 1),
				preload("res://images/dandelion.webp"),
				false,
				Vector2(1.0, 1.0), Vector2(0.0, 0.0),
				Vector2(0.0, 0.0), Vector2(0.0, 0.0),
			),
			BlockFieldDescEntry.new_from(
				Vector2i(4, 2),
				preload("res://images/dying_tree.webp"),
				true,
				Vector2(4.0, 6.0), Vector2(0.0, 0.0),
				Vector2(1.0, 0.5), Vector2(1.5, 0.0),
			),
			BlockFieldDescEntry.new_from(
				Vector2i(1, 1),
				preload("res://images/fallen_branch.webp"),
				false,
				Vector2(1.0, 1.0), Vector2(0.0, 0.0),
				Vector2(0.0, 0.0), Vector2(0.0, 0.0),
			),
			BlockFieldDescEntry.new_from(
				Vector2i(1, 1),
				preload("res://images/fallen_leaves.webp"),
				false,
				Vector2(1.0, 1.0), Vector2(0.0, 0.0),
				Vector2(0.0, 0.0), Vector2(0.0, 0.0),
			),
			BlockFieldDescEntry.new_from(
				Vector2i(1, 1),
				preload("res://images/mix_grass.webp"),
				false,
				Vector2(1.0, 1.0), Vector2(0.0, 0.0),
				Vector2(0.0, 0.0), Vector2(0.0, 0.0),
			),
			BlockFieldDescEntry.new_from(
				Vector2i(1, 1),
				preload("res://images/mix_pebbles.webp"),
				false,
				Vector2(1.0, 1.0), Vector2(0.0, 0.0),
				Vector2(0.0, 0.0), Vector2(0.0, 0.0),
			),
			BlockFieldDescEntry.new_from(
				Vector2i(2, 1),
				preload("res://images/mix_rock.webp"),
				true,
				Vector2(2.0, 2.0), Vector2(0.0, 0.0),
				Vector2(2.0, 1.0), Vector2(0.0, 0.0),
			),
			BlockFieldDescEntry.new_from(
				Vector2i(4, 2),
				preload("res://images/oak_tree.webp"),
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
				preload("res://images/frame1x2.webp"),
				true,
				Vector2(1.0, 2.0), Vector2(-0.5, 0.0),
				Vector2(0.8, 0.8), Vector2(-0.4, 0.1),
			),
			EntityFieldDescEntry.new_from(
				preload("res://images/frame1x1.webp"),
				true,
				Vector2(1.0, 1.0), Vector2(-0.5, 0.0),
				Vector2(0.8, 0.8), Vector2(-0.4, 0.1),
			),
		] as Array[EntityFieldDescEntry],
		preload("res://field.gdshader"),
	)
	_entity_field = EntityField.new_from(_entity_field_desc, get_world_3d())

	_node_store = NodeStore.new_from()

	var _callback_store_builder = CallbackStoreBuilder.new_from()
	_callback_store_builder.insert_bundle(Callback.new_generator(16, 4))
	_callback_store_builder.insert_bundle(Callback.new_random_walk_forward())
	_callback_store_builder.insert_bundle(Callback.new_random_walk(1, 1.0, 3.0, 1.0, 5.0, 3.0))
	var _callback_store = _callback_store_builder.build()
	# CallbackBundle is initialized after running CallbackStoreBuilder.insert(CallbackBundle)
	# CallbackStoreBuilder is initialized after running CallbackStoreBuilder.build()

	_world = WorldContext.new_from(
		_tile_field,
		_block_field,
		_entity_field,
		_node_store,
		_callback_store,
	)

	Action.before(_world)
	Action.generate_chunk(_world, Vector2i(0, 0))

	for y in range(-4, 5):
		for x in range(-4, 5):
			_tile_field.insert_view(Vector2i(x, y))
			_block_field.insert_view(Vector2i(x, y))
			_entity_field.insert_view(Vector2i(x, y))


func _process(delta):
	Action.forward(_world, delta)

	_tile_field.update_view()
	_block_field.update_view()
	_entity_field.update_view()
