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
const ENTITY_CHICKET: int = 4
const ENTITY_BIRD: int = 5
const ENTITY_ID_SIZE: int = 6

var _root: Root


func _ready():
	var tile_descriptors: Array[TileDescriptor] = []
	tile_descriptors.resize(TILE_ID_SIZE)
	tile_descriptors[TILE_DIRT] = TileDescriptor.create(
		[preload("res://images/surface_dirt.webp")] as Array[Image],
		false,
		TileFeature.create(),
	)
	tile_descriptors[TILE_GRASS] = TileDescriptor.create(
		[preload("res://images/surface_grass.webp")] as Array[Image],
		false,
		TileFeature.create(),
	)

	var block_descriptors: Array[BlockDescriptor] = []
	block_descriptors.resize(BLOCK_ID_SIZE)
	block_descriptors[BLOCK_DANDELION] = BlockDescriptor.create(
		[preload("res://images/dandelion.webp")] as Array[Image],
		false,
		Vector2i(1, 1),
		Vector2(0.0, 0.0), Vector2(0.0, 0.0),
		Vector2(1.0, 1.0), Vector2(0.0, 0.0),
		BlockFeature.create(),
	)
	block_descriptors[BLOCK_FALLEN_LEAVES] = BlockDescriptor.create(
		[preload("res://images/fallen_leaves.webp")] as Array[Image],
		false,
		Vector2i(1, 1),
		Vector2(0.0, 0.0), Vector2(0.0, 0.0),
		Vector2(1.0, 1.0), Vector2(0.0, 0.0),
		BlockFeature.create(),
	)
	block_descriptors[BLOCK_MIX_GRASS] = BlockDescriptor.create(
		[preload("res://images/mix_grass.webp")] as Array[Image],
		true,
		Vector2i(1, 1),
		Vector2(0.0, 0.0), Vector2(0.0, 0.0),
		Vector2(1.0, 1.0), Vector2(0.0, 0.0),
		BlockFeature.create(),
	)
	block_descriptors[BLOCK_MIX_PEBBLES] = BlockDescriptor.create(
		[preload("res://images/mix_pebbles.webp")] as Array[Image],
		false,
		Vector2i(1, 1),
		Vector2(0.0, 0.0), Vector2(0.0, 0.0),
		Vector2(1.0, 1.0), Vector2(0.0, 0.0),
		BlockFeature.create(),
	)

	var entity_descriptors: Array[EntityDescriptor] = []
	entity_descriptors.resize(ENTITY_ID_SIZE)
	entity_descriptors[ENTITY_PLAYER] = EntityDescriptor.create(
		[preload("res://images/player.webp")] as Array[Image],
		true,
		Vector2(0.8, 0.8), Vector2(-0.4, 0.1),
		Vector2(1.5, 2.5), Vector2(-0.75, 0.0),
		EntityFeature.create(),
	)
	entity_descriptors[ENTITY_PIG] = EntityDescriptor.create(
		[preload("res://images/pig.webp")] as Array[Image],
		true,
		Vector2(0.8, 0.8), Vector2(-0.4, 0.1),
		Vector2(2.0, 2.0), Vector2(-1.0, 0.0),
		EntityFeature.create(),
	)
	entity_descriptors[ENTITY_COW] = EntityDescriptor.create(
		[preload("res://images/cow.webp")] as Array[Image],
		true,
		Vector2(0.8, 0.8), Vector2(-0.4, 0.1),
		Vector2(2.0, 2.0), Vector2(-1.0, 0.0),
		EntityFeature.create(),
	)
	entity_descriptors[ENTITY_SHEEP] = EntityDescriptor.create(
		[preload("res://images/sheep.webp")] as Array[Image],
		true,
		Vector2(0.8, 0.8), Vector2(-0.4, 0.1),
		Vector2(2.0, 2.0), Vector2(-1.0, 0.0),
		EntityFeature.create(),
	)
	entity_descriptors[ENTITY_CHICKET] = EntityDescriptor.create(
		[preload("res://images/chiken.webp")] as Array[Image],
		true,
		Vector2(0.8, 0.8), Vector2(-0.4, 0.1),
		Vector2(1.0, 1.0), Vector2(-0.5, 0.0),
		EntityFeature.create(),
	)
	entity_descriptors[ENTITY_BIRD] = EntityDescriptor.create(
		[preload("res://images/bird.webp")] as Array[Image],
		true,
		Vector2(0.8, 0.8), Vector2(-0.4, 0.1),
		Vector2(1.0, 1.0), Vector2(-0.5, 0.0),
		EntityFeature.create(),
	)

	_root = Root.create(
		RootDescriptor.create(
			TileFieldDescriptor.create(
				32,
				512,
				2048,
				8,
				tile_descriptors,
				[preload("res://field.gdshader")] as Array[Shader],
				get_world_3d()
			),
			BlockFieldDescriptor.create(
				32,
				512,
				2048,
				8,
				block_descriptors,
				[preload("res://field.gdshader"), preload("res://field_shadow.gdshader")] as Array[Shader],
				get_world_3d()
			),
			EntityFieldDescriptor.create(
				32,
				512,
				2048,
				8,
				entity_descriptors,
				[preload("res://field.gdshader"), preload("res://field_shadow.gdshader")] as Array[Shader],
				get_world_3d()
			)
		)
	)

	var _generator_descriptor = GeneratorDescriptor.create(
		32,
		[
			GeneratorRuleDescriptor.create_marching(0, 0.5, TILE_GRASS),
			GeneratorRuleDescriptor.create_marching(0, 1.0, TILE_DIRT)
		] as Array[GeneratorRuleDescriptor],
		[
			GeneratorRuleDescriptor.create_spawn(0, 0.01, BLOCK_FALLEN_LEAVES),
			GeneratorRuleDescriptor.create_spawn(0, 0.01, BLOCK_MIX_GRASS)
		] as Array[GeneratorRuleDescriptor],
		[
			GeneratorRuleDescriptor.create_spawn(0, 0.01, ENTITY_COW),
			GeneratorRuleDescriptor.create_spawn(0, 0.01, ENTITY_PIG)
		] as Array[GeneratorRuleDescriptor]
	)
	var _generator = Generator.create(_generator_descriptor)
	_root.init_generator(_generator)


func _process(_delta):
	# logic
	_root.forward(min_forward_rect)

	_root.generate_chunk(min_generate_rect)

	# rendering
	_root.update_view(min_view_rect)
