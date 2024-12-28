# -*- coding: utf-8 -*-

from mod.common.mod import Mod
import mod.server.extraServerApi as serverApi


@Mod.Binding(name="__mod_name__", version="0.0.1")
class __mod_name__(object):

	def __init__(self):
		print("===== init __mod_name__ mod =====")

	@Mod.InitServer()
	def on_server_init(self):
		print("===== init __mod_name__ server =====")
		serverApi.RegisterSystem("__mod_name__", "__mod_name__ServerSystem", "__mod_name_lower__Scripts.modServer.serverSystem.__mod_name__ServerSystem")

	@Mod.DestroyServer()
	def on_server_destroy(self):
		pass

	@Mod.InitClient()
	def on_init_client(self):
		pass

	@Mod.DestroyClient()
	def on_init_destroy(self):
		pass
