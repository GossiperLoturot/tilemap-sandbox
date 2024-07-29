extends Node3D
class_name World


@export var min_forward_rect: Rect2
@export var min_view_rect: Rect2

var _root: Root


func _ready():
	var tile_field = TileField.new_from(TileFieldDescriptor.new_from(
		32,
		512,
		2048,
		8,
		[
			TileDescriptor.new_from(
				[preload("res://images/surface_dirt.webp")] as Array[Image],
				false,
			),
			TileDescriptor.new_from(
				[preload("res://images/surface_grass.webp")] as Array[Image],
				false,
			)
		] as Array[TileDescriptor],
		[
			preload("res://field.gdshader"),
		] as Array[Shader],
		get_world_3d()
	))

	var block_field = BlockField.new_from(BlockFieldDescriptor.new_from(
		32,
		512,
		2048,
		8,
		[
			BlockDescriptor.new_from(
				Vector2i(1, 1),
				[preload("res://images/dandelion.webp")] as Array[Image],
				false,
				Vector2(1.0, 1.0), Vector2(0.0, 0.0),
				Vector2(0.0, 0.0), Vector2(0.0, 0.0),
			),
			BlockDescriptor.new_from(
				Vector2i(1, 1),
				[preload("res://images/fallen_leaves.webp")] as Array[Image],
				false,
				Vector2(1.0, 1.0), Vector2(0.0, 0.0),
				Vector2(0.0, 0.0), Vector2(0.0, 0.0),
			),
			BlockDescriptor.new_from(
				Vector2i(1, 1),
				[preload("res://images/mix_grass.webp")] as Array[Image],
				true,
				Vector2(1.0, 1.0), Vector2(0.0, 0.0),
				Vector2(0.0, 0.0), Vector2(0.0, 0.0),
			),
			BlockDescriptor.new_from(
				Vector2i(1, 1),
				[preload("res://images/mix_pebbles.webp")] as Array[Image],
				false,
				Vector2(1.0, 1.0), Vector2(0.0, 0.0),
				Vector2(0.0, 0.0), Vector2(0.0, 0.0),
			)
		] as Array[BlockDescriptor],
		[
			preload("res://field.gdshader"),
			preload("res://field_shadow.gdshader")
		] as Array[Shader],
		get_world_3d()
	))

	var entity_field = EntityField.new_from(EntityFieldDescriptor.new_from(
		32,
		512,
		2048,
		8,
		[
			EntityDescriptor.new_from(
				[preload("res://images/player.webp")] as Array[Image],
				true,
				Vector2(1.5, 2.5), Vector2(-0.75, 0.0),
				Vector2(0.8, 0.8), Vector2(-0.4, 0.1),
			),
			EntityDescriptor.new_from(
				[preload("res://images/pig.webp")] as Array[Image],
				true,
				Vector2(2.0, 2.0), Vector2(-1.0, 0.0),
				Vector2(0.8, 0.8), Vector2(-0.4, 0.1),
			),
			EntityDescriptor.new_from(
				[preload("res://images/cow.webp")] as Array[Image],
				true,
				Vector2(2.0, 2.0), Vector2(-1.0, 0.0),
				Vector2(0.8, 0.8), Vector2(-0.4, 0.1),
			),
			EntityDescriptor.new_from(
				[preload("res://images/sheep.webp")] as Array[Image],
				true,
				Vector2(2.0, 2.0), Vector2(-1.0, 0.0),
				Vector2(0.8, 0.8), Vector2(-0.4, 0.1),
			),
			EntityDescriptor.new_from(
				[preload("res://images/chiken.webp")] as Array[Image],
				true,
				Vector2(1.0, 1.0), Vector2(-0.5, 0.0),
				Vector2(0.8, 0.8), Vector2(-0.4, 0.1),
			),
			EntityDescriptor.new_from(
				[preload("res://images/bird.webp")] as Array[Image],
				true,
				Vector2(1.0, 1.0), Vector2(-0.5, 0.0),
				Vector2(0.8, 0.8), Vector2(-0.4, 0.1),
			)
		] as Array[EntityDescriptor],
		[
			preload("res://field.gdshader"),
			preload("res://field_shadow.gdshader"),
		] as Array[Shader],
		get_world_3d()
	))

	var node_store = NodeStore.new_from()

	var callback_store = CallbackStore.new_from(CallbackStoreDescriptor.new_from(
		[
			CallbackBundles.new_generator(),
			CallbackBundles.new_random_walk_forward_local(),
			CallbackBundles.new_base_tile(0),
			CallbackBundles.new_base_tile(1),
			CallbackBundles.new_base_block(0),
			CallbackBundles.new_base_block(1),
			CallbackBundles.new_base_block(2),
			CallbackBundles.new_base_block(3),
			CallbackBundles.new_base_entity(0),
			CallbackBundles.new_animal_entity(1, 3.0, 60.0, 1.0, 5.0, 0.5),
			CallbackBundles.new_animal_entity(2, 3.0, 60.0, 1.0, 5.0, 0.5),
			CallbackBundles.new_animal_entity(3, 3.0, 60.0, 1.0, 5.0, 0.5),
			CallbackBundles.new_animal_entity(4, 3.0, 60.0, 1.0, 5.0, 0.5),
			CallbackBundles.new_animal_entity(5, 3.0, 60.0, 1.0, 5.0, 0.5),
		] as Array[CallbackBundle]
	))

	_root = Root.new_from(
		tile_field,
		block_field,
		entity_field,
		node_store,
		callback_store,
	)

	# initialize world context
	Actions.before(_root)


func _process(delta):
	Actions.forward(_root, delta)
	Actions.forward_local(_root, delta, min_forward_rect)

	Actions.generate_chunk(_root, min_view_rect)

	# rendering
	_root.tile_field.update_view(min_view_rect)
	_root.block_field.update_view(min_view_rect)
	_root.entity_field.update_view(min_view_rect)


func _exit_tree():
	# clean up world context
	Actions.after(_root)
