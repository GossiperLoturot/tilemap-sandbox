extends Control
class_name Selection


static var context: Selection

@export var label: Label


func _enter_tree() -> void:
	context = self


func _exit_tree() -> void:
	context = null


func select_tile(tile_key: TileKey) -> void:
	if tile_key:
		show_selection()
		var tile = Context.get_tile(tile_key)
		draw_selection(tile.display_name)
		Context.draw_selection_tile(tile_key)
	else:
		hide_selection()


func select_block(block_key: BlockKey) -> void:
	if block_key:
		show_selection()
		var block = Context.get_block(block_key)
		draw_selection(block.display_name)
		Context.draw_selection_block(block_key)
	else:
		hide_selection()


func select_entity(entity_key: EntityKey) -> void:
	if entity_key:
		show_selection()
		var entity = Context.get_entity(entity_key)
		draw_selection(entity.display_name)
		Context.draw_selection_entity(entity_key)
	else:
		hide_selection()


func select_item(slot_key: SlotKey) -> void:
	var slot = Context.get_slot(slot_key)
	if slot:
		show_selection()
		draw_selection(slot.display_name)
	else:
		hide_selection()


func confirm_tile(tile_key: TileKey) -> void:
	print(tile_key)


func confirm_block(block_key: BlockKey) -> void:
	print(block_key)


func confirm_entity(entity_key: EntityKey) -> void:
	print(entity_key)


func confirm_item(slot_key: SlotKey) -> void:
	print(slot_key)


func show_selection() -> void:
	self.show()
	Context.draw_selection_none()


func hide_selection() -> void:
	self.hide()
	Context.draw_selection_none()


func draw_selection(text) -> void:
	label.text = text

	self.position = self.get_global_mouse_position()
