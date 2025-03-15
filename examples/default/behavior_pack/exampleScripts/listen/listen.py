from mod.common.system.baseSystem import BaseSystem


def listen(event, namespace=None, system_name=None):
    def decorator(func):
        func._annotation_listen = event
        func._listen_namespace = namespace
        func._listen_system_name = system_name
        return func

    return decorator


def inject_listener(instance, system, namespace, system_name):  # type: (object, BaseSystem, str, str) -> None
    for name, method in instance.__class__.__dict__.items():
        if (callable(method) and hasattr(method, '_annotation_listen')
                and hasattr(method, '_listen_namespace')
                and hasattr(method, '_listen_system_name')):
            event = method._annotation_listen
            anno_namespace = method._listen_namespace
            anno_system_name = method._listen_system_name
            final_namespace = anno_namespace if anno_system_name is not None else namespace
            final_system_name = anno_system_name if anno_system_name is not None else system_name
            system.ListenForEvent(final_namespace, final_system_name, event, instance, method)
            print(event)
