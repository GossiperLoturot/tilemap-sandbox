extends Node3D
class_name World


@export var min_forward_rect: Rect2
@export var min_generate_rect: Rect2
@export var min_view_rect: Rect2

var _root: Root


func _ready():
	_root = Root.create(
		RootDescriptor.create(
			TileFieldDescriptor.create(
				32,
				512,
				2048,
				8,
				[
					TileDescriptor.create(
						[preload("res://images/surface_dirt.webp")] as Array[Image],
						false,
						TileFeature.create(),
					),
					TileDescriptor.create(
						[preload("res://images/surface_grass.webp")] as Array[Image],
						false,
						TileFeature.create(),
					)
				] as Array[TileDescriptor],
				[
					preload("res://field.gdshader"),
				] as Array[Shader],
				get_world_3d()
			),
			BlockFieldDescriptor.create(
				32,
				512,
				2048,
				8,
				[
					BlockDescriptor.create(
						[preload("res://images/dandelion.webp")] as Array[Image],
						false,
						Vector2i(1, 1),
						Vector2(0.0, 0.0), Vector2(0.0, 0.0),
						Vector2(1.0, 1.0), Vector2(0.0, 0.0),
						BlockFeature.create(),
					),
					BlockDescriptor.create(
						[preload("res://images/fallen_leaves.webp")] as Array[Image],
						false,
						Vector2i(1, 1),
						Vector2(0.0, 0.0), Vector2(0.0, 0.0),
						Vector2(1.0, 1.0), Vector2(0.0, 0.0),
						BlockFeature.create(),
					),
					BlockDescriptor.create(
						[preload("res://images/mix_grass.webp")] as Array[Image],
						true,
						Vector2i(1, 1),
						Vector2(0.0, 0.0), Vector2(0.0, 0.0),
						Vector2(1.0, 1.0), Vector2(0.0, 0.0),
						BlockFeature.create(),
					),
					BlockDescriptor.create(
						[preload("res://images/mix_pebbles.webp")] as Array[Image],
						false,
						Vector2i(1, 1),
						Vector2(0.0, 0.0), Vector2(0.0, 0.0),
						Vector2(1.0, 1.0), Vector2(0.0, 0.0),
						BlockFeature.create(),
					)
				] as Array[BlockDescriptor],
				[
					preload("res://field.gdshader"),
					preload("res://field_shadow.gdshader")
				] as Array[Shader],
				get_world_3d()
			),
			EntityFieldDescriptor.create(
				32,
				512,
				2048,
				8,
				[
					EntityDescriptor.create(
						[preload("res://images/player.webp")] as Array[Image],
						true,
						Vector2(0.8, 0.8), Vector2(-0.4, 0.1),
						Vector2(1.5, 2.5), Vector2(-0.75, 0.0),
						EntityFeature.create(),
					),
					EntityDescriptor.create(
						[preload("res://images/pig.webp")] as Array[Image],
						true,
						Vector2(0.8, 0.8), Vector2(-0.4, 0.1),
						Vector2(2.0, 2.0), Vector2(-1.0, 0.0),
						EntityFeature.create(),
					),
					EntityDescriptor.create(
						[preload("res://images/cow.webp")] as Array[Image],
						true,
						Vector2(0.8, 0.8), Vector2(-0.4, 0.1),
						Vector2(2.0, 2.0), Vector2(-1.0, 0.0),
						EntityFeature.create(),
					),
					EntityDescriptor.create(
						[preload("res://images/sheep.webp")] as Array[Image],
						true,
						Vector2(0.8, 0.8), Vector2(-0.4, 0.1),
						Vector2(2.0, 2.0), Vector2(-1.0, 0.0),
						EntityFeature.create(),
					),
					EntityDescriptor.create(
						[preload("res://images/chiken.webp")] as Array[Image],
						true,
						Vector2(0.8, 0.8), Vector2(-0.4, 0.1),
						Vector2(1.0, 1.0), Vector2(-0.5, 0.0),
						EntityFeature.create(),
					),
					EntityDescriptor.create(
						[preload("res://images/bird.webp")] as Array[Image],
						true,
						Vector2(0.8, 0.8), Vector2(-0.4, 0.1),
						Vector2(1.0, 1.0), Vector2(-0.5, 0.0),
						EntityFeature.create(),
					)
				] as Array[EntityDescriptor],
				[
					preload("res://field.gdshader"),
					preload("res://field_shadow.gdshader"),
				] as Array[Shader],
				get_world_3d()
			)
		)
	)

	_root.init_generator(32)


func _process(_delta):
	# logic
	_root.forward(min_forward_rect)
	_root.generate_chunk(min_generate_rect)

	# rendering
	_root.update_view(min_view_rect)
