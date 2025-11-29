# -*- coding: utf-8 -*-

import mod.server.extraServerApi as serverApi
import mod.client.extraClientApi as clientApi

from mod.common.system.baseSystem import BaseSystem

from __mod_name_lower__Scripts.listen.listen import inject_listener


class EasyModBaseSystem(serverApi.GetServerSystemCls()):

    def __init__(self, namespace, system_name, engine_namespace, engine_system_name):
        super(EasyModBaseSystem, self).__init__(namespace, system_name)
        inject_listener(self.__class__, self, engine_namespace, engine_system_name)

class EasyModServerSystem(EasyModBaseSystem, ServerSystem):

    def __init__(self, namespace, system_name, engine_namespace, engine_system_name):
        super(EasyModServerSystem, self).__init__(namespace, system_name, engine_namespace, engine_system_name)

class EasyModClientSystem(EasyModBaseSystem):
