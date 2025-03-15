# -*- coding: utf-8 -*-

import mod.server.extraServerApi as serverApi

from __mod_name_lower__Scripts.listen.listen import inject_listener

ServerSystem = serverApi.GetServerSystemCls()


class EasyModServerSystem(ServerSystem):

    def __init__(self, namespace, systemName):
        super(EasyModServerSystem, self).__init__(namespace, systemName)
        inject_listener(self.__class__, self, serverApi.GetEngineNamespace(), serverApi.GetEngineSystemName())
