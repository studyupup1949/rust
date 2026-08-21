# Copyright (c) 2012-2019, Mark Peek <mark@peek.org>
# All rights reserved.
#
# See LICENSE file for full license.

from . import AWSObject, AWSProperty
from .validators import integer


class VirtualNodeServiceProvider(AWSProperty):
    props = {
        'VirtualNodeName': (str, True),
    }


class VirtualRouterServiceProvider(AWSProperty):
    props = {
        'VirtualRouterName': (str, True),
    }


class VirtualServiceProvider(AWSProperty):
    props = {
        'VirtualNode': (VirtualNodeServiceProvider, False),
        'VirtualRouter': (VirtualRouterServiceProvider, False),
    }


class VirtualServiceSpec(AWSProperty):
    props = {
        'Provider': (VirtualServiceProvider, False),
    }


class TagRef(AWSProperty):
    props = {
        'Key': (str, True),
        'Value': (str, False),
    }


class VirtualService(AWSObject):
    resource_type = "AWS::AppMesh::VirtualService"

    props = {
        'MeshName': (str, True),
        'Spec': (VirtualServiceSpec, True),
        'Tags': ([TagRef], False),
        'VirtualServiceName': (str, True),
    }


class HealthCheckPolicy(AWSProperty):
    props = {
        'HealthyThreshold': (integer, True),
        'IntervalMillis': (integer, True),
        'Path': (str, False),
        'Port': (integer, False),
        'Protocol': (str, True),
        'TimeoutMillis': (integer, True),
        'UnhealthyThreshold': (integer, True),
    }


class PortMapping(AWSProperty):
    props = {
        'Port': (integer, True),
        'Protocol': (str, True),
    }


class Listener(AWSProperty):
    props = {
        'HealthCheckPolicy': (HealthCheckPolicy, False),
        'PortMapping': (PortMapping, True),
    }


class DnsServiceDiscovery(AWSProperty):
    props = {
        'Hostname': (str, True),
    }


class ServiceDiscovery(AWSProperty):
    props = {
        'DNS': (DnsServiceDiscovery, True),
    }


class FileAccessLog(AWSProperty):
    props = {
        'Path': (str, True),
    }


class AccessLog(AWSProperty):
    props = {
        'File': (FileAccessLog, False),
    }


class Logging(AWSProperty):
    props = {
        'AccessLog': (AccessLog, False),
    }


class VirtualServiceBackend(AWSProperty):
    props = {
        'VirtualServiceName': (str, True),
    }


class Backend(AWSProperty):
    props = {
        'VirtualService': (VirtualServiceBackend, False),
    }


class VirtualNodeSpec(AWSProperty):
    props = {
        'Backends': ([Backend], False),
        'Listeners': ([Listener], False),
        'Logging': (Logging, False),
        'ServiceDiscovery': (ServiceDiscovery, False),
    }


class VirtualNode(AWSObject):
    resource_type = "AWS::AppMesh::VirtualNode"

    props = {
        'MeshName': (str, True),
        'Spec': (VirtualNodeSpec, True),
        'Tags': ([TagRef], False),
        'VirtualNodeName': (str, True),
    }


class WeightedTarget(AWSProperty):
    props = {
        'VirtualNode': (str, True),
        'Weight': (integer, True),
    }


class HttpRouteAction(AWSProperty):
    props = {
        'WeightedTargets': ([WeightedTarget], True),
    }


class HttpRouteMatch(AWSProperty):
    props = {
        'Prefix': (str, True),
    }


class HttpRoute(AWSProperty):
    props = {
        'Action': (HttpRouteAction, True),
        'Match': (HttpRouteMatch, True),
    }


class TcpRouteAction(AWSProperty):
    props = {
        'WeightedTargets': ([WeightedTarget], True),
    }


class TcpRoute(AWSProperty):
    props = {
        'Action': (TcpRouteAction, True),
    }


class RouteSpec(AWSProperty):
    props = {
        'HttpRoute': (HttpRoute, False),
        'TcpRoute': (TcpRoute, False),
    }


class Route(AWSObject):
    resource_type = "AWS::AppMesh::Route"

    props = {
        'MeshName': (str, True),
        'RouteName': (str, True),
        'Spec': (RouteSpec, True),
        'Tags': ([TagRef], False),
        'VirtualRouterName': (str, True),
    }


class EgressFilter(AWSProperty):
    props = {
        'Type': (str, True),
    }


class MeshSpec(AWSProperty):
    props = {
        'EgressFilter': (EgressFilter, False),
    }


class Mesh(AWSObject):
    resource_type = "AWS::AppMesh::Mesh"

    props = {
        'MeshName': (str, True),
        'Spec': (MeshSpec, False),
        'Tags': ([TagRef], False),
    }
