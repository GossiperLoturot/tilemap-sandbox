extends Control


var _inventory_key: int
var _item_nodes: Array[Control]
var _src_local_key: int = -1


func _enter_tree() -> void:
	_item_nodes = [
		$"Container/Body/Container/InventoryItem#0/Container/Placeholder",
		$"Container/Body/Container/InventoryItem#1/Container/Placeholder",
		$"Container/Body/Container/InventoryItem#2/Container/Placeholder",
		$"Container/Body/Container/InventoryItem#3/Container/Placeholder",
		$"Container/Body/Container/InventoryItem#4/Container/Placeholder",
		$"Container/Body/Container/InventoryItem#5/Container/Placeholder",
		$"Container/Body/Container/InventoryItem#6/Container/Placeholder",
		$"Container/Body/Container/InventoryItem#7/Container/Placeholder",
		$"Container/Body/Container/InventoryItem#8/Container/Placeholder",
		$"Container/Body/Container/InventoryItem#9/Container/Placeholder",
		$"Container/Body/Container/InventoryItem#10/Container/Placeholder",
		$"Container/Body/Container/InventoryItem#11/Container/Placeholder",
		$"Container/Body/Container/InventoryItem#12/Container/Placeholder",
		$"Container/Body/Container/InventoryItem#13/Container/Placeholder",
		$"Container/Body/Container/InventoryItem#14/Container/Placeholder",
		$"Container/Body/Container/InventoryItem#15/Container/Placeholder",
		$"Container/Body/Container/InventoryItem#16/Container/Placeholder",
		$"Container/Body/Container/InventoryItem#17/Container/Placeholder",
		$"Container/Body/Container/InventoryItem#18/Container/Placeholder",
		$"Container/Body/Container/InventoryItem#19/Container/Placeholder",
		$"Container/Body/Container/InventoryItem#20/Container/Placeholder",
		$"Container/Body/Container/InventoryItem#21/Container/Placeholder",
		$"Container/Body/Container/InventoryItem#22/Container/Placeholder",
		$"Container/Body/Container/InventoryItem#23/Container/Placeholder",
		$"Container/Body/Container/InventoryItem#24/Container/Placeholder",
		$"Container/Body/Container/InventoryItem#25/Container/Placeholder",
		$"Container/Body/Container/InventoryItem#26/Container/Placeholder",
		$"Container/Body/Container/InventoryItem#27/Container/Placeholder",
		$"Container/Body/Container/InventoryItem#28/Container/Placeholder",
		$"Container/Body/Container/InventoryItem#29/Container/Placeholder",
		$"Container/Body/Container/InventoryItem#30/Container/Placeholder",
		$"Container/Body/Container/InventoryItem#31/Container/Placeholder",
	]

	# register function for each slot
	for i in _item_nodes.size():
		_item_nodes[i].connect("gui_input", cursor_slot.bind(i))


func _process(_delta: float) -> void:
	for i in _item_nodes.size():
		Root.draw_item(_inventory_key, i, _item_nodes[i])


# invoked by the instantiate function dynamically from native library
func set_inventory_key(inventory_key: int) -> void:
	_inventory_key = inventory_key


# invoked by the signal
func drag_inventory(event: InputEvent) -> void:
	if event is InputEventMouseMotion:
		var is_inside = self.get_viewport_rect().has_point(self.get_global_mouse_position())
		if is_inside and event.button_mask == MOUSE_BUTTON_MASK_LEFT:
			self.position = self.position + event.relative


# invoked by the signal
func close_inventory() -> void:
	self.queue_free()


# invoked by the signal
func cursor_slot(event: InputEvent, local_key: int) -> void:
	if event is InputEventMouseButton:
		var is_inside = self.get_viewport_rect().has_point(self.get_global_mouse_position())
		if event.button_mask == MOUSE_BUTTON_MASK_LEFT:
			if _src_local_key == -1:
				# nothing to do
				_src_local_key = local_key
			else:
				Root.swap_item(_inventory_key, _src_local_key, _inventory_key, local_key)
				_src_local_key = -1
