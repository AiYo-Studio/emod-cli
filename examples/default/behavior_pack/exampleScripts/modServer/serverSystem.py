# -*- coding: utf-8 -*-

import mod.server.extraServerApi as serverApi

from __mod_name_lower__Scripts.modCommon.emodSystem import EasyModBaseSystem

ServerSystem = serverApi.GetServerSystemCls()
compFactory = serverApi.GetEngineCompFactory()
namespace = serverApi.GetEngineNamespace()
engineSystemName = serverApi.GetEngineSystemName()


class __mod_name__ServerSystem(EasyModServerSystem):

    def __init__(self, namespace, systemName):
        super(__mod_name__ServerSystem, self).__init__(namespace, systemName)
        print("===== __mod_name__ServerSystem init =====")

