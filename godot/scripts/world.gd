extends Node3D
class_name World


var _tile_field_desc: TileFieldDesc
var _block_field_desc: BlockFieldDesc
var _entity_field_desc: EntityFieldDesc

var _tile_field: TileField
var _block_field: BlockField
var _entity_field: EntityField
var _node_store: NodeStore
var _callback_store: CallbackStore
var _world: WorldContext


func _enter_tree():
	_tile_field_desc = TileFieldDesc.new_from(
		2048,
		8,
		[
			TileFieldDescEntry.new_from(
				[preload("res://images/surface_dirt.webp")] as Array[Image],
				false,
			),
			TileFieldDescEntry.new_from(
				[preload("res://images/surface_grass.webp")] as Array[Image],
				false,
			)
		] as Array[TileFieldDescEntry],
		preload("res://field.gdshader"),
	)
	_tile_field = TileField.new_from(_tile_field_desc, get_world_3d())

	_block_field_desc = BlockFieldDesc.new_from(
		2048,
		8,
		[
			BlockFieldDescEntry.new_from(
				Vector2i(1, 1),
				[preload("res://images/dandelion.webp")] as Array[Image],
				false,
				Vector2(1.0, 1.0), Vector2(0.0, 0.0),
				Vector2(0.0, 0.0), Vector2(0.0, 0.0),
			),
			BlockFieldDescEntry.new_from(
				Vector2i(1, 1),
				[preload("res://images/fallen_leaves.webp")] as Array[Image],
				false,
				Vector2(1.0, 1.0), Vector2(0.0, 0.0),
				Vector2(0.0, 0.0), Vector2(0.0, 0.0),
			),
			BlockFieldDescEntry.new_from(
				Vector2i(1, 1),
				[preload("res://images/mix_grass.webp")] as Array[Image],
				true,
				Vector2(1.0, 1.0), Vector2(0.0, 0.0),
				Vector2(0.0, 0.0), Vector2(0.0, 0.0),
			),
			BlockFieldDescEntry.new_from(
				Vector2i(1, 1),
				[preload("res://images/mix_pebbles.webp")] as Array[Image],
				false,
				Vector2(1.0, 1.0), Vector2(0.0, 0.0),
				Vector2(0.0, 0.0), Vector2(0.0, 0.0),
			)
		] as Array[BlockFieldDescEntry],
		preload("res://field.gdshader"),
	)
	_block_field = BlockField.new_from(_block_field_desc, get_world_3d())

	_entity_field_desc = EntityFieldDesc.new_from(
		2048,
		8,
		[
			EntityFieldDescEntry.new_from(
				[preload("res://images/player.webp")] as Array[Image],
				true,
				Vector2(1.5, 2.5), Vector2(-0.75, 0.0),
				Vector2(0.8, 0.8), Vector2(-0.4, 0.1),
			),
			EntityFieldDescEntry.new_from(
				[preload("res://images/pig.webp")] as Array[Image],
				true,
				Vector2(2.0, 2.0), Vector2(-1.0, 0.0),
				Vector2(0.8, 0.8), Vector2(-0.4, 0.1),
			),
			EntityFieldDescEntry.new_from(
				[preload("res://images/cow.webp")] as Array[Image],
				true,
				Vector2(2.0, 2.0), Vector2(-1.0, 0.0),
				Vector2(0.8, 0.8), Vector2(-0.4, 0.1),
			),
			EntityFieldDescEntry.new_from(
				[preload("res://images/sheep.webp")] as Array[Image],
				true,
				Vector2(2.0, 2.0), Vector2(-1.0, 0.0),
				Vector2(0.8, 0.8), Vector2(-0.4, 0.1),
			),
			EntityFieldDescEntry.new_from(
				[preload("res://images/chiken.webp")] as Array[Image],
				true,
				Vector2(1.0, 1.0), Vector2(-0.5, 0.0),
				Vector2(0.8, 0.8), Vector2(-0.4, 0.1),
			),
			EntityFieldDescEntry.new_from(
				[preload("res://images/bird.webp")] as Array[Image],
				true,
				Vector2(1.0, 1.0), Vector2(-0.5, 0.0),
				Vector2(0.8, 0.8), Vector2(-0.4, 0.1),
			)
		] as Array[EntityFieldDescEntry],
		preload("res://field.gdshader"),
	)
	_entity_field = EntityField.new_from(_entity_field_desc, get_world_3d())

	_node_store = NodeStore.new_from()

	var _callback_store_builder = CallbackStoreBuilder.new_from()
	_callback_store_builder.insert_bundle(Callback.new_generator(32, 2))
	_callback_store_builder.insert_bundle(Callback.new_random_walk_forward())
	_callback_store_builder.insert_bundle(Callback.new_random_walk(1, 3.0, 60.0, 1.0, 5.0, 0.5))
	_callback_store_builder.insert_bundle(Callback.new_random_walk(2, 3.0, 60.0, 1.0, 5.0, 0.5))
	_callback_store_builder.insert_bundle(Callback.new_random_walk(3, 3.0, 60.0, 1.0, 5.0, 0.5))
	_callback_store_builder.insert_bundle(Callback.new_random_walk(4, 3.0, 60.0, 1.0, 5.0, 0.5))
	_callback_store_builder.insert_bundle(Callback.new_random_walk(5, 3.0, 60.0, 1.0, 5.0, 0.5))
	_callback_store = _callback_store_builder.build()
	# CallbackBundle is initialized after running CallbackStoreBuilder.insert(CallbackBundle)
	# CallbackStoreBuilder is initialized after running CallbackStoreBuilder.build()

	_world = WorldContext.new_from(
		_tile_field,
		_block_field,
		_entity_field,
		_node_store,
		_callback_store,
	)

	# initialize world context
	Action.before(_world)


func _process(delta):
	Action.forward(_world, delta)

	# rendering
	_tile_field.update_view()
	_block_field.update_view()
	_entity_field.update_view()


func _exit_tree():
	# clean up world context
	Action.after(_world)
