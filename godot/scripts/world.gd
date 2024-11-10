extends Node3D
class_name World


@export var min_forward_rect: Rect2
@export var min_generate_rect: Rect2
@export var min_view_rect: Rect2

const TILE_DIRT: int = 0
const TILE_GRASS: int = 1
const TILE_ID_SIZE: int = 2

const BLOCK_DANDELION: int = 0
const BLOCK_FALLEN_LEAVES: int = 1
const BLOCK_MIX_GRASS: int = 2
const BLOCK_MIX_PEBBLES: int = 3
const BLOCK_ID_SIZE: int = 4

const ENTITY_PLAYER: int = 0
const ENTITY_PIG: int = 1
const ENTITY_COW: int = 2
const ENTITY_SHEEP: int = 3
const ENTITY_CHICKEN: int = 4
const ENTITY_BIRD: int = 5
const ENTITY_PACKAGE: int = 6
const ENTITY_ID_SIZE: int = 7

const ITEM_PACKAGE: int = 0
const ITEM_ID_SIZE: int = 1

var _root: Root


func _ready() -> void:
	var tile_descriptors: Array[TileDescriptor] = []
	tile_descriptors.resize(TILE_ID_SIZE)
	tile_descriptors[TILE_DIRT] = TileDescriptor.create(
		[
			TileImageDescriptor.create(
				[preload("res://images/surface_dirt.webp")] as Array[Image],
				0,
				true
			)
		] as Array[TileImageDescriptor],
		false,
		TileFeature.create_empty()
	)
	tile_descriptors[TILE_GRASS] = TileDescriptor.create(
		[
			TileImageDescriptor.create(
				[preload("res://images/surface_grass.webp")] as Array[Image],
				0,
				true
			)
		] as Array[TileImageDescriptor],
		false,
		TileFeature.create_empty()
	)

	var block_descriptors: Array[BlockDescriptor] = []
	block_descriptors.resize(BLOCK_ID_SIZE)
	block_descriptors[BLOCK_DANDELION] = BlockDescriptor.create(
		[
			BlockImageDescriptor.create(
				[preload("res://images/dandelion.webp")] as Array[Image],
				0,
				true
			)
		] as Array[BlockImageDescriptor],
		false,
		Vector2i(1, 1),
		Vector2(0.0, 0.0), Vector2(0.0, 0.0),
		Vector2(1.0, 1.0), Vector2(0.0, 0.0),
		BlockFeature.create_empty()
	)
	block_descriptors[BLOCK_FALLEN_LEAVES] = BlockDescriptor.create(
		[
			BlockImageDescriptor.create(
				[preload("res://images/fallen_leaves.webp")] as Array[Image],
				0,
				true
			)
		] as Array[BlockImageDescriptor],
		false,
		Vector2i(1, 1),
		Vector2(0.0, 0.0), Vector2(0.0, 0.0),
		Vector2(1.0, 1.0), Vector2(0.0, 0.0),
		BlockFeature.create_empty()
	)
	block_descriptors[BLOCK_MIX_GRASS] = BlockDescriptor.create(
		[
			BlockImageDescriptor.create(
				[preload("res://images/mix_grass.webp")] as Array[Image],
				0,
				true
			)
		] as Array[BlockImageDescriptor],
		true,
		Vector2i(1, 1),
		Vector2(0.0, 0.0), Vector2(0.0, 0.0),
		Vector2(1.0, 1.0), Vector2(0.0, 0.0),
		BlockFeature.create_empty()
	)
	block_descriptors[BLOCK_MIX_PEBBLES] = BlockDescriptor.create(
		[
			BlockImageDescriptor.create(
				[preload("res://images/mix_pebbles.webp")] as Array[Image],
				0,
				true
			)
		] as Array[BlockImageDescriptor],
		false,
		Vector2i(1, 1),
		Vector2(0.0, 0.0), Vector2(0.0, 0.0),
		Vector2(1.0, 1.0), Vector2(0.0, 0.0),
		BlockFeature.create_empty()
	)

	var entity_descriptors: Array[EntityDescriptor] = []
	entity_descriptors.resize(ENTITY_ID_SIZE)
	entity_descriptors[ENTITY_PLAYER] = EntityDescriptor.create(
		[
			EntityImageDescriptor.create(
				[
					preload("res://images/player_idle_0.webp"),
					preload("res://images/player_idle_1.webp")
				] as Array[Image],
				24,
				true
			),
			EntityImageDescriptor.create(
				[
					preload("res://images/player_walk_0.webp"),
					preload("res://images/player_idle_1.webp"),
					preload("res://images/player_walk_1.webp"),
					preload("res://images/player_idle_1.webp")
				] as Array[Image],
				6,
				true
			)
		] as Array[EntityImageDescriptor],
		true,
		Vector2(0.8, 0.8), Vector2(-0.4, 0.1),
		Vector2(1.5, 2.25), Vector2(-0.75, 0.0),
		EntityFeature.create_player()
	)
	entity_descriptors[ENTITY_PIG] = EntityDescriptor.create(
		[
			EntityImageDescriptor.create(
				[
					preload("res://images/pig_idle_0.webp"),
					preload("res://images/pig_idle_1.webp")
				] as Array[Image],
				24,
				true
			),
			EntityImageDescriptor.create(
				[
					preload("res://images/pig_walk_0.webp"),
					preload("res://images/pig_idle_1.webp"),
					preload("res://images/pig_walk_1.webp"),
					preload("res://images/pig_idle_1.webp")
				] as Array[Image],
				12,
				true
			)
		] as Array[EntityImageDescriptor],
		true,
		Vector2(0.8, 0.8), Vector2(-0.4, 0.1),
		Vector2(2.0, 2.0), Vector2(-1.0, 0.0),
		EntityFeature.create_animal(0.0, 10.0, 0.0, 10.0, 1.0, 0, 1)
	)
	entity_descriptors[ENTITY_COW] = EntityDescriptor.create(
		[
			EntityImageDescriptor.create(
				[
					preload("res://images/cow_idle_0.webp"),
					preload("res://images/cow_idle_1.webp")
				] as Array[Image],
				24,
				true
			),
			EntityImageDescriptor.create(
				[
					preload("res://images/cow_walk_0.webp"),
					preload("res://images/cow_idle_1.webp"),
					preload("res://images/cow_walk_1.webp"),
					preload("res://images/cow_idle_1.webp")
				] as Array[Image],
				12,
				true
			)
		] as Array[EntityImageDescriptor],
		true,
		Vector2(0.8, 0.8), Vector2(-0.4, 0.1),
		Vector2(2.0, 2.0), Vector2(-1.0, 0.0),
		EntityFeature.create_animal(0.0, 10.0, 0.0, 10.0, 1.0, 0, 1)
	)
	entity_descriptors[ENTITY_SHEEP] = EntityDescriptor.create(
		[
			EntityImageDescriptor.create(
				[
					preload("res://images/sheep_idle_0.webp"),
					preload("res://images/sheep_idle_1.webp")
				] as Array[Image],
				24,
				true
			),
			EntityImageDescriptor.create(
				[
					preload("res://images/sheep_walk_0.webp"),
					preload("res://images/sheep_idle_1.webp"),
					preload("res://images/sheep_walk_1.webp"),
					preload("res://images/sheep_idle_1.webp")
				] as Array[Image],
				12,
				true
			)
		] as Array[EntityImageDescriptor],
		true,
		Vector2(0.8, 0.8), Vector2(-0.4, 0.1),
		Vector2(2.0, 2.0), Vector2(-1.0, 0.0),
		EntityFeature.create_animal(0.0, 10.0, 0.0, 10.0, 1.0, 0, 1)
	)
	entity_descriptors[ENTITY_CHICKEN] = EntityDescriptor.create(
		[
			EntityImageDescriptor.create(
				[preload("res://images/chiken_idle.webp")] as Array[Image],
				0,
				true
			),
			EntityImageDescriptor.create(
				[
					preload("res://images/chiken_walk.webp"),
					preload("res://images/chiken_idle.webp")
				] as Array[Image],
				12,
				true
			)
		] as Array[EntityImageDescriptor],
		true,
		Vector2(0.8, 0.8), Vector2(-0.4, 0.1),
		Vector2(1.0, 1.0), Vector2(-0.5, 0.0),
		EntityFeature.create_animal(0.0, 10.0, 0.0, 10.0, 1.0, 0, 1)
	)
	entity_descriptors[ENTITY_BIRD] = EntityDescriptor.create(
		[
			EntityImageDescriptor.create(
				[preload("res://images/bird_idle.webp")] as Array[Image],
				0,
				true
			),
			EntityImageDescriptor.create(
				[
					preload("res://images/bird_walk.webp"),
					preload("res://images/bird_idle.webp")
				] as Array[Image],
				12,
				true
			)
		] as Array[EntityImageDescriptor],
		true,
		Vector2(0.8, 0.8), Vector2(-0.4, 0.1),
		Vector2(1.0, 1.0), Vector2(-0.5, 0.0),
		EntityFeature.create_animal(0.0, 10.0, 0.0, 10.0, 1.0, 0, 1)
	)
	entity_descriptors[ENTITY_PACKAGE] = EntityDescriptor.create(
		[
			EntityImageDescriptor.create(
				[preload("res://images/package.webp")] as Array[Image],
				0,
				true
			)
		] as Array[EntityImageDescriptor],
		true,
		Vector2(0.0, 0.0), Vector2(0.0, 0.0),
		Vector2(0.8, 0.8), Vector2(-0.4, 0.0),
		EntityFeature.create_empty()
	)

	var item_descriptors: Array[ItemDescriptor] = []
	item_descriptors.resize(ITEM_ID_SIZE)
	item_descriptors[ITEM_PACKAGE] = ItemDescriptor.create(
		preload("res://images/package.webp"),
		ItemFeature.create_empty()
	)

	_root = Root.create(
		RootDescriptor.create(
			TileFieldDescriptor.create(
				tile_descriptors,
				[preload("res://field.gdshader")] as Array[Shader],
				get_world_3d()
			),
			BlockFieldDescriptor.create(
				block_descriptors,
				[preload("res://field.gdshader"), preload("res://field_shadow.gdshader")] as Array[Shader],
				get_world_3d()
			),
			EntityFieldDescriptor.create(
				entity_descriptors,
				[preload("res://field.gdshader"), preload("res://field_shadow.gdshader")] as Array[Shader],
				get_world_3d()
			),
			ItemIndexDescriptor.create(
				item_descriptors
			)
		)
	)

	_root.forwarder_init()

	var generator_resource_descriptor = GeneratorResourceDescriptor.create(
		[
			GeneratorRule.create_marching(0.5, TILE_GRASS),
			GeneratorRule.create_marching(1.0, TILE_DIRT)
		] as Array[GeneratorRule],
		[
			GeneratorRule.create_spawn(0.05, BLOCK_DANDELION),
			GeneratorRule.create_spawn(0.05, BLOCK_FALLEN_LEAVES),
			GeneratorRule.create_spawn(0.05, BLOCK_MIX_GRASS),
			GeneratorRule.create_spawn(0.05, BLOCK_MIX_PEBBLES)
		] as Array[GeneratorRule],
		[
			GeneratorRule.create_spawn(0.001, ENTITY_COW),
			GeneratorRule.create_spawn(0.001, ENTITY_PIG),
			GeneratorRule.create_spawn(0.001, ENTITY_SHEEP),
			GeneratorRule.create_spawn(0.001, ENTITY_CHICKEN),
			GeneratorRule.create_spawn(0.001, ENTITY_BIRD),
			GeneratorRule.create_spawn(0.001, ENTITY_PACKAGE)
		] as Array[GeneratorRule]
	)
	_root.generator_init(generator_resource_descriptor)

	_root.inventory_init()

	_root.player_init()


func _process(delta_secs) -> void:
	# logic
	_root.forwarder_exec_rect(min_forward_rect, delta_secs)

	_root.generator_exec_rect(min_generate_rect)

	_root.time_forward(delta_secs)

	# rendering
	_root.update_view(min_view_rect)
