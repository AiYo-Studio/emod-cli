# -*- coding: utf-8 -*-

import mod.server.extraServerApi as serverApi

ServerSystem = serverApi.GetServerSystemCls()
compFactory = serverApi.GetEngineCompFactory()
namespace = serverApi.GetEngineNamespace()
engineSystemName = serverApi.GetEngineSystemName()


class __mod_name__ServerSystem(ServerSystem):

    def __init__(self, namespace, systemName):
        super(__mod_name__ServerSystem, self).__init__(namespace, systemName)
        print("===== __mod_name__ServerSystem init =====")

